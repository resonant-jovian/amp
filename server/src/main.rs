//! AMP Server - Address-Parking Correlation CLI
//! Supports multiple correlation algorithms, benchmarking, testing with visual verification
extern crate core;
use amp_core::api::api;
use amp_core::benchmark::Benchmarker;
use amp_core::checksum::DataChecksum;
use amp_core::correlation_algorithms::rtree_spatial::RTreeSpatialParkeringAlgo;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, DistanceBasedParkeringAlgo, GridNearestAlgo,
    GridNearestParkeringAlgo, KDTreeParkeringAlgo, KDTreeSpatialAlgo, OverlappingChunksAlgo,
    OverlappingChunksParkeringAlgo, ParkeringCorrelationAlgo, RTreeSpatialAlgo, RaycastingAlgo,
    RaycastingParkeringAlgo,
};
use amp_core::parquet::{write_adress_clean_parquet, write_output_parquet};
use amp_core::structs::{
    AdressClean, CorrelationResult, MiljoeDataClean, OutputData, OutputDataWithDistance,
    ParkeringsDataClean,
};
use clap::{Parser, Subcommand};
use geojson::{Feature, GeoJson};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use std::time::Instant;
#[derive(Parser)]
#[command(name = "amp-server")]
#[command(about = "AMP Address-Parking Correlation Server", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Run correlation with specified algorithm
    Correlate {
        #[arg(short, long, value_enum, default_value_t = AlgorithmChoice::KDTree)]
        algorithm: AlgorithmChoice,
        #[arg(short, long, default_value_t = 20., help = "Distance cutoff in meters")]
        cutoff: f64,
    },
    /// Output correlation results to parquet (for server database or Android app)
    Output {
        #[arg(short, long, value_enum, default_value_t = AlgorithmChoice::KDTree)]
        algorithm: AlgorithmChoice,
        #[arg(short, long, default_value_t = 20., help = "Distance cutoff in meters")]
        cutoff: f64,
        #[arg(short, long, default_value = "db.parquet", help = "Output file path")]
        output: String,
        #[arg(
            short,
            long,
            help = "Also generate Android-formatted local storage (with day/time extraction)"
        )]
        android: bool,
    },
    /// Test correlation with visual browser verification
    Test {
        #[arg(short, long, value_enum, default_value_t = AlgorithmChoice::KDTree)]
        algorithm: AlgorithmChoice,
        #[arg(short, long, default_value_t = 20., help = "Distance cutoff in meters")]
        cutoff: f64,
        #[arg(
            short,
            long,
            default_value_t = 10,
            help = "Number of browser windows to open"
        )]
        windows: usize,
    },
    /// Benchmark all algorithms
    Benchmark {
        #[arg(
            short,
            long,
            default_value_t = 100,
            help = "Number of addresses to test"
        )]
        sample_size: usize,
        #[arg(short, long, default_value_t = 20., help = "Distance cutoff in meters")]
        cutoff: f64,
    },
    /// Check for data updates from Malmö open data portal
    CheckUpdates {
        #[arg(
            short,
            long,
            default_value = "checksums.json",
            help = "Checksum file path"
        )]
        checksum_file: String,
    },
    /// Convert GeoJSON adresser.json to adresser.parquet
    CreateAdresserParquet {
        #[arg(
            short,
            long,
            default_value = "data/adresser.json",
            help = "Input JSON file path"
        )]
        input: String,
        #[arg(
            short,
            long,
            default_value = "android/assets/data/adresser.parquet",
            help = "Output parquet file path"
        )]
        output: String,
    },
}
#[derive(clap::ValueEnum, Clone, Debug)]
enum AlgorithmChoice {
    #[value(name = "distance-based")]
    DistanceBased,
    #[value(name = "raycasting")]
    Raycasting,
    #[value(name = "overlapping-chunks")]
    OverlappingChunks,
    #[value(name = "rtree")]
    RTree,
    #[value(name = "kdtree")]
    KDTree,
    #[value(name = "grid")]
    Grid,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Correlate { algorithm, cutoff } => {
            run_correlation(algorithm, cutoff)?;
        }
        Commands::Output {
            algorithm,
            cutoff,
            output,
            android,
        } => {
            run_output(algorithm, cutoff, &output, android)?;
        }
        Commands::Test {
            algorithm,
            cutoff,
            windows,
        } => {
            run_test_mode(algorithm, cutoff, windows)?;
        }
        Commands::Benchmark {
            sample_size,
            cutoff,
        } => {
            run_benchmark(sample_size, cutoff)?;
        }
        Commands::CheckUpdates { checksum_file } => {
            tokio::runtime::Runtime::new()?.block_on(check_updates(&checksum_file))?
        }
        Commands::CreateAdresserParquet { input, output } => {
            run_create_adresser_parquet(&input, &output)?;
        }
    }
    Ok(())
}
fn run_create_adresser_parquet(input: &str, output: &str) -> anyhow::Result<()> {
    let file = File::open(input)
        .map_err(|e| anyhow::anyhow!("Failed to open input JSON '{}': {}", input, e))?;
    let reader = BufReader::new(file);
    let geojson: GeoJson = serde_json::from_reader(reader)
        .map_err(|e| anyhow::anyhow!("Failed to parse GeoJSON from '{}': {}", input, e))?;
    let fc = match geojson {
        GeoJson::FeatureCollection(fc) => fc,
        _ => anyhow::bail!("Expected a GeoJSON FeatureCollection in '{}'", input),
    };
    let total = fc.features.len() as u64;
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )?
        .progress_chars("##-"),
    );
    let mut records = Vec::with_capacity(fc.features.len());
    for feature in fc.features.into_iter() {
        pb.inc(1);
        if let Some(record) = feature_to_adress_clean(&feature) {
            records.push(record);
        }
    }
    pb.finish_with_message("Parsed adresser, writing parquet...");
    write_adress_clean_parquet(records, output)?;
    pb.finish_with_message("Done writing adresser.parquet");
    Ok(())
}
fn feature_to_adress_clean(feature: &Feature) -> Option<AdressClean> {
    let geometry = feature.geometry.as_ref()?;
    if geometry.value.type_name() != "Point" {
        return None;
    }
    let coords = match geometry.value.clone() {
        geojson::Value::Point(c) => c,
        _ => return None,
    };
    if coords.len() != 2 {
        return None;
    }
    let x = Decimal::from_f64(coords[0])?;
    let y = Decimal::from_f64(coords[1])?;
    let props = feature.properties.as_ref()?;
    let adress = props
        .get("BELADRESS")?
        .as_str()
        .unwrap_or_default()
        .to_string();
    let gata = props
        .get("ADRESSOMR")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let nrnum = props
        .get("NR_NUM")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let nrlitt = props.get("ADRESSPLAT").and_then(|v| v.as_str()).unwrap_or("");
    let gatunummer = if nrlitt.is_empty() {
        nrnum.clone()
    } else {
        format!("{}{}", nrnum, nrlitt)
    };
    let postnummer = props
        .get("POSTNR")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Some(AdressClean {
        coordinates: [x, y],
        postnummer,
        adress,
        gata,
        gatunummer,
    })
}
/// Load asset files (HTML, CSS, JS) from server/src/assets/
fn load_asset_file(filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    let paths = vec![
        format!("server/src/assets/{}", filename),
        format!("src/assets/{}", filename),
        format!("assets/{}", filename),
    ];
    for path in paths {
        if let Ok(content) = fs::read_to_string(&path) {
            return Ok(content);
        }
    }
    Err(format!("Could not find asset file: {}", filename).into())
}
/// Simple base64 encoder for data URIs
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };
        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
        result.push(CHARSET[((n >> 18) & 63) as usize] as char);
        result.push(CHARSET[((n >> 12) & 63) as usize] as char);
        result.push(if i + 1 < data.len() {
            CHARSET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        result.push(if i + 2 < data.len() {
            CHARSET[(n & 63) as usize] as char
        } else {
            '='
        });
        i += 3;
    }
    result
}
/// Create a data URI for HTML content using base64 encoding
fn create_data_uri(html: &str) -> String {
    let base64_encoded = base64_encode(html.as_bytes());
    format!("data:text/html;base64,{}", base64_encoded)
}
/// Prompt user to select which algorithms to benchmark
fn select_algorithms() -> Vec<&'static str> {
    let algorithms = vec![
        "Distance-Based",
        "Raycasting",
        "Overlapping Chunks",
        "R-Tree",
        "KD-Tree",
        "Grid",
    ];
    println!("\n🔧 Algorithm Selection (Y/N to include, default is Y if just Enter is pressed):\n",);
    let mut selected = Vec::new();
    for algo in &algorithms {
        loop {
            print!("  Include {} benchmark? [Y/n]: ", algo);
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            let input = input.trim().to_lowercase();
            if input.is_empty() || input == "y" || input == "yes" {
                selected.push(*algo);
                println!("  ✓ {} selected", algo);
                break;
            } else if input == "n" || input == "no" {
                println!("  ✗ {} skipped", algo);
                break;
            } else {
                println!("  ❌ Invalid input. Please enter Y/N");
            }
        }
    }
    if selected.is_empty() {
        println!("\n⚠️  No algorithms selected! Running all algorithms instead.\n");
        algorithms
    } else {
        println!();
        selected
    }
}
type CorDat = Result<Vec<(String, f64, MiljoeDataClean)>, Box<dyn std::error::Error>>;
/// Generic correlation function for miljoe dataset that handles all algorithms
fn correlate_miljoe_dataset(
    algorithm: &AlgorithmChoice,
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    cutoff: f64,
    pb: &ProgressBar,
) -> CorDat {
    let counter = Arc::new(AtomicUsize::new(0));
    let process_address =
        |addr: &AdressClean, idx: usize, dist: f64| -> Option<(String, f64, MiljoeDataClean)> {
            if dist > cutoff {
                return None;
            }
            let info = zones.get(idx)?.clone();
            let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
            if count.is_multiple_of(100) || count == addresses.len() {
                pb.set_position(count as u64);
            }
            Some((addr.adress.clone(), dist, info))
        };
    let results: Vec<_> = match algorithm {
        AlgorithmChoice::DistanceBased => {
            let algo = DistanceBasedAlgo;
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::Raycasting => {
            let algo = RaycastingAlgo;
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::OverlappingChunks => {
            let algo = OverlappingChunksAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::RTree => {
            let algo = RTreeSpatialAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::KDTree => {
            let algo = KDTreeSpatialAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::Grid => {
            let algo = GridNearestAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
    };
    pb.set_position(addresses.len() as u64);
    Ok(results)
}
type CorPark = Result<Vec<(String, f64, ParkeringsDataClean)>, Box<dyn std::error::Error>>;
/// Generic correlation function for parkering dataset that handles all algorithms
fn correlate_parkering_dataset(
    algorithm: &AlgorithmChoice,
    addresses: &[AdressClean],
    zones: &[ParkeringsDataClean],
    cutoff: f64,
    pb: &ProgressBar,
) -> CorPark {
    let counter = Arc::new(AtomicUsize::new(0));
    let process_address =
        |addr: &AdressClean, idx: usize, dist: f64| -> Option<(String, f64, ParkeringsDataClean)> {
            if dist > cutoff {
                return None;
            }
            let data = zones.get(idx)?.clone();
            let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
            if count.is_multiple_of(100) || count == addresses.len() {
                pb.set_position(count as u64);
            }
            Some((addr.adress.clone(), dist, data))
        };
    let results: Vec<_> = match algorithm {
        AlgorithmChoice::DistanceBased => {
            let algo = DistanceBasedParkeringAlgo;
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::Raycasting => {
            let algo = RaycastingParkeringAlgo;
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::OverlappingChunks => {
            let algo = OverlappingChunksParkeringAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::RTree => {
            let algo = RTreeSpatialParkeringAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::KDTree => {
            let algo = KDTreeParkeringAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
        AlgorithmChoice::Grid => {
            let algo = GridNearestParkeringAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    process_address(addr, idx, dist)
                })
                .collect()
        }
    };
    pb.set_position(addresses.len() as u64);
    Ok(results)
}
/// Merge correlate results from two datasets
fn merge_results(
    addresses: &[AdressClean],
    miljo_results: &[(String, f64, MiljoeDataClean)],
    parkering_results: &[(String, f64, ParkeringsDataClean)],
) -> Vec<OutputDataWithDistance> {
    use std::collections::HashMap;
    let miljo_map: HashMap<_, _> = miljo_results
        .iter()
        .map(|(addr, dist, miljodata)| (addr.clone(), (*dist, miljodata.clone())))
        .collect();
    let parkering_map: HashMap<_, _> = parkering_results
        .iter()
        .map(|(addr, dist, data)| (addr.clone(), (*dist, data.clone())))
        .collect();
    addresses
        .iter()
        .map(|addr| {
            let miljo_data = miljo_map.get(&addr.adress);
            let parkering_data = parkering_map.get(&addr.adress);
            let (info, tid, dag, miljo_distance) = if let Some((dist, miljodata)) = miljo_data {
                (
                    Some(miljodata.info.clone()),
                    Some(miljodata.tid.clone()),
                    Some(miljodata.dag),
                    Some(*dist),
                )
            } else {
                (None, None, None, None)
            };
            let (taxa, antal_platser, typ_av_parkering, parkering_distance) =
                if let Some((dist, p_data)) = parkering_data {
                    (
                        Some(p_data.taxa.clone()),
                        Some(p_data.antal_platser),
                        Some(p_data.typ_av_parkering.clone()),
                        Some(*dist),
                    )
                } else {
                    (None, None, None, None)
                };
            OutputDataWithDistance {
                data: OutputData {
                    postnummer: addr.postnummer.clone(),
                    adress: addr.adress.clone(),
                    gata: addr.gata.clone(),
                    gatunummer: addr.gatunummer.clone(),
                    info,
                    tid,
                    dag,
                    taxa,
                    antal_platser,
                    typ_av_parkering,
                },
                miljo_distance,
                parkering_distance,
            }
        })
        .collect()
}
fn run_correlation(
    algorithm: AlgorithmChoice,
    cutoff: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Loading data...");
    let (addresses, miljodata, parkering): (
        Vec<AdressClean>,
        Vec<MiljoeDataClean>,
        Vec<ParkeringsDataClean>,
    ) = api()?;
    pb.finish_with_message(format!(
        "✓ Loaded {} addresses, {} miljödata zones, {} parkering zones",
        addresses.len(),
        miljodata.len(),
        parkering.len(),
    ));
    println!("\n📋 Dataset Information:");
    println!("  Correlating with: Miljödata + Parkering (dual dataset)");
    println!("  Addresses: {}", addresses.len());
    println!("  Miljödata zones: {}", miljodata.len());
    println!("  Parkering zones: {}\n", parkering.len());
    println!("  Distance threshold: {} meters\n", cutoff);
    let algo_name = format!("{:?}", algorithm);
    println!("🚀 Running correlation with {} algorithm", algo_name);
    let start = Instant::now();
    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {percent}% {msg}")?
            .progress_chars("█▓▒░ "),
    );
    let miljo_results = correlate_miljoe_dataset(&algorithm, &addresses, &miljodata, cutoff, &pb)?;
    let parkering_results =
        correlate_parkering_dataset(&algorithm, &addresses, &parkering, cutoff, &pb)?;
    let merged = merge_results(&addresses, &miljo_results, &parkering_results);
    let both = merged
        .iter()
        .filter(|r| r.data.info.is_some() && r.data.taxa.is_some())
        .count();
    let miljo_only = merged
        .iter()
        .filter(|r| r.data.info.is_some() && r.data.taxa.is_none())
        .count();
    let parkering_only = merged
        .iter()
        .filter(|r| r.data.info.is_none() && r.data.taxa.is_some())
        .count();
    let duration = start.elapsed();
    pb.finish_with_message(format!("✓ Completed in {:.2?}", duration));
    let no_match = merged
        .iter()
        .filter(|r| r.data.info.is_none() && r.data.taxa.is_none())
        .count();
    let total_matches = both + miljo_only + parkering_only;
    println!("\n📊 Results:");
    println!("  Addresses processed: {}", addresses.len());
    println!(
        "  Total matches: {} ({:.1}%)",
        total_matches,
        (total_matches as f64 / addresses.len() as f64) * 100.0,
    );
    println!(
        "  ├─ Both datasets: {} ({:.1}%)",
        both,
        (both as f64 / addresses.len() as f64) * 100.0,
    );
    println!(
        "  ├─ Miljödata only: {} ({:.1}%)",
        miljo_only,
        (miljo_only as f64 / addresses.len() as f64) * 100.0,
    );
    println!(
        "  ├─ Parkering only: {} ({:.1}%)",
        parkering_only,
        (parkering_only as f64 / addresses.len() as f64) * 100.0,
    );
    println!(
        "  └─ No match: {} ({:.1}%)",
        no_match,
        (no_match as f64 / addresses.len() as f64) * 100.0,
    );
    println!(
        "  Average time per address: {:.2?}",
        duration / addresses.len() as u32
    );
    if total_matches == 0 {
        println!("\n⚠️  Warning: No matches found! Check data files.");
    } else {
        let mut rng = thread_rng();
        let mut random_results: Vec<_> = merged
            .iter()
            .filter(|r| r.data.info.is_some() && r.data.taxa.is_some())
            .collect();
        random_results.shuffle(&mut rng);
        println!("\n🎲 10 Random Matches:");
        for result in random_results.iter().take(10) {
            println!(
                "  {} ({})",
                result.data.adress,
                result.data.dataset_source()
            );
            if let Some(dist) = result.miljo_distance {
                println!("    ├─ Miljödata: {:.2}m", dist);
            }
            if let Some(dist) = result.parkering_distance {
                println!("    └─ Parkering: {:.2}m", dist);
            }
        }
        let mut sorted_by_distance: Vec<_> = merged
            .iter()
            .filter(|r| r.data.info.is_some() && r.data.taxa.is_some())
            .collect();
        sorted_by_distance.sort_by(|a, b| {
            b.closest_distance()
                .partial_cmp(&a.closest_distance())
                .unwrap()
        });
        println!(
            "\n📏 10 Addresses with Largest Distances (all should be ≤{}m):",
            cutoff as i32,
        );
        for result in sorted_by_distance.iter().take(10) {
            if let Some(dist) = result.closest_distance() {
                println!(
                    "  {} - {:.2}m ({})",
                    result.data.adress,
                    dist,
                    result.data.dataset_source(),
                );
            }
        }
        let exceeds_threshold = sorted_by_distance
            .iter()
            .any(|r| r.closest_distance().map(|d| d > cutoff).unwrap_or(false));
        if exceeds_threshold {
            println!(
                "\n⚠️  ERROR: Some matches exceed {}m threshold!",
                cutoff as i32,
            );
        } else {
            println!(
                "\n✅ Threshold verification: All matches are within {}m",
                cutoff as i32,
            );
        }
    }
    Ok(())
}
/// Run correlation and output results to parquet file (server database or Android app)
fn run_output(
    algorithm: AlgorithmChoice,
    cutoff: f64,
    output_path: &str,
    generate_android: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Loading data...");
    let (addresses, miljodata, parkering): (
        Vec<AdressClean>,
        Vec<MiljoeDataClean>,
        Vec<ParkeringsDataClean>,
    ) = api()?;
    pb.finish_with_message(format!(
        "✓ Loaded {} addresses, {} miljödata zones, {} parkering zones",
        addresses.len(),
        miljodata.len(),
        parkering.len(),
    ));
    println!("\n📋 Output Configuration:");
    println!("  Algorithm: {:?}", algorithm);
    println!("  Distance cutoff: {} meters", cutoff);
    println!("  Output file: {}", output_path);
    if generate_android {
        println!("  Android format: Enabled (extracting day/time data)");
    }
    println!();
    let start = Instant::now();
    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {percent}% {msg}")?
            .progress_chars("█▓▒░ "),
    );
    pb.set_message("Correlating with miljödata...");
    let miljo_results = correlate_miljoe_dataset(&algorithm, &addresses, &miljodata, cutoff, &pb)?;
    pb.set_message("Correlating with parkering...");
    let parkering_results =
        correlate_parkering_dataset(&algorithm, &addresses, &parkering, cutoff, &pb)?;
    let duration = start.elapsed();
    pb.finish_with_message(format!("✓ Completed in {:.2?}", duration));
    let merged = merge_results(&addresses, &miljo_results, &parkering_results);
    let total_matches = merged.iter().filter(|r| r.data.info.is_some()).count();
    println!("\n✓ Correlation complete");
    println!(
        "  Total matches: {}/{} ({:.1}%)",
        total_matches,
        addresses.len(),
        (total_matches as f64 / addresses.len() as f64) * 100.0,
    );
    println!("\n💾 Writing server parquet file...");
    let output_data: Vec<OutputData> = merged
        .iter()
        .filter(|r| r.data.has_match())
        .map(|r| r.data.clone())
        .collect();
    println!(
        "  Filtered: {}/{} entries with matches ({:.1}%)",
        output_data.len(),
        merged.len(),
        (output_data.len() as f64 / merged.len() as f64) * 100.0,
    );
    write_output_parquet(output_data.clone(), "../android/assets/data/db.parquet")
        .map_err(|e| format!("Failed to write parquet: {}", e))?;
    println!("  ✓ Saved to {}", output_path);
    println!("  ✓ Wrote {} entries with matches", output_data.len());
    println!("\n✅ Output complete!");
    Ok(())
}
fn run_test_mode(
    algorithm: AlgorithmChoice,
    cutoff: f64,
    num_windows: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Loading data for testing...");
    let (addresses, miljodata, parkering): (
        Vec<AdressClean>,
        Vec<MiljoeDataClean>,
        Vec<ParkeringsDataClean>,
    ) = api()?;
    pb.finish_with_message(format!(
        "✓ Loaded {} addresses, {} miljödata zones, {} parkering zones",
        addresses.len(),
        miljodata.len(),
        parkering.len(),
    ));
    println!("\n📋 Test Mode Configuration:");
    println!("  Algorithm: {:?}", algorithm);
    println!("  Distance threshold: {} meters", cutoff);
    println!("  Browser windows to open: {}", num_windows);
    println!("  Total addresses available: {}\n", addresses.len());
    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {percent}%")?
            .progress_chars("█▓▒░ "),
    );
    let miljo_results = correlate_miljoe_dataset(&algorithm, &addresses, &miljodata, cutoff, &pb)?;
    let parkering_results =
        correlate_parkering_dataset(&algorithm, &addresses, &parkering, cutoff, &pb)?;
    pb.finish_with_message("✓ Correlation complete".to_string());
    let merged = merge_results(&addresses, &miljo_results, &parkering_results);
    let matching_addresses: Vec<_> = merged.iter().filter(|r| r.data.has_match()).collect();
    if matching_addresses.is_empty() {
        println!("\n❌ No matching addresses found for testing!");
        return Ok(());
    }
    println!("\n📊 Correlation Results:");
    println!("  Total matches found: {}", matching_addresses.len());
    let actual_windows = num_windows.min(matching_addresses.len());
    println!(
        "  Windows to open: {} (sample size from {} matches)",
        actual_windows,
        matching_addresses.len(),
    );
    let mut rng = thread_rng();
    let mut sampled = matching_addresses.clone();
    sampled.shuffle(&mut rng);
    let selected: Vec<_> = sampled.iter().take(actual_windows).collect();
    println!("\n🌐 Opening {} browser windows...", actual_windows);
    println!("  Each window has 4 integrated tabs with nested StadsAtlas map:");
    println!("    - Tab 1: Address search with nested StadsAtlas map");
    println!("    - Tab 2: Step-by-step instructions");
    println!("    - Tab 3: Correlation data visualization");
    println!("    - Tab 4: Debug console with address search logs\n");
    for (idx, result) in selected.iter().enumerate() {
        let corr_result = CorrelationResult {
            address: result.data.adress.clone(),
            postnummer: result.data.postnummer.clone().unwrap_or_default(),
            miljo_match: result
                .data
                .info
                .as_ref()
                .map(|info| (result.miljo_distance.unwrap_or(0.0), info.clone())),
            parkering_match: result
                .data
                .taxa
                .as_ref()
                .map(|taxa| (result.parkering_distance.unwrap_or(0.0), taxa.clone())),
        };
        println!(
            "  [{}/{}] Opening window for: {}",
            idx + 1,
            actual_windows,
            corr_result.address,
        );
        if let Err(e) = open_browser_window(&corr_result, idx) {
            println!("    ⚠️  Failed to open: {}", e);
        }
        if idx < actual_windows - 1 {
            thread::sleep(Duration::from_millis(500));
        }
    }
    println!("\n✅ Test mode complete!");
    println!(
        "  Review the {} opened windows to verify correlation accuracy.",
        actual_windows,
    );
    Ok(())
}
/// Get the browser executable to use on Linux
fn get_browser_executable() -> String {
    if let Ok(browser) = env::var("BROWSER")
        && !browser.is_empty()
    {
        return browser;
    }
    let common_browsers = vec![
        "firefox",
        "chromium",
        "chromium-browser",
        "google-chrome",
        "chrome",
    ];
    for browser in common_browsers {
        if std::process::Command::new("which")
            .arg(browser)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return browser.to_string();
        }
    }
    "firefox".to_string()
}
fn format_matches_html(result: &CorrelationResult) -> String {
    match (&result.miljo_match, &result.parkering_match) {
        (Some((dist_m, info_m)), Some((dist_p, info_p))) => {
            format!(
                "\n          <div class=\"match-card\">\n            <div class=\"match-icon\">🌍 Miljödata</div>\n            <div class=\"match-distance\">{:.2}m away</div>\n            <div class=\"match-info\">{}</div>\n          </div>\n\n          <div class=\"match-card\">\n            <div class=\"match-icon\">🅿️ Parkering</div>\n            <div class=\"match-distance\">{:.2}m away</div>\n            <div class=\"match-info\">{}</div>\n          </div>",
                dist_m, info_m, dist_p, info_p,
            )
        }
        (Some((dist, info)), None) => {
            format!(
                "\n          <div class=\"match-card\">\n            <div class=\"match-icon\">🌍 Miljödata</div>\n            <div class=\"match-distance\">{:.2}m away</div>\n            <div class=\"match-info\">{}</div>\n          </div>",
                dist, info,
            )
        }
        (None, Some((dist, info))) => {
            format!(
                "\n          <div class=\"match-card\">\n            <div class=\"match-icon\">🅿️ Parkering</div>\n            <div class=\"match-distance\">{:.2}m away</div>\n            <div class=\"match-info\">{}</div>\n          </div>",
                dist, info,
            )
        }
        (None, None) => "          <div>✗ No matches found</div>".to_string(),
    }
}
/// Create HTML page by loading template and inlining CSS/JS, with origo_map.html as embedded data URI
fn create_tabbed_interface_page(
    address: &str,
    result: &CorrelationResult,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = load_asset_file("stadsatlas_interface.html")?;
    let css = load_asset_file("stadsatlas_interface.css")?;
    let mut js = load_asset_file("stadsatlas_interface.js")?;
    let origo_map_html = load_asset_file("origo_map.html")?;
    let matches_html = format_matches_html(result);
    let address_escaped = address.replace('"', "&quot;");
    let origo_data_uri = create_data_uri(&origo_map_html);
    js = js.replace("{ORIGO_DATA_URI}", &origo_data_uri);
    html = html.replace("{ADDRESS}", &address_escaped);
    html = html.replace("{RESULT_ADDRESS}", &result.address);
    html = html.replace("{RESULT_POSTNUMMER}", &result.postnummer);
    html = html.replace("{RESULT_SOURCE}", result.dataset_source());
    html = html.replace("{RESULT_MATCHES}", &matches_html);
    html = html.replace(
        "<link rel=\"stylesheet\" href=\"stadsatlas_interface.css\">",
        &format!("<style>{}</style>", css),
    );
    html = html.replace(
        "<script src=\"stadsatlas_interface.js\"></script>",
        &format!("<script>\n{}\n</script>", js),
    );
    Ok(html)
}
/// Open a single browser window with integrated tabbed interface
fn open_browser_window(
    result: &CorrelationResult,
    window_idx: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = &result.address;
    let tabbed_page = create_tabbed_interface_page(address, result)?;
    let temp_dir = env::temp_dir();
    let filename = format!("amp_test_{}.html", window_idx);
    let temp_file = temp_dir.join(&filename);
    fs::write(&temp_file, &tabbed_page)?;
    #[allow(unused)]
    let file_url = format!("file://{}", temp_file.display());
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", &format!("start chrome \"{}\"", file_url)])
            .output()
            .ok();
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("bash")
            .args(&["-c", &format!("open '{}'", file_url)])
            .output()
            .ok();
    }
    #[cfg(target_os = "linux")]
    {
        let browser = get_browser_executable();
        std::process::Command::new(&browser)
            .arg(&file_url)
            .spawn()
            .ok();
    }
    Ok(())
}
fn run_benchmark(sample_size: usize, cutoff: f64) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Loading data for benchmarking...");
    let (addresses, zones) = amp_core::api::api_miljo_only()?;
    let actual_sample_size = sample_size.min(addresses.len());
    let requested_msg = if sample_size > addresses.len() {
        format!(
            " (requested {} but only {} available)",
            sample_size,
            addresses.len()
        )
    } else {
        String::new()
    };
    pb.finish_with_message(format!(
        "✓ Loaded {} addresses, {} zones{}",
        addresses.len(),
        zones.len(),
        requested_msg,
    ));
    let selected_algos = select_algorithms();
    let benchmarker = Benchmarker::new(addresses, zones);
    println!(
        "🏁 Benchmarking {} selected algorithm(s) with {} samples (distance cutoff: {}m)\n",
        selected_algos.len(),
        actual_sample_size,
        cutoff as i32,
    );
    let multi_pb = MultiProgress::new();
    let pbs: Vec<_> = selected_algos
        .iter()
        .map(|name| {
            let pb = multi_pb.add(ProgressBar::new(actual_sample_size as u64));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!(
                        "{{spinner:.green}} [{:20}] [{{bar:30.cyan/blue}}] {{pos}}/{{len}} {{msg}}",
                        name,
                    ))
                    .unwrap()
                    .progress_chars("█▓▒░ "),
            );
            pb.set_message("waiting...");
            pb
        })
        .collect();
    let results = benchmark_selected_with_progress(
        &benchmarker,
        actual_sample_size,
        &selected_algos,
        &pbs,
        cutoff,
    );
    for pb in pbs {
        pb.finish_and_clear();
    }
    println!(
        "\n📊 Benchmark Results (distance cutoff: {}m):\n",
        cutoff as i32
    );
    Benchmarker::print_results(&results);
    Ok(())
}
type AlgorithmBenchmarkFn =
    Box<dyn Fn(&Benchmarker, &[AdressClean], &ProgressBar, &AtomicUsize, &Arc<AtomicUsize>, f64)>;
fn benchmark_selected_with_progress(
    benchmarker: &Benchmarker,
    sample_size: usize,
    selected_algos: &[&str],
    pbs: &[ProgressBar],
    cutoff: f64,
) -> Vec<amp_core::benchmark::BenchmarkResult> {
    use amp_core::benchmark::BenchmarkResult;
    let addresses_to_test = &benchmarker.addresses[..sample_size];
    let mut results = Vec::new();
    let all_algos: Vec<(&str, AlgorithmBenchmarkFn)> = vec![
        (
            "Distance-Based",
            Box::new(|bm, addrs, pb, matches, counter, cutoff| {
                let algo = DistanceBasedAlgo;
                run_single_benchmark(
                    &algo,
                    addrs,
                    &bm.parking_lines,
                    pb,
                    matches,
                    counter,
                    "Distance-Based",
                    cutoff,
                );
            }),
        ),
        (
            "Raycasting",
            Box::new(|bm, addrs, pb, matches, counter, cutoff| {
                let algo = RaycastingAlgo;
                run_single_benchmark(
                    &algo,
                    addrs,
                    &bm.parking_lines,
                    pb,
                    matches,
                    counter,
                    "Raycasting",
                    cutoff,
                );
            }),
        ),
        (
            "Overlapping Chunks",
            Box::new(|bm, addrs, pb, matches, counter, cutoff| {
                let algo = OverlappingChunksAlgo::new(&bm.parking_lines);
                run_single_benchmark(
                    &algo,
                    addrs,
                    &bm.parking_lines,
                    pb,
                    matches,
                    counter,
                    "Overlapping Chunks",
                    cutoff,
                );
            }),
        ),
        (
            "R-Tree",
            Box::new(|bm, addrs, pb, matches, counter, cutoff| {
                let algo = RTreeSpatialAlgo::new(&bm.parking_lines);
                run_single_benchmark(
                    &algo,
                    addrs,
                    &bm.parking_lines,
                    pb,
                    matches,
                    counter,
                    "R-Tree",
                    cutoff,
                );
            }),
        ),
        (
            "KD-Tree",
            Box::new(|bm, addrs, pb, matches, counter, cutoff| {
                let algo = KDTreeSpatialAlgo::new(&bm.parking_lines);
                run_single_benchmark(
                    &algo,
                    addrs,
                    &bm.parking_lines,
                    pb,
                    matches,
                    counter,
                    "KD-Tree",
                    cutoff,
                );
            }),
        ),
        (
            "Grid",
            Box::new(|bm, addrs, pb, matches, counter, cutoff| {
                let algo = GridNearestAlgo::new(&bm.parking_lines);
                run_single_benchmark(
                    &algo,
                    addrs,
                    &bm.parking_lines,
                    pb,
                    matches,
                    counter,
                    "Grid",
                    cutoff,
                );
            }),
        ),
    ];
    let mut pb_idx = 0;
    for (name, run_fn) in all_algos.iter() {
        if !selected_algos.contains(name) {
            continue;
        }
        pbs[pb_idx].set_message("running...");
        let start = Instant::now();
        let matches = AtomicUsize::new(0);
        let counter = Arc::new(AtomicUsize::new(0));
        run_fn(
            benchmarker,
            addresses_to_test,
            &pbs[pb_idx],
            &matches,
            &counter,
            cutoff,
        );
        let total_duration = start.elapsed();
        let avg_per_address = total_duration / addresses_to_test.len() as u32;
        pbs[pb_idx].finish_with_message(format!("✓ {:.2?}", total_duration));
        results.push(BenchmarkResult {
            algorithm_name: name.to_string(),
            total_duration,
            avg_per_address,
            addresses_processed: addresses_to_test.len(),
            matches_found: matches.load(Ordering::Relaxed),
        });
        pb_idx += 1;
    }
    results
}
#[allow(clippy::too_many_arguments)]
fn run_single_benchmark<A: CorrelationAlgo + Sync>(
    algo: &A,
    addresses: &[AdressClean],
    parking_lines: &[MiljoeDataClean],
    pb: &ProgressBar,
    matches: &AtomicUsize,
    counter: &Arc<AtomicUsize>,
    _name: &str,
    cutoff: f64,
) {
    addresses.par_iter().for_each(|address| {
        if let Some((_, dist)) = algo.correlate(address, parking_lines)
            && dist <= cutoff
        {
            matches.fetch_add(1, Ordering::Relaxed);
        }
        let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
        if count.is_multiple_of(5) || count == addresses.len() {
            pb.set_position(count as u64);
        }
    });
    pb.set_position(addresses.len() as u64);
}
async fn check_updates(checksum_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Checking for data updates...\n");
    let old_checksums = DataChecksum::load_from_file(checksum_file).ok();
    let mut new_checksums = DataChecksum::new(
        "https://opendata.malmo.se/@fastighets-och-gatukontoret/miljoparkering/73490f00-0d71-4b17-903c-f77ab7664a53"
            .to_string(),
        "https://opendata.malmo.se/@fastighets-och-gatukontoret/parkeringsavgifter/1a6bd68b-30ca-40a5-9d62-01e2a566982e"
            .to_string(),
        "https://opendata.malmo.se/@stadsbyggnadskontoret/adresser/caf1cee8-9af2-4a75-8fb7-f1d7cb11daeb"
            .to_string(),
    );
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Fetching remote data...");
    new_checksums.update_from_remote().await?;
    pb.finish_with_message("✓ Data fetched");
    if let Some(old) = old_checksums {
        if new_checksums.has_changed(&old) {
            println!("\n✓ Data has changed!");
            println!("  Old checksums from: {}", old.last_checked);
            println!("  New checksums from: {}", new_checksums.last_checked);
        } else {
            println!("\n✓ Data is up to date (no changes detected)");
        }
    } else {
        println!("\n✓ No previous checksums found - created new baseline");
    }
    new_checksums.save_to_file(checksum_file)?;
    println!("✓ Checksums saved to {}\n", checksum_file);
    Ok(())
}
