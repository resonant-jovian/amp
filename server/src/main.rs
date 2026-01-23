//! AMP Server - Address-Parking Correlation CLI
//! Supports multiple correlation algorithms, benchmarking, and checksum verification

use amp_core::api::api;
use amp_core::structs::{AdressClean, MiljoeDataClean};
use amp_core::correlation_algorithms::{
    DistanceBasedAlgo, RaycastingAlgo, OverlappingChunksAlgo, LinearAlgebraAlgo, CorrelationAlgo
};
use amp_core::checksum::DataChecksum;
use amp_core::benchmark::Benchmarker;
use clap::{Parser, Subcommand};

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
        #[arg(short, long, value_enum, default_value_t = AlgorithmChoice::DistanceBased)]
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
    #[value(name = "linear-algebra")]
    LinearAlgebra,
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
            // Only checksum checking needs async
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
    // Load data synchronously
    println!("Loading data...");
    let (addresses, zones): (Vec<AdressClean>, Vec<MiljoeDataClean>) = api()?;
    println!("Loaded {} addresses, {} zones", addresses.len(), zones.len());
    
    // Select and run algorithm
    println!("Running correlation with {:?} algorithm...", algorithm);
    let results = match algorithm {
        AlgorithmChoice::DistanceBased => {
            let algo = DistanceBasedAlgo;
            correlate_all(&addresses, &zones, &algo)
        }
        AlgorithmChoice::Raycasting => {
            let algo = RaycastingAlgo;
            correlate_all(&addresses, &zones, &algo)
        }
        AlgorithmChoice::OverlappingChunks => {
            let algo = OverlappingChunksAlgo::new(&zones);
            correlate_all(&addresses, &zones, &algo)
        }
        AlgorithmChoice::LinearAlgebra => {
            let algo = LinearAlgebraAlgo;
            correlate_all(&addresses, &zones, &algo)
        }
    };
    
    println!("\nFound {} matches", results.len());
    println!("\nFirst 10 results:");
    for (addr, dist) in results.iter().take(10) {
        println!("  {} - {:.2}m", addr, dist);
    }
    
    Ok(())
}

fn correlate_all<A: CorrelationAlgo>(
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    algo: &A,
) -> Vec<(String, f64)> {
    addresses.iter()
        .filter_map(|addr| {
            algo.correlate(addr, zones)
                .map(|(_, dist)| (addr.adress.clone(), dist))
        })
        .collect()
}

fn run_benchmark(
    _miljo_file: Option<String>,
    _addresses_file: Option<String>,
    sample_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading data for benchmarking...");
    let (addresses, zones) = api()?;
    
    let benchmarker = Benchmarker::new(addresses, zones);
    
    println!("Running benchmarks with {} samples...", sample_size);
    let results = benchmarker.benchmark_all(Some(sample_size));
    
    Benchmarker::print_results(&results);
    
    Ok(())
}

async fn check_updates(checksum_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking for data updates...");
    
    // Try to load existing checksums
    let old_checksums = DataChecksum::load_from_file(checksum_file).ok();
    
    // Create new checksum with Malmö URLs
    let mut new_checksums = DataChecksum::new(
        "https://opendata.malmo.se/@fastighets-och-gatukontoret/miljoparkering/73490f00-0d71-4b17-903c-f77ab7664a53".to_string(),
        "https://opendata.malmo.se/@fastighets-och-gatukontoret/parkeringsavgifter/1a6bd68b-30ca-40a5-9d62-01e2a566982e".to_string(),
        "https://opendata.malmo.se/@stadsbyggnadskontoret/adresser/caf1cee8-9af2-4a75-8fb7-f1d7cb11daeb".to_string(),
    );
    
    // Fetch and update checksums
    println!("Fetching remote data...");
    new_checksums.update_from_remote().await?;
    
    // Check if changed
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
    
    // Save new checksums
    new_checksums.save_to_file(checksum_file)?;
    println!("✓ Checksums saved to {}", checksum_file);
    
    Ok(())
}
