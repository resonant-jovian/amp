//! AMP Server - Address-Parking Correlation CLI
//! Supports multiple correlation algorithms, benchmarking, testing with visual verification
use amp_core::api::api;
use amp_core::benchmark::Benchmarker;
use amp_core::checksum::DataChecksum;
use amp_core::correlation_algorithms::rtree_spatial::RTreeParkeringAlgo;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, DistanceBasedParkeringAlgo, GridNearestAlgo,
    GridNearestParkeringAlgo, KDTreeParkeringAlgo, KDTreeSpatialAlgo, OverlappingChunksAlgo,
    OverlappingChunksParkeringAlgo, ParkeringCorrelationAlgo, RTreeSpatialAlgo, RaycastingAlgo,
    RaycastingParkeringAlgo,
};
use amp_core::parquet::write_output_parquet;
use amp_core::structs::{
    AdressClean, CorrelationResult, MiljoeDataClean, OutputData, OutputDataWithDistance,
    ParkeringsDataClean,
};
use clap::{Parser, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use std::time::Instant;
mod classification;
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
        #[arg(short, long, default_value_t = 50., help = "Distance cutoff in meters")]
        cutoff: f64,
    },
    /// Output correlation results to parquet (for server database or Android app)
    Output {
        #[arg(short, long, value_enum, default_value_t = AlgorithmChoice::KDTree)]
        algorithm: AlgorithmChoice,
        #[arg(short, long, default_value_t = 50., help = "Distance cutoff in meters")]
        cutoff: f64,
        #[arg(
            short,
            long,
            default_value = "correlation_results.parquet",
            help = "Output file path"
        )]
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
        #[arg(short, long, default_value_t = 50., help = "Distance cutoff in meters")]
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
        #[arg(short, long, default_value_t = 50., help = "Distance cutoff in meters")]
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
    }
    Ok(())
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
            let algo = RTreeParkeringAlgo::new(zones);
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
// ... rest of the file unchanged ...
