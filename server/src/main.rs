//! AMP Server - Address-Parking Correlation CLI
//! Supports multiple correlation algorithms, benchmarking, and checksum verification

use amp_core::api::api;
use amp_core::structs::{AdressClean, MiljoeDataClean};
use amp_core::correlation_algorithms::{
    DistanceBasedAlgo, RaycastingAlgo, OverlappingChunksAlgo, 
    RTreeSpatialAlgo, KDTreeSpatialAlgo, GridNearestAlgo,
    CorrelationAlgo
};
use amp_core::checksum::DataChecksum;
use amp_core::benchmark::Benchmarker;
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use rand::seq::SliceRandom;
use rand::thread_rng;

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
        
        #[arg(short, long, help = "Path to miljö parking JSON file (optional)")]
        miljo_file: Option<String>,
        
        #[arg(short, long, help = "Path to parkering avgifter JSON file (optional)")]
        parkering_file: Option<String>,
        
        #[arg(short, long, help = "Path to addresses JSON file (optional)")]
        addresses_file: Option<String>,
    },
    
    /// Benchmark all algorithms
    Benchmark {
        #[arg(short, long, help = "Path to miljö parking JSON file (optional)")]
        miljo_file: Option<String>,
        
        #[arg(short, long, help = "Path to addresses JSON file (optional)")]
        addresses_file: Option<String>,
        
        #[arg(short, long, default_value_t = 100, help = "Number of addresses to test")]
        sample_size: usize,
    },
    
    /// Check for data updates from Malmö open data portal
    CheckUpdates {
        #[arg(short, long, default_value = "checksums.json", help = "Checksum file path")]
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
        Commands::Correlate { algorithm, miljo_file, parkering_file, addresses_file } => {
            run_correlation(algorithm, miljo_file, parkering_file, addresses_file)?;
        }
        Commands::Benchmark { miljo_file, addresses_file, sample_size } => {
            run_benchmark(miljo_file, addresses_file, sample_size)?;
        }
        Commands::CheckUpdates { checksum_file } => {
            tokio::runtime::Runtime::new()?.block_on(check_updates(&checksum_file))?;
        }
    }
    
    Ok(())
}

fn run_correlation(
    algorithm: AlgorithmChoice,
    _miljo_file: Option<String>,
    _parkering_file: Option<String>,
    _addresses_file: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load data with progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")?
    );
    pb.set_message("Loading data...");
    
    let (addresses, zones): (Vec<AdressClean>, Vec<MiljoeDataClean>) = api()?;
    pb.finish_with_message(format!("✓ Loaded {} addresses, {} zones", addresses.len(), zones.len()));
    
    // Show which dataset is being used
    println!("\n📋 Dataset Information:");
    println!("   Source: Miljödata (Environmental Parking Zones)");
    println!("   Note: Only correlating with miljödata, not parkering avgifter");
    println!("   Max distance threshold: 50 meters");
    
    // Setup algorithm
    let algo_name = format!("{:?}", algorithm);
    println!("\n🚀 Running correlation with {} algorithm\n", algo_name);
    
    let start = Instant::now();
    
    // Create progress bar with Arch/pacman style
    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {percent}% {msg}")?
            .progress_chars("█▓▒░ ")
    );
    
    let results = match algorithm {
        AlgorithmChoice::DistanceBased => {
            let algo = DistanceBasedAlgo;
            correlate_with_progress(&addresses, &zones, &algo, &pb)
        }
        AlgorithmChoice::Raycasting => {
            let algo = RaycastingAlgo;
            correlate_with_progress(&addresses, &zones, &algo, &pb)
        }
        AlgorithmChoice::OverlappingChunks => {
            let algo = OverlappingChunksAlgo::new(&zones);
            correlate_with_progress(&addresses, &zones, &algo, &pb)
        }
        AlgorithmChoice::RTree => {
            let algo = RTreeSpatialAlgo::new(&zones);
            correlate_with_progress(&addresses, &zones, &algo, &pb)
        }
        AlgorithmChoice::KDTree => {
            let algo = KDTreeSpatialAlgo::new(&zones);
            correlate_with_progress(&addresses, &zones, &algo, &pb)
        }
        AlgorithmChoice::Grid => {
            let algo = GridNearestAlgo::new(&zones);
            correlate_with_progress(&addresses, &zones, &algo, &pb)
        }
    };
    
    let duration = start.elapsed();
    pb.finish_with_message(format!("✓ Completed in {:.2?}", duration));
    
    let match_percentage = (results.len() as f64 / addresses.len() as f64) * 100.0;
    
    println!("\n📊 Results:");
    println!("   Addresses processed: {}", addresses.len());
    println!("   Matches found: {} ({:.1}%)", results.len(), match_percentage);
    println!("   No match: {} ({:.1}%)", 
        addresses.len() - results.len(),
        ((addresses.len() - results.len()) as f64 / addresses.len() as f64) * 100.0
    );
    println!("   Average time per address: {:.2?}", duration / addresses.len() as u32);
    
    if results.is_empty() {
        println!("\n⚠️  Warning: No matches found! Check data files.");
    } else {
        // Show 10 random matches
        let mut rng = thread_rng();
        let mut random_results = results.clone();
        random_results.shuffle(&mut rng);
        
        println!("\n🎲 10 Random Matches:");
        for (addr, dist) in random_results.iter().take(10) {
            println!("   {} - {:.2}m", addr, dist);
        }
        
        // Show 10 largest distances (should all be ≤50m)
        let mut sorted_by_distance = results.clone();
        sorted_by_distance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        println!("\n📏 10 Addresses with Largest Distances (all should be ≤50m):");
        for (addr, dist) in sorted_by_distance.iter().take(10) {
            let warning = if dist > &50.0 { " ⚠️  EXCEEDS 50m THRESHOLD!" } else { "" };
            println!("   {} - {:.2}m{}", addr, dist, warning);
        }
        
        // Check if any exceed threshold
        let exceeds_threshold = sorted_by_distance.iter().any(|(_, dist)| dist > &50.0);
        if exceeds_threshold {
            println!("\n⚠️  ERROR: Some matches exceed 50m threshold! Algorithm bug detected.");
        } else {
            println!("\n✅ Threshold verification: All matches are within 50m");
        }
    }
    
    Ok(())
}

/// Correlate all addresses with progress bar updates
fn correlate_with_progress<A: CorrelationAlgo + Sync>(
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    algo: &A,
    pb: &ProgressBar,
) -> Vec<(String, f64)> {
    let counter = Arc::new(AtomicUsize::new(0));
    
    let results: Vec<_> = addresses.par_iter()
        .filter_map(|addr| {
            let result = algo.correlate(addr, zones)
                .map(|(_, dist)| (addr.adress.clone(), dist));
            
            // Update progress
            let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
            if count % 10 == 0 || count == addresses.len() {
                pb.set_position(count as u64);
            }
            
            result
        })
        .collect();
    
    pb.set_position(addresses.len() as u64);
    results
}

fn run_benchmark(
    _miljo_file: Option<String>,
    _addresses_file: Option<String>,
    sample_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")?
    );
    pb.set_message("Loading data for benchmarking...");
    
    let (addresses, zones) = api()?;
    pb.finish_with_message(format!("✓ Loaded {} addresses, {} zones", addresses.len(), zones.len()));
    
    let benchmarker = Benchmarker::new(addresses, zones);
    
    println!("\n🏁 Benchmarking all 6 algorithms with {} samples\n", sample_size);
    
    // Create multi-progress for all algorithms
    let multi_pb = MultiProgress::new();
    
    let algorithms = vec![
        "Distance-Based",
        "Raycasting",
        "Overlapping Chunks",
        "R-Tree",
        "KD-Tree",
        "Grid",
    ];
    
    // Create progress bars for each algorithm
    let pbs: Vec<_> = algorithms.iter().map(|name| {
        let pb = multi_pb.add(ProgressBar::new(sample_size as u64));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{spinner:.green}} [{:20}] [{{bar:30.cyan/blue}}] {{pos}}/{{len}} {{msg}}", name))
                .unwrap()
                .progress_chars("█▓▒░ ")
        );
        pb.set_message("waiting...");
        pb
    }).collect();
    
    // Run benchmarks with progress updates
    let results = benchmark_all_with_progress(&benchmarker, Some(sample_size), &pbs);
    
    // Finish all progress bars
    for pb in pbs {
        pb.finish_and_clear();
    }
    
    println!("\n📊 Benchmark Results:\n");
    Benchmarker::print_results(&results);
    
    Ok(())
}

fn benchmark_all_with_progress(
    benchmarker: &Benchmarker,
    sample_size: Option<usize>,
    pbs: &[ProgressBar],
) -> Vec<amp_core::benchmark::BenchmarkResult> {
    use amp_core::benchmark::BenchmarkResult;
    
    let sample_size = sample_size.unwrap_or(benchmarker.addresses.len());
    let addresses_to_test = &benchmarker.addresses[..sample_size.min(benchmarker.addresses.len())];
    
    let mut results = Vec::new();
    
    let algos: Vec<(&str, fn(&Benchmarker, &[AdressClean], &ProgressBar, &AtomicUsize, &Arc<AtomicUsize>) -> ())> = vec![
        ("Distance-Based", |bm, addrs, pb, matches, counter| {
            let algo = DistanceBasedAlgo;
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, counter, "Distance-Based");
        }),
        ("Raycasting", |bm, addrs, pb, matches, counter| {
            let algo = RaycastingAlgo;
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, counter, "Raycasting");
        }),
        ("Overlapping Chunks", |bm, addrs, pb, matches, counter| {
            let algo = OverlappingChunksAlgo::new(&bm.parking_lines);
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, counter, "Overlapping Chunks");
        }),
        ("R-Tree", |bm, addrs, pb, matches, counter| {
            let algo = RTreeSpatialAlgo::new(&bm.parking_lines);
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, counter, "R-Tree");
        }),
        ("KD-Tree", |bm, addrs, pb, matches, counter| {
            let algo = KDTreeSpatialAlgo::new(&bm.parking_lines);
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, counter, "KD-Tree");
        }),
        ("Grid", |bm, addrs, pb, matches, counter| {
            let algo = GridNearestAlgo::new(&bm.parking_lines);
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, counter, "Grid");
        }),
    ];
    
    for (idx, (name, run_fn)) in algos.iter().enumerate() {
        pbs[idx].set_message("running...");
        
        let start = Instant::now();
        let matches = AtomicUsize::new(0);
        let counter = Arc::new(AtomicUsize::new(0));
        
        run_fn(benchmarker, addresses_to_test, &pbs[idx], &matches, &counter);
        
        let total_duration = start.elapsed();
        let avg_per_address = total_duration / addresses_to_test.len() as u32;
        
        pbs[idx].finish_with_message(format!("✓ {:.2?}", total_duration));
        
        results.push(BenchmarkResult {
            algorithm_name: name.to_string(),
            total_duration,
            avg_per_address,
            addresses_processed: addresses_to_test.len(),
            matches_found: matches.load(Ordering::Relaxed),
        });
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
        if count % 5 == 0 || count == addresses.len() {
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
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")?
    );
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
