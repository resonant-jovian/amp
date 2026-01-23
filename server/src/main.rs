//! AMP Server - Address-Parking Correlation CLI
//! Supports multiple correlation algorithms, benchmarking, and dual dataset correlation

use amp_core::api::api;
use amp_core::benchmark::Benchmarker;
use amp_core::checksum::DataChecksum;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, GridNearestAlgo, KDTreeSpatialAlgo, OverlappingChunksAlgo,
    RTreeSpatialAlgo, RaycastingAlgo,
};
use amp_core::structs::{AdressClean, CorrelationResult, MiljoeDataClean};
use clap::{Parser, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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
        #[arg(short, long, value_enum, default_value_t = AlgorithmChoice::RTree)]
        algorithm: AlgorithmChoice,
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
        Commands::Correlate { algorithm } => {
            run_correlation(algorithm)?;
        }
        Commands::Benchmark { sample_size } => {
            run_benchmark(sample_size)?;
        }
        Commands::CheckUpdates { checksum_file } => {
            tokio::runtime::Runtime::new()?.block_on(check_updates(&checksum_file))?;
        }
    }

    Ok(())
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

    println!("\n🔧 Algorithm Selection (Y/N to include, default is Y if just Enter is pressed):\n");

    let mut selected = Vec::new();

    for algo in &algorithms {
        loop {
            print!("   Include {} benchmark? [Y/n]: ", algo);
            io::stdout().flush().ok();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            let input = input.trim().to_lowercase();

            // Default to "y" if just Enter is pressed
            if input.is_empty() || input == "y" || input == "yes" {
                selected.push(*algo);
                println!("      ✓ {} selected", algo);
                break;
            } else if input == "n" || input == "no" {
                println!("      ✗ {} skipped", algo);
                break;
            } else {
                println!("      ❌ Invalid input. Please enter Y/N");
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

fn run_correlation(algorithm: AlgorithmChoice) -> Result<(), Box<dyn std::error::Error>> {
    // Load data with progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Loading data...");

    let (addresses, miljodata, parkering): (
        Vec<AdressClean>,
        Vec<MiljoeDataClean>,
        Vec<MiljoeDataClean>,
    ) = api()?;
    pb.finish_with_message(format!(
        "✓ Loaded {} addresses, {} miljödata zones, {} parkering zones",
        addresses.len(),
        miljodata.len(),
        parkering.len()
    ));

    // Show which datasets are being used
    println!("\n📋 Dataset Information:");
    println!("   Correlating with: Miljödata + Parkering (dual dataset)");
    println!("   Addresses: {}", addresses.len());
    println!("   Miljödata zones: {}", miljodata.len());
    println!("   Parkering zones: {}", parkering.len());
    println!("   Max distance threshold: 50 meters\n");

    // Setup algorithm
    let algo_name = format!("{:?}", algorithm);
    println!("🚀 Running correlation with {} algorithm", algo_name);

    let start = Instant::now();

    // Create progress bar
    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {percent}% {msg}")?
            .progress_chars("█▓▒░ "),
    );

    // Correlate with miljödata
    pb.set_message("Correlating with miljödata...");
    let miljo_results = correlate_dataset(&algorithm, &addresses, &miljodata, &pb)?;

    // Correlate with parkering
    pb.set_message("Correlating with parkering...");
    let parkering_results = correlate_dataset(&algorithm, &addresses, &parkering, &pb)?;

    let duration = start.elapsed();
    pb.finish_with_message(format!("✓ Completed in {:.2?}", duration));

    // Merge results
    let merged = merge_results(&addresses, &miljo_results, &parkering_results);

    // Calculate statistics
    let both = merged
        .iter()
        .filter(|r| r.miljo_match.is_some() && r.parkering_match.is_some())
        .count();
    let miljo_only = merged
        .iter()
        .filter(|r| r.miljo_match.is_some() && r.parkering_match.is_none())
        .count();
    let parkering_only = merged
        .iter()
        .filter(|r| r.miljo_match.is_none() && r.parkering_match.is_some())
        .count();
    let no_match = merged.iter().filter(|r| !r.has_match()).count();
    let total_matches = both + miljo_only + parkering_only;

    println!("\n📊 Results:");
    println!("   Addresses processed: {}", addresses.len());
    println!(
        "   Total matches: {} ({:.1}%)",
        total_matches,
        (total_matches as f64 / addresses.len() as f64) * 100.0
    );
    println!(
        "   ├─ Both datasets: {} ({:.1}%)",
        both,
        (both as f64 / addresses.len() as f64) * 100.0
    );
    println!(
        "   ├─ Miljödata only: {} ({:.1}%)",
        miljo_only,
        (miljo_only as f64 / addresses.len() as f64) * 100.0
    );
    println!(
        "   ├─ Parkering only: {} ({:.1}%)",
        parkering_only,
        (parkering_only as f64 / addresses.len() as f64) * 100.0
    );
    println!(
        "   └─ No match: {} ({:.1}%)",
        no_match,
        (no_match as f64 / addresses.len() as f64) * 100.0
    );
    println!(
        "   Average time per address: {:.2?}",
        duration / addresses.len() as u32
    );

    if total_matches == 0 {
        println!("\n⚠️  Warning: No matches found! Check data files.");
    } else {
        // Show 10 random matches
        let mut rng = thread_rng();
        let mut random_results: Vec<_> = merged.iter().filter(|r| r.has_match()).collect();
        random_results.shuffle(&mut rng);

        println!("\n🎲 10 Random Matches:");
        for result in random_results.iter().take(10) {
            println!("   {} ({})", result.address, result.dataset_source());
            if let Some((dist, _)) = &result.miljo_match {
                println!("      ├─ Miljödata: {:.2}m", dist);
            }
            if let Some((dist, _)) = &result.parkering_match {
                println!("      └─ Parkering: {:.2}m", dist);
            }
        }

        // Show addresses with largest distances
        let mut sorted_by_distance: Vec<_> = merged.iter().filter(|r| r.has_match()).collect();
        sorted_by_distance.sort_by(|a, b| {
            b.closest_distance()
                .partial_cmp(&a.closest_distance())
                .unwrap()
        });

        println!("\n📏 10 Addresses with Largest Distances (all should be ≤50m):");
        for result in sorted_by_distance.iter().take(10) {
            if let Some(dist) = result.closest_distance() {
                println!(
                    "   {} - {:.2}m ({})",
                    result.address,
                    dist,
                    result.dataset_source()
                );
            }
        }

        // Verify threshold
        let exceeds_threshold = sorted_by_distance
            .iter()
            .any(|r| r.closest_distance().map(|d| d > 50.0).unwrap_or(false));

        if exceeds_threshold {
            println!("\n⚠️  ERROR: Some matches exceed 50m threshold!");
        } else {
            println!("\n✅ Threshold verification: All matches are within 50m");
        }
    }

    Ok(())
}
type Dat = Result<Vec<(String, f64, String)>, Box<dyn std::error::Error>>;
/// Correlate addresses with a dataset using the specified algorithm
fn correlate_dataset(
    algorithm: &AlgorithmChoice,
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    pb: &ProgressBar,
) -> Dat {
    let counter = Arc::new(AtomicUsize::new(0));

    let results: Vec<_> = match algorithm {
        AlgorithmChoice::DistanceBased => {
            let algo = DistanceBasedAlgo;
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();

                    let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if count.is_multiple_of(100) || count == addresses.len() {
                        pb.set_position(count as u64);
                    }

                    Some((addr.adress.clone(), dist, info))
                })
                .collect()
        }
        AlgorithmChoice::Raycasting => {
            let algo = RaycastingAlgo;
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();

                    let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if count.is_multiple_of(100) || count == addresses.len() {
                        pb.set_position(count as u64);
                    }

                    Some((addr.adress.clone(), dist, info))
                })
                .collect()
        }
        AlgorithmChoice::OverlappingChunks => {
            let algo = OverlappingChunksAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();

                    let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if count.is_multiple_of(100) || count == addresses.len() {
                        pb.set_position(count as u64);
                    }

                    Some((addr.adress.clone(), dist, info))
                })
                .collect()
        }
        AlgorithmChoice::RTree => {
            let algo = RTreeSpatialAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();

                    let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if count.is_multiple_of(100) || count == addresses.len() {
                        pb.set_position(count as u64);
                    }

                    Some((addr.adress.clone(), dist, info))
                })
                .collect()
        }
        AlgorithmChoice::KDTree => {
            let algo = KDTreeSpatialAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();

                    let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if count.is_multiple_of(100) || count == addresses.len() {
                        pb.set_position(count as u64);
                    }

                    Some((addr.adress.clone(), dist, info))
                })
                .collect()
        }
        AlgorithmChoice::Grid => {
            let algo = GridNearestAlgo::new(zones);
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();

                    let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if count.is_multiple_of(100) || count == addresses.len() {
                        pb.set_position(count as u64);
                    }

                    Some((addr.adress.clone(), dist, info))
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
    miljo_results: &[(String, f64, String)],
    parkering_results: &[(String, f64, String)],
) -> Vec<CorrelationResult> {
    let miljo_map: std::collections::HashMap<_, _> = miljo_results
        .iter()
        .map(|(addr, dist, info)| (addr.clone(), (*dist, info.clone())))
        .collect();

    let parkering_map: std::collections::HashMap<_, _> = parkering_results
        .iter()
        .map(|(addr, dist, info)| (addr.clone(), (*dist, info.clone())))
        .collect();

    addresses
        .iter()
        .map(|addr| {
            let miljo_match = miljo_map.get(&addr.adress).map(|(d, i)| (*d, i.clone()));
            let parkering_match = parkering_map
                .get(&addr.adress)
                .map(|(d, i)| (*d, i.clone()));

            CorrelationResult {
                address: addr.adress.clone(),
                postnummer: addr.postnummer.clone(),
                miljo_match,
                parkering_match,
            }
        })
        .collect()
}

fn run_benchmark(sample_size: usize) -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Loading data for benchmarking...");

    let (addresses, zones) = amp_core::api::api_miljo_only()?;
    pb.finish_with_message(format!(
        "✓ Loaded {} addresses, {} zones",
        addresses.len(),
        zones.len()
    ));

    // Let user select which algorithms to benchmark
    let selected_algos = select_algorithms();

    let benchmarker = Benchmarker::new(addresses, zones);

    println!(
        "🏁 Benchmarking {} selected algorithm(s) with {} samples\n",
        selected_algos.len(),
        sample_size
    );

    // Create multi-progress for selected algorithms
    let multi_pb = MultiProgress::new();

    // Create progress bars for each selected algorithm
    let pbs: Vec<_> = selected_algos
        .iter()
        .map(|name| {
            let pb = multi_pb.add(ProgressBar::new(sample_size as u64));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!(
                        "{{spinner:.green}} [{:20}] [{{bar:30.cyan/blue}}] {{pos}}/{{len}} {{msg}}",
                        name
                    ))
                    .unwrap()
                    .progress_chars("█▓▒░ "),
            );
            pb.set_message("waiting...");
            pb
        })
        .collect();

    // Run benchmarks with progress updates
    let results =
        benchmark_selected_with_progress(&benchmarker, sample_size, &selected_algos, &pbs);

    // Finish all progress bars
    for pb in pbs {
        pb.finish_and_clear();
    }

    println!("\n📊 Benchmark Results:\n");
    Benchmarker::print_results(&results);

    Ok(())
}

type Alg<'a> = Vec<(
    &'a str,
    fn(&Benchmarker, &[AdressClean], &ProgressBar, &AtomicUsize, &Arc<AtomicUsize>) -> (),
)>;
fn benchmark_selected_with_progress(
    benchmarker: &Benchmarker,
    sample_size: usize,
    selected_algos: &[&str],
    pbs: &[ProgressBar],
) -> Vec<amp_core::benchmark::BenchmarkResult> {
    use amp_core::benchmark::BenchmarkResult;

    let sample_size = sample_size.min(benchmarker.addresses.len());
    let addresses_to_test = &benchmarker.addresses[..sample_size];

    let mut results = Vec::new();

    let all_algos: Alg = vec![
        ("Distance-Based", |bm, addrs, pb, matches, counter| {
            let algo = DistanceBasedAlgo;
            run_single_benchmark(
                &algo,
                addrs,
                &bm.parking_lines,
                pb,
                matches,
                counter,
                "Distance-Based",
            );
        }),
        ("Raycasting", |bm, addrs, pb, matches, counter| {
            let algo = RaycastingAlgo;
            run_single_benchmark(
                &algo,
                addrs,
                &bm.parking_lines,
                pb,
                matches,
                counter,
                "Raycasting",
            );
        }),
        ("Overlapping Chunks", |bm, addrs, pb, matches, counter| {
            let algo = OverlappingChunksAlgo::new(&bm.parking_lines);
            run_single_benchmark(
                &algo,
                addrs,
                &bm.parking_lines,
                pb,
                matches,
                counter,
                "Overlapping Chunks",
            );
        }),
        ("R-Tree", |bm, addrs, pb, matches, counter| {
            let algo = RTreeSpatialAlgo::new(&bm.parking_lines);
            run_single_benchmark(
                &algo,
                addrs,
                &bm.parking_lines,
                pb,
                matches,
                counter,
                "R-Tree",
            );
        }),
        ("KD-Tree", |bm, addrs, pb, matches, counter| {
            let algo = KDTreeSpatialAlgo::new(&bm.parking_lines);
            run_single_benchmark(
                &algo,
                addrs,
                &bm.parking_lines,
                pb,
                matches,
                counter,
                "KD-Tree",
            );
        }),
        ("Grid", |bm, addrs, pb, matches, counter| {
            let algo = GridNearestAlgo::new(&bm.parking_lines);
            run_single_benchmark(
                &algo,
                addrs,
                &bm.parking_lines,
                pb,
                matches,
                counter,
                "Grid",
            );
        }),
    ];

    let mut pb_idx = 0;
    for (name, run_fn) in all_algos.iter() {
        // Only run if this algorithm was selected
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

fn run_single_benchmark<A: CorrelationAlgo + Sync>(
    algo: &A,
    addresses: &[AdressClean],
    parking_lines: &[MiljoeDataClean],
    pb: &ProgressBar,
    matches: &AtomicUsize,
    counter: &Arc<AtomicUsize>,
    _name: &str,
) {
    addresses.par_iter().for_each(|address| {
        if algo.correlate(address, parking_lines).is_some() {
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
        "https://opendata.malmo.se/@fastighets-och-gatukontoret/miljoparkering/73490f00-0d71-4b17-903c-f77ab7664a53".to_string(),
        "https://opendata.malmo.se/@fastighets-och-gatukontoret/parkeringsavgifter/1a6bd68b-30ca-40a5-9d62-01e2a566982e".to_string(),
        "https://opendata.malmo.se/@stadsbyggnadskontoret/adresser/caf1cee8-9af2-4a75-8fb7-f1d7cb11daeb".to_string(),
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
