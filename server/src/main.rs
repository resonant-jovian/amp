//! AMP Server - Address-Parking Correlation CLI
//! Supports multiple correlation algorithms, benchmarking, testing with visual verification

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
use std::env;
use std::fs;
use std::io::{self, Write};
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

        #[arg(short, long, default_value_t = 50., help = "Distance cutoff in meters")]
        cutoff: f64,
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

type CorDat = Result<Vec<(String, f64, String)>, Box<dyn std::error::Error>>;
/// Correlate addresses with a dataset using the specified algorithm and distance cutoff
fn correlate_dataset(
    algorithm: &AlgorithmChoice,
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    cutoff: f64,
    pb: &ProgressBar,
) -> CorDat {
    let counter = Arc::new(AtomicUsize::new(0));

    let results: Vec<_> = match algorithm {
        AlgorithmChoice::DistanceBased => {
            let algo = DistanceBasedAlgo;
            addresses
                .par_iter()
                .filter_map(|addr| {
                    let (idx, dist) = algo.correlate(addr, zones)?;
                    if dist > cutoff {
                        return None;
                    }
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
                    if dist > cutoff {
                        return None;
                    }
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
                    if dist > cutoff {
                        return None;
                    }
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
                    if dist > cutoff {
                        return None;
                    }
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
                    if dist > cutoff {
                        return None;
                    }
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
                    if dist > cutoff {
                        return None;
                    }
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

fn run_correlation(
    algorithm: AlgorithmChoice,
    cutoff: f64,
) -> Result<(), Box<dyn std::error::Error>> {
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
    println!("   Distance threshold: {} meters\n", cutoff);

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
    let miljo_results = correlate_dataset(&algorithm, &addresses, &miljodata, cutoff, &pb)?;

    // Correlate with parkering
    pb.set_message("Correlating with parkering...");
    let parkering_results = correlate_dataset(&algorithm, &addresses, &parkering, cutoff, &pb)?;

    let duration = start.elapsed();
    pb.finish_with_message(format!("✓ Completed in {:.2?}", duration));

    // Merge results
    let merged = merge_results(&addresses, &miljo_results, &parkering_results);

    // Calculate statistics
    let both = merged
        .iter()
        .filter(|r: &&CorrelationResult| r.miljo_match.is_some() && r.parkering_match.is_some())
        .count();
    let miljo_only = merged
        .iter()
        .filter(|r: &&CorrelationResult| r.miljo_match.is_some() && r.parkering_match.is_none())
        .count();
    let parkering_only = merged
        .iter()
        .filter(|r: &&CorrelationResult| r.miljo_match.is_none() && r.parkering_match.is_some())
        .count();
    let no_match = merged
        .iter()
        .filter(|r: &&CorrelationResult| !r.has_match())
        .count();
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
        let mut random_results: Vec<_> = merged
            .iter()
            .filter(|r: &&CorrelationResult| r.has_match())
            .collect();
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
        let mut sorted_by_distance: Vec<_> = merged
            .iter()
            .filter(|r: &&CorrelationResult| r.has_match())
            .collect();
        sorted_by_distance.sort_by(|a: &&CorrelationResult, b: &&CorrelationResult| {
            b.closest_distance()
                .partial_cmp(&a.closest_distance())
                .unwrap()
        });

        println!(
            "\n📏 10 Addresses with Largest Distances (all should be ≤{}m):",
            cutoff as i32
        );
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
        let exceeds_threshold = sorted_by_distance.iter().any(|r: &&CorrelationResult| {
            r.closest_distance().map(|d| d > cutoff).unwrap_or(false)
        });

        if exceeds_threshold {
            println!(
                "\n⚠️  ERROR: Some matches exceed {}m threshold!",
                cutoff as i32
            );
        } else {
            println!(
                "\n✅ Threshold verification: All matches are within {}m",
                cutoff as i32
            );
        }
    }

    Ok(())
}

fn run_test_mode(
    algorithm: AlgorithmChoice,
    cutoff: f64,
    num_windows: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load data with progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Loading data for testing...");

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

    println!("\n📋 Test Mode Configuration:");
    println!("   Algorithm: {:?}", algorithm);
    println!("   Distance threshold: {} meters", cutoff);
    println!("   Browser windows to open: {}", num_windows);
    println!("   Total addresses available: {}\n", addresses.len());

    // Run correlation
    let pb = ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {percent}%")?
            .progress_chars("█▓▒░ "),
    );

    let miljo_results = correlate_dataset(&algorithm, &addresses, &miljodata, cutoff, &pb)?;
    let parkering_results = correlate_dataset(&algorithm, &addresses, &parkering, cutoff, &pb)?;
    pb.finish_with_message("✓ Correlation complete".to_string());

    let merged = merge_results(&addresses, &miljo_results, &parkering_results);

    // Filter to only matching addresses
    let matching_addresses: Vec<_> = merged
        .iter()
        .filter(|r: &&CorrelationResult| r.has_match())
        .collect();

    if matching_addresses.is_empty() {
        println!("\n❌ No matching addresses found for testing!");
        return Ok(());
    }

    println!("\n📊 Correlation Results:");
    println!("   Total matches found: {}", matching_addresses.len());

    // Determine how many windows to actually open
    let actual_windows = num_windows.min(matching_addresses.len());
    println!(
        "   Windows to open: {} (sample size from {} matches)",
        actual_windows,
        matching_addresses.len()
    );

    // Random sampling
    let mut rng = thread_rng();
    let mut sampled = matching_addresses.clone();
    sampled.shuffle(&mut rng);
    let selected: Vec<_> = sampled.iter().take(actual_windows).collect();

    println!("\n🌐 Opening {} browser windows...", actual_windows);
    println!("   Each window has 4 integrated tabs in a single page:");
    println!("   - Tab 1: Live StadsAtlas with postMessage-based search");
    println!("   - Tab 2: Step-by-step instructions");
    println!("   - Tab 3: Correlation data visualization");
    println!("   - Tab 4: Debug console with communication logs\n");

    // Open browser windows with delays to prevent overwhelming the system
    for (idx, result) in selected.iter().enumerate() {
        println!(
            "   [{}/{}] Opening window for: {}",
            idx + 1,
            actual_windows,
            result.address
        );

        if let Err(e) = open_browser_window(result, idx) {
            println!("      ⚠️  Failed to open: {}", e);
        }

        // Small delay between opening windows
        if idx < actual_windows - 1 {
            thread::sleep(Duration::from_millis(500));
        }
    }

    println!("\n✅ Test mode complete!");
    println!(
        "   Review the {} opened windows to verify correlation accuracy.",
        actual_windows
    );

    Ok(())
}

/// Get the browser executable to use on Linux
fn get_browser_executable() -> String {
    // Check BROWSER environment variable first
    if let Ok(browser) = env::var("BROWSER")
        && !browser.is_empty()
    {
        return browser;
    }

    // Try to find common browsers
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

    // Default fallback
    "firefox".to_string()
}

fn format_matches_html(result: &CorrelationResult) -> String {
    match (&result.miljo_match, &result.parkering_match) {
        (Some((dist_m, info_m)), Some((dist_p, info_p))) => {
            format!(
                "<div class=\"match\">\n    <div class=\"match-item\">\n        <strong>🌍 Miljödata</strong><br>\n        <span class=\"distance\">{:.2}m away</span><br>\n        <div class=\"info\">{}</div>\n    </div>\n</div>\n<div class=\"match\">\n    <div class=\"match-item\">\n        <strong>🅿️ Parkering</strong><br>\n        <span class=\"distance\">{:.2}m away</span><br>\n        <div class=\"info\">{}</div>\n    </div>\n</div>",
                dist_m, info_m, dist_p, info_p
            )
        }
        (Some((dist, info)), None) => {
            format!(
                "<div class=\"match\">\n    <div class=\"match-item\">\n        <strong>🌍 Miljödata</strong><br>\n        <span class=\"distance\">{:.2}m away</span><br>\n        <div class=\"info\">{}</div>\n    </div>\n</div>",
                dist, info
            )
        }
        (None, Some((dist, info))) => {
            format!(
                "<div class=\"match\">\n    <div class=\"match-item\">\n        <strong>🅿️ Parkering</strong><br>\n        <span class=\"distance\">{:.2}m away</span><br>\n        <div class=\"info\">{}</div>\n    </div>\n</div>",
                dist, info
            )
        }
        (None, None) => "<div class='no-match'>✗ No matches found</div>".to_string(),
    }
}

/// Create a single HTML page with 4 integrated tabs
/// Tab 1: StadsAtlas Live Iframe with postMessage-based address injection
/// Tab 2: Instructions
/// Tab 3: Correlation Data
/// Tab 4: Debug Console with postMessage lifecycle logs
fn create_tabbed_interface_page(address: &str, result: &CorrelationResult) -> String {
    let matches_html = format_matches_html(result);
    let address_escaped = address.replace('"', "&quot;");

    // Build HTML with string concatenation to avoid format string issues with JavaScript
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str(&format!(
        "    <title>AMP Testing Interface - {}</title>\n",
        address
    ));
    html.push_str("    <meta charset=\"UTF-8\">\n");
    html.push_str(
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
    );
    html.push_str("    <style>\n");
    html.push_str("        * { margin: 0; padding: 0; box-sizing: border-box; }\n");
    html.push_str("        html, body { height: 100%; width: 100%; }\n");
    html.push_str("        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; display: flex; flex-direction: column; }\n");
    html.push_str("        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; text-align: center; box-shadow: 0 2px 8px rgba(0,0,0,0.1); flex-shrink: 0; }\n");
    html.push_str("        .header h1 { font-size: 24px; margin-bottom: 8px; }\n");
    html.push_str(
        "        .header .address { font-size: 14px; opacity: 0.9; font-weight: 500; }\n",
    );
    html.push_str("        .tab-container { flex: 1; display: flex; flex-direction: column; max-width: 1400px; width: 100%; margin: 20px auto; background: white; border-radius: 8px; box-shadow: 0 2px 12px rgba(0,0,0,0.1); overflow: hidden; }\n");
    html.push_str("        .tab-buttons { display: flex; border-bottom: 2px solid #e0e0e0; background: #fafafa; flex-shrink: 0; }\n");
    html.push_str("        .tab-btn { flex: 1; padding: 16px 20px; background: none; border: none; cursor: pointer; font-size: 14px; font-weight: 600; color: #666; text-transform: uppercase; transition: all 0.3s ease; position: relative; }\n");
    html.push_str("        .tab-btn:hover { background: #f0f0f0; color: #667eea; }\n");
    html.push_str("        .tab-btn.active { color: #667eea; background: white; }\n");
    html.push_str("        .tab-btn.active::after { content: ''; position: absolute; bottom: -2px; left: 0; right: 0; height: 2px; background: #667eea; }\n");
    html.push_str("        .tab-content { display: none; flex: 1; overflow-y: auto; padding: 30px; }\n");
    html.push_str("        .tab-content.active { display: block; animation: fadeIn 0.3s ease; }\n");
    html.push_str("        @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }\n");
    html.push_str("        #tab1 { padding: 0; display: flex; flex-direction: column; }\n");
    html.push_str("        .iframe-wrapper { display: flex; flex-direction: column; flex: 1; height: 100%; }\n");
    html.push_str("        .iframe-container { flex: 1; display: flex; flex-direction: column; min-height: 600px; }\n");
    html.push_str("        iframe { width: 100%; height: 100%; border: none; flex: 1; }\n");
    html.push_str("        .instruction { background: #e8f5e9; padding: 15px; border-radius: 4px; margin: 15px 0; border-left: 4px solid #4caf50; }\n");
    html.push_str("        .steps { counter-reset: step-counter; margin: 20px 0; }\n");
    html.push_str("        .step { counter-increment: step-counter; margin: 15px 0; padding: 15px; background: #f9f9f9; border-radius: 4px; border-left: 3px solid #667eea; }\n");
    html.push_str("        .step::before { content: counter(step-counter); display: inline-block; background: #667eea; color: white; width: 28px; height: 28px; border-radius: 50%; text-align: center; line-height: 28px; margin-right: 12px; font-weight: bold; font-size: 14px; }\n");
    html.push_str("        .step strong { color: #667eea; }\n");
    html.push_str("        .note { color: #666; font-size: 14px; margin-top: 20px; padding-top: 20px; border-top: 1px solid #ddd; }\n");
    html.push_str("        .address-display { background: #fff3e0; padding: 15px; border-radius: 4px; margin: 15px 0; font-weight: bold; border-left: 4px solid #ff9800; }\n");
    html.push_str("        .field { margin: 20px 0; }\n");
    html.push_str("        .label { font-weight: bold; color: #666; font-size: 11px; text-transform: uppercase; margin-bottom: 8px; letter-spacing: 0.5px; }\n");
    html.push_str("        .value { color: #333; padding: 12px; background: #f9f9f9; border-radius: 4px; border-left: 3px solid #667eea; font-size: 14px; }\n");
    html.push_str("        .match { background: #e8f5e9; padding: 15px; border-radius: 4px; margin: 10px 0; border-left: 4px solid #4caf50; }\n");
    html.push_str("        .match strong { color: #2e7d32; }\n");
    html.push_str("        .no-match { background: #ffebee; padding: 15px; border-radius: 4px; border-left: 4px solid #c62828; }\n");
    html.push_str("        .match-item { margin-bottom: 10px; }\n");
    html.push_str("        .distance { color: #e67e22; font-weight: bold; font-size: 16px; }\n");
    html.push_str("        .info { color: #7f8c8d; font-size: 12px; margin-top: 8px; }\n");
    html.push_str("        h2 { color: #555; font-size: 18px; margin-top: 25px; margin-bottom: 15px; border-bottom: 2px solid #e0e0e0; padding-bottom: 10px; }\n");
    html.push_str("        .console-log { background: #1e1e1e; color: #00ff00; padding: 15px; border-radius: 4px; font-family: 'Courier New', monospace; font-size: 12px; max-height: 400px; overflow-y: auto; margin-top: 15px; border: 1px solid #444; }\n");
    html.push_str("        .console-log .error { color: #ff6b6b; }\n");
    html.push_str("        .console-log .success { color: #51cf66; }\n");
    html.push_str("        .console-log .info { color: #4dabf7; }\n");
    html.push_str("        .console-log .warning { color: #ffd43b; }\n");
    html.push_str("        .log-entry { padding: 4px 0; border-bottom: 1px solid #333; }\n");
    html.push_str("        .log-timestamp { color: #888; margin-right: 8px; }\n");
    html.push_str("        .control-button { background: #667eea; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; margin: 10px 10px 10px 0; }\n");
    html.push_str("        .control-button:hover { background: #764ba2; }\n");
    html.push_str("        .control-panel { background: #f5f5f5; padding: 15px; border-radius: 4px; margin-bottom: 15px; flex-shrink: 0; }\n");
    html.push_str("        .control-button.retry { background: #ff9800; }\n");
    html.push_str("        .control-button.retry:hover { background: #e68900; }\n");
    html.push_str("    </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    html.push_str("    <div class=\"header\">\n");
    html.push_str("        <h1>📍 AMP Correlation Testing Interface</h1>\n");
    html.push_str(&format!(
        "        <div class=\"address\">{}</div>\n",
        address
    ));
    html.push_str("    </div>\n");
    html.push_str("    <div class=\"tab-container\">\n");
    html.push_str("        <div class=\"tab-buttons\">\n");
    html.push_str("            <button class=\"tab-btn active\" onclick=\"switchTab(event, 1)\">🗺️ StadsAtlas</button>\n");
    html.push_str("            <button class=\"tab-btn\" onclick=\"switchTab(event, 2)\">📋 Instructions</button>\n");
    html.push_str(
        "            <button class=\"tab-btn\" onclick=\"switchTab(event, 3)\">📊 Data</button>\n",
    );
    html.push_str(
        "            <button class=\"tab-btn\" onclick=\"switchTab(event, 4)\">🐛 Debug</button>\n",
    );
    html.push_str("        </div>\n");
    html.push_str("        <div id=\"tab1\" class=\"tab-content active\">\n");
    html.push_str("            <div class=\"control-panel\">\n");
    html.push_str("                <button class=\"control-button\" onclick=\"injectAddress()\">🔍 Inject Address</button>\n");
    html.push_str("                <button class=\"control-button retry\" onclick=\"retryInjection()\">🔄 Retry</button>\n");
    html.push_str("                <span id=\"status-indicator\" style=\"color: #666; font-size: 14px; margin-left: 20px;\">Ready to inject</span>\n");
    html.push_str("            </div>\n");
    html.push_str("            <div class=\"iframe-wrapper\">\n");
    html.push_str("                <div class=\"iframe-container\">\n");
    html.push_str("                    <iframe id=\"stadsatlas-iframe\" src=\"https://stadsatlas.malmo.se/stadsatlas/\" sandbox=\"allow-same-origin allow-scripts allow-forms allow-popups\" title=\"StadsAtlas Map\"></iframe>\n");
    html.push_str("                </div>\n");
    html.push_str("            </div>\n");
    html.push_str("        </div>\n");
    html.push_str("        <div id=\"tab2\" class=\"tab-content\">\n");
    html.push_str("            <h1>📋 StadsAtlas Verification Instructions</h1>\n");
    html.push_str("            <div class=\"instruction\">✓ Follow these steps to verify the address in StadsAtlas (Tab 1)</div>\n");
    html.push_str(&format!(
        "            <div class=\"address-display\">{}</div>\n",
        address
    ));
    html.push_str("            <div class=\"steps\">\n");
    html.push_str("                <div class=\"step\"><strong>Click the \"Inject Address\" button</strong> at the top of the StadsAtlas tab. This will send your address to the map using secure cross-origin messaging.</div>\n");
    html.push_str("                <div class=\"step\">The address will be automatically searched in StadsAtlas. Look for it to appear in the center of the map view. The map may take 2-3 seconds to complete the search.</div>\n");
    html.push_str("                <div class=\"step\">Check the <strong>Layers panel</strong> (on the right side) and look for <strong>Miljöparkering</strong> or <strong>Parkering</strong> layers.</div>\n");
    html.push_str("                <div class=\"step\">If layers are not visible, click the \"Inject Address\" button again or use the \"Retry\" button to resend the search.</div>\n");
    html.push_str("                <div class=\"step\">Verify the parking zone information displayed on the map matches the expected correlation data shown in the Data tab.</div>\n");
    html.push_str("            </div>\n");
    html.push_str("            <div class=\"note\">💡 <strong>How it works:</strong> We use the postMessage API to securely communicate with StadsAtlas across the browser's cross-origin boundary. This is more reliable than direct DOM manipulation and respects security standards.</div>\n");
    html.push_str("            <div class=\"note\">⚠️ <strong>Note:</strong> StadsAtlas uses the Origo mapping framework. Some browser console warnings about deprecated cookies (NSC_ESNS) and missing MS-specific pseudo-elements are harmless and can be ignored.</div>\n");
    html.push_str("        </div>\n");
    html.push_str("        <div id=\"tab3\" class=\"tab-content\">\n");
    html.push_str("            <h1>📊 Correlation Result Data</h1>\n");
    html.push_str("            <div class=\"field\">\n");
    html.push_str("                <div class=\"label\">Address</div>\n");
    html.push_str(&format!(
        "                <div class=\"value\">{}</div>\n",
        result.address
    ));
    html.push_str("            </div>\n");
    html.push_str("            <div class=\"field\">\n");
    html.push_str("                <div class=\"label\">Postal Code</div>\n");
    html.push_str(&format!(
        "                <div class=\"value\">{}</div>\n",
        result.postnummer
    ));
    html.push_str("            </div>\n");
    html.push_str("            <div class=\"field\">\n");
    html.push_str("                <div class=\"label\">Dataset Source</div>\n");
    html.push_str(&format!(
        "                <div class=\"value\">{}</div>\n",
        result.dataset_source()
    ));
    html.push_str("            </div>\n");
    html.push_str("            <h2>Matched Zones</h2>\n");
    html.push_str(&matches_html);
    html.push_str("        </div>\n");
    html.push_str("        <div id=\"tab4\" class=\"tab-content\">\n");
    html.push_str("            <h1>🐛 Debug Console - postMessage Lifecycle</h1>\n");
    html.push_str("            <div class=\"note\"><strong>Status:</strong> All postMessage communication is logged below. Also check browser DevTools Console (F12) for extended logs marked with <code>[AMP]</code>.</div>\n");
    html.push_str("            <div class=\"field\">\n");
    html.push_str("                <div class=\"label\">Communication Status</div>\n");
    html.push_str(
        "                <div class=\"value\" id=\"comm-status\">Waiting for action...</div>\n",
    );
    html.push_str("            </div>\n");
    html.push_str("            <div class=\"field\">\n");
    html.push_str("                <div class=\"label\">iframe Readiness</div>\n");
    html.push_str("                <div class=\"value\" id=\"iframe-status\">Loading...</div>\n");
    html.push_str("            </div>\n");
    html.push_str("            <div class=\"field\">\n");
    html.push_str("                <div class=\"label\">Injection Attempts</div>\n");
    html.push_str("                <div class=\"value\" id=\"attempt-count\">0</div>\n");
    html.push_str("            </div>\n");
    html.push_str(
        "            <div class=\"label\" style=\"margin-top: 20px;\">Message Logs</div>\n",
    );
    html.push_str("            <div class=\"console-log\" id=\"message-logs\"></div>\n");
    html.push_str("        </div>\n");
    html.push_str("    </div>\n");
    html.push_str("    <script>\n");
    html.push_str("        const logs = [];\n");
    html.push_str("        let injectionAttempts = 0;\n");
    html.push_str(&("        const addressToInject = '".to_owned() + &address_escaped + "';\n"));
    html.push_str("        const iframeElement = document.getElementById('stadsatlas-iframe');\n");
    html.push('\n');
    html.push_str("        function logMessage(category, message, type = 'info') {\n");
    html.push_str("            const timestamp = new Date().toLocaleTimeString();\n");
    html.push_str("            const logEntry = {timestamp, category, message, type};\n");
    html.push_str("            logs.push(logEntry);\n");
    html.push('\n');
    html.push_str(
        "            console.log('[AMP] [' + timestamp + '] [' + category + '] ' + message);\n",
    );
    html.push('\n');
    html.push_str("            const logsDiv = document.getElementById('message-logs');\n");
    html.push_str("            if (logsDiv) {\n");
    html.push_str("                const entry = document.createElement('div');\n");
    html.push_str("                entry.className = 'log-entry ' + type;\n");
    html.push_str("                entry.innerHTML = '<span class=\"log-timestamp\">[' + timestamp + ']</span> <strong>' + category + ':</strong> ' + message;\n");
    html.push_str("                logsDiv.appendChild(entry);\n");
    html.push_str("                logsDiv.scrollTop = logsDiv.scrollHeight;\n");
    html.push_str("            }\n");
    html.push_str("        }\n");
    html.push('\n');
    html.push_str("        function updateStatus(status, statusId = 'comm-status') {\n");
    html.push_str("            const statusDiv = document.getElementById(statusId);\n");
    html.push_str("            if (statusDiv) {\n");
    html.push_str("                statusDiv.textContent = status;\n");
    html.push_str("            }\n");
    html.push_str("        }\n");
    html.push('\n');
    html.push_str("        function switchTab(event, tabNumber) {\n");
    html.push_str("            const tabs = document.querySelectorAll('.tab-content');\n");
    html.push_str("            tabs.forEach(function(tab) { tab.classList.remove('active'); });\n");
    html.push_str("            const btns = document.querySelectorAll('.tab-btn');\n");
    html.push_str("            btns.forEach(function(btn) { btn.classList.remove('active'); });\n");
    html.push_str(
        "            document.getElementById('tab' + tabNumber).classList.add('active');\n",
    );
    html.push_str("            event.target.classList.add('active');\n");
    html.push_str("        }\n");
    html.push('\n');
    html.push_str("        function injectAddress() {\n");
    html.push_str("            injectionAttempts++;\n");
    html.push_str("            updateStatus('Injection attempts: ' + injectionAttempts, 'attempt-count');\n");
    html.push_str("            logMessage('INJECT', 'Attempt #' + injectionAttempts + ': Sending postMessage to iframe with address: ' + addressToInject, 'info');\n");
    html.push_str("            updateStatus('⏳ Sending address to StadsAtlas (attempt #' + injectionAttempts + ')...');\n");
    html.push('\n');
    html.push_str("            // Ensure iframe is ready\n");
    html.push_str("            if (!iframeElement.contentWindow) {\n");
    html.push_str("                logMessage('INJECT', 'ERROR: iframe contentWindow not accessible', 'error');\n");
    html.push_str("                updateStatus('❌ iframe not ready. Please wait and try again.');\n");
    html.push_str("                return;\n");
    html.push_str("            }\n");
    html.push('\n');
    html.push_str("            // Send postMessage to the iframe with multiple attempts\n");
    html.push_str("            // StadsAtlas uses Origo which listens for custom messages\n");
    html.push_str("            iframeElement.contentWindow.postMessage({\n");
    html.push_str("                type: 'SEARCH',\n");
    html.push_str("                address: addressToInject,\n");
    html.push_str("                source: 'amp-testing'\n");
    html.push_str("            }, 'https://stadsatlas.malmo.se');\n");
    html.push('\n');
    html.push_str(
        "            logMessage('INJECT', 'postMessage sent successfully', 'success');\n",
    );
    html.push_str("            updateStatus('✅ Address sent. Check StadsAtlas map tab. Map may take 2-3 seconds to respond.');\n");
    html.push_str("        }\n");
    html.push('\n');
    html.push_str("        function retryInjection() {\n");
    html.push_str("            logMessage('RETRY', 'User clicked retry button', 'warning');\n");
    html.push_str("            // Wait a moment then send again\n");
    html.push_str("            setTimeout(function() {\n");
    html.push_str("                injectAddress();\n");
    html.push_str("            }, 500);\n");
    html.push_str("        }\n");
    html.push('\n');
    html.push_str("        // Listen for messages from the iframe\n");
    html.push_str("        window.addEventListener('message', function(event) {\n");
    html.push_str("            // Only accept messages from StadsAtlas\n");
    html.push_str("            if (event.origin !== 'https://stadsatlas.malmo.se') {\n");
    html.push_str("                logMessage('SECURITY', 'Rejected message from untrusted origin: ' + event.origin, 'error');\n");
    html.push_str("                return;\n");
    html.push_str("            }\n");
    html.push('\n');
    html.push_str("            logMessage('RESPONSE', 'Received message from StadsAtlas: ' + JSON.stringify(event.data).substring(0, 100), 'success');\n");
    html.push_str("            updateStatus('📨 Response received from StadsAtlas');\n");
    html.push_str("        });\n");
    html.push('\n');
    html.push_str("        // Track iframe loading state\n");
    html.push_str("        iframeElement.addEventListener('load', function() {\n");
    html.push_str(
        "            logMessage('INIT', 'StadsAtlas iframe loaded and ready', 'success');\n",
    );
    html.push_str("            updateStatus('✅ iframe ready. Click \"Inject Address\" to search.', 'iframe-status');\n");
    html.push_str("        });\n");
    html.push('\n');
    html.push_str("        iframeElement.addEventListener('error', function() {\n");
    html.push_str(
        "            logMessage('ERROR', 'Failed to load StadsAtlas iframe', 'error');\n",
    );
    html.push_str("            updateStatus('❌ iframe failed to load', 'iframe-status');\n");
    html.push_str("        });\n");
    html.push('\n');
    html.push_str("        // Initial status\n");
    html.push_str("        window.addEventListener('load', function() {\n");
    html.push_str("            logMessage('READY', 'AMP Testing Interface loaded. Ready to inject address into StadsAtlas.', 'info');\n");
    html.push_str("        });\n");
    html.push_str("    </script>\n");
    html.push_str("</body>\n");
    html.push_str("</html>");

    html
}

/// Open a single browser window with integrated tabbed interface
fn open_browser_window(
    result: &&CorrelationResult,
    window_idx: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = &result.address;

    // Create the complete tabbed HTML page
    let tabbed_page = create_tabbed_interface_page(address, result);

    // Write to temporary file with unique name
    let temp_dir = std::env::temp_dir();
    let filename = format!("amp_test_{}.html", window_idx);
    let temp_file = temp_dir.join(&filename);

    fs::write(&temp_file, &tabbed_page)?;

    let file_url = format!("file://{}", temp_file.display());

    // Try to open window using different methods depending on OS
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
    // Load data
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    pb.set_message("Loading data for benchmarking...");

    let (addresses, zones) = amp_core::api::api_miljo_only()?;

    // Validate sample size
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
        requested_msg
    ));

    // Let user select which algorithms to benchmark
    let selected_algos = select_algorithms();

    let benchmarker = Benchmarker::new(addresses, zones);

    println!(
        "🏁 Benchmarking {} selected algorithm(s) with {} samples (distance cutoff: {}m)\n",
        selected_algos.len(),
        actual_sample_size,
        cutoff as i32
    );

    // Create multi-progress for selected algorithms
    let multi_pb = MultiProgress::new();

    // Create progress bars for each selected algorithm
    let pbs: Vec<_> = selected_algos
        .iter()
        .map(|name| {
            let pb = multi_pb.add(ProgressBar::new(actual_sample_size as u64));
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
    let results = benchmark_selected_with_progress(
        &benchmarker,
        actual_sample_size,
        &selected_algos,
        &pbs,
        cutoff,
    );

    // Finish all progress bars
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
    fn(&Benchmarker, &[AdressClean], &ProgressBar, &AtomicUsize, &Arc<AtomicUsize>, f64) -> ();

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
            |bm, addrs, pb, matches, counter, cutoff| {
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
            },
        ),
        ("Raycasting", |bm, addrs, pb, matches, counter, cutoff| {
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
        (
            "Overlapping Chunks",
            |bm, addrs, pb, matches, counter, cutoff| {
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
            },
        ),
        ("R-Tree", |bm, addrs, pb, matches, counter, cutoff| {
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
        ("KD-Tree", |bm, addrs, pb, matches, counter, cutoff| {
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
        ("Grid", |bm, addrs, pb, matches, counter, cutoff| {
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
