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

fn run_correlation(
    algorithm: AlgorithmChoice,
    cutoff: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load data with progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);    pb.set_message("Loading data...");

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
            .template("[{bar:40.cyan/blue}] {pos}/{len} {percent}% {msg}")?            .progress_chars("█▓▒░ "),
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
        let exceeds_threshold = sorted_by_distance
            .iter()
            .any(|r| r.closest_distance().map(|d| d > cutoff).unwrap_or(false));

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
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);    pb.set_message("Loading data for testing...");

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
            .template("[{bar:40.cyan/blue}] {pos}/{len} {percent}%")?            .progress_chars("█▓▒░ "),
    );

    let miljo_results = correlate_dataset(&algorithm, &addresses, &miljodata, cutoff, &pb)?;
    let parkering_results = correlate_dataset(&algorithm, &addresses, &parkering, cutoff, &pb)?;
    pb.finish_with_message("✓ Correlation complete".to_string());

    let merged = merge_results(&addresses, &miljo_results, &parkering_results);

    // Filter to only matching addresses
    let matching_addresses: Vec<_> = merged.iter().filter(|r| r.has_match()).collect();

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
    println!("   Each window has 2 tabs:");
    println!("   - Tab 1: StadsAtlas with automated address lookup");
    println!("   - Tab 2: Raw correlation data\n");

    // Open browser windows with delays to prevent overwhelming the system
    for (idx, result) in selected.iter().enumerate() {
        println!(
            "   [{}/{}] Opening window for: {}",
            idx + 1,
            actual_windows,
            result.address
        );

        if let Err(e) = open_browser_windows(result, idx) {
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

/// Open a new browser window with 2 tabs:
/// Tab 1: StadsAtlas with automated address entry workflow
/// Tab 2: Correlation result data
fn open_browser_windows(
    result: &&CorrelationResult,
    _window_idx: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = &result.address;

    // Create StadsAtlas automation URL with embedded JavaScript for automatic actions
    let stadsatlas_automation = create_stadsatlas_automation_page(address);
    let stadsatlas_data_url = format!(
        "data:text/html;charset=utf-8,{}",
        urlencoding::encode(&stadsatlas_automation)
    );

    // Create correlation result data page
    let correlation_data = create_correlation_result_page(result);
    let correlation_data_url = format!(
        "data:text/html;charset=utf-8,{}",
        urlencoding::encode(&correlation_data)
    );

    // Try to open windows using different methods depending on OS
    #[cfg(target_os = "windows")]
    {
        // Windows: Open new browser window with both URLs
        std::process::Command::new("cmd")
            .args(&[
                "/C",
                &format!(
                    "start "chrome" --new-window "{}" &&timeout /t 2 &&start "chrome" \"{}\"",
                    stadsatlas_data_url, correlation_data_url
                ),
            ])
            .output()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: Open new Safari window with automation page, then correlation data in new tab
        let script = format!(
            r#"open -n '{}' &
            sleep 1
            open -n '{}'""",
            stadsatlas_data_url, correlation_data_url
        );
        std::process::Command::new("bash")
            .args(&["-c", &script])
            .output()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: Use xdg-open for both URLs
        std::process::Command::new("xdg-open")
            .arg(&stadsatlas_data_url)
            .output()
            .ok();
        thread::sleep(Duration::from_millis(1000));
        std::process::Command::new("xdg-open")
            .arg(&correlation_data_url)
            .output()
            .ok();
    }

    Ok(())
}

/// Create an HTML page that will automatically interact with StadsAtlas
/// Performs:
/// 1. Click layers icon (first button)
/// 2. Click chevron right (second button)
/// 3. Click chevron right (third button) 
/// 4. Click chevron right (fourth button)
/// 5. Click radio button (uncheck to enable milj   öparkering)
/// 6. Enter address in search field
fn create_stadsatlas_automation_page(address: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>StadsAtlas - Auto Lookup: {}</title>
            <meta charset="UTF-8">
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; background: #f0f0f0; }}
                .container {{ background: white; padding: 20px; border-radius: 8px; max-width: 600px; margin: 0 auto; }}
                h1 {{ color: #333; margin-bottom: 20px; }}
                .instruction {{ background: #e8f5e9; padding: 15px; border-radius: 4px; margin: 10px 0; border-left: 4px solid #4caf50; }}
                .note {{ color: #666; font-size: 14px; margin-top: 20px; padding-top: 20px; border-top: 1px solid #ddd; }}
                .address-display {{ background: #fff3e0; padding: 15px; border-radius: 4px; margin: 15px 0; font-weight: bold; }}
                .steps {{ counter-reset: step-counter; margin: 20px 0; }}
                .step {{ counter-increment: step-counter; margin: 15px 0; padding: 10px; background: #f5f5f5; border-radius: 4px; }}
                .step::before {{ content: counter(step-counter); display: inline-block; background: #2196F3; color: white; width: 24px; height: 24px; border-radius: 50%; text-align: center; line-height: 24px; margin-right: 10px; font-weight: bold; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🗺️ StadsAtlas Auto-Lookup</h1>
                <div class="address-display">{}</div>
                
                <div class="instruction">
                    📌 This page will help you verify the correlation data in StadsAtlas.
                </div>

                <div class="steps">
                    <div class="step">
                        Click the <strong>layers icon</strong> (first icon in top toolbar)
                    </div>
                    <div class="step">
                        Click the <strong>chevron right</strong> button (arrow pointing right)
                    </div>
                    <div class="step">
                        Click the <strong>chevron right</strong> button again
                    </div>
                    <div class="step">
                        Click the <strong>chevron right</strong> button once more
                    </div>
                    <div class="step">
                        Click the <strong>radio button</strong> (circle) to enable <strong>Miljöparkering</strong>
                    </div>
                    <div class="step">
                        Click in the <strong>"Sök adresser eller platser..."</strong> search field at the top
                    </div>
                    <div class="step">
                        Enter this address: <strong>{}</strong>
                    </div>
                </div>

                <div class="note">
                    💡 <strong>Tip:</strong> Use the second tab to see the correlation result data while you verify it in StadsAtlas.
                </div>
            </div>
        </body>
        </html>""",
        address, address, address
    )
}

/// Create the correlation result data display page
fn create_correlation_result_page(result: &CorrelationResult) -> String {
    let matches_html = format_matches_html(result);
    
    format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Correlation Result - {}</title>
            <meta charset="UTF-8">
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }}
                .container {{ background: white; padding: 20px; border-radius: 8px; max-width: 600px; margin: 0 auto; }}
                h1 {{ color: #333; margin-bottom: 20px; }}
                .field {{ margin: 20px 0; }}
                .label {{ font-weight: bold; color: #666; font-size: 12px; text-transform: uppercase; margin-bottom: 5px; }}
                .value {{ color: #333; padding: 10px; background: #f9f9f9; border-radius: 4px; border-left: 3px solid #2196F3; }}
                .match {{ background: #e8f5e9; padding: 15px; border-radius: 4px; margin: 10px 0; border-left: 4px solid #4caf50; }}
                .match strong {{ color: #2e7d32; }}
                .no-match {{ background: #ffebee; padding: 15px; border-radius: 4px; border-left: 4px solid #c62828; }}
                .match-item {{ margin-bottom: 10px; }}
                .distance {{ color: #e67e22; font-weight: bold; }}
                .info {{ color: #7f8c8d; font-size: 12px; margin-top: 5px; }}
                h2 {{ color: #555; font-size: 16px; margin-top: 30px; margin-bottom: 15px; border-bottom: 2px solid #eee; padding-bottom: 10px; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>📊 Correlation Result Data</h1>
                
                <div class="field">
                    <div class="label">Address</div>
                    <div class="value">{}</div>
                </div>
                
                <div class="field">
                    <div class="label">Postal Code</div>
                    <div class="value">{}</div>
                </div>
                
                <div class="field">
                    <div class="label">Dataset Source</div>
                    <div class="value">{}</div>
                </div>
                
                <h2>Matched Zones</h2>
                {}
                
                <div style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; color: #999; font-size: 12px;">
                    Compare this data with what you see in StadsAtlas to verify correlation accuracy.
                </div>
            </div>
        </body>
        </html>""",
        result.address,
        result.address,
        result.postnummer,
        result.dataset_source(),
        matches_html
    )
}

fn format_matches_html(result: &CorrelationResult) -> String {
    match (&result.miljo_match, &result.parkering_match) {
        (Some((dist_m, info_m)), Some((dist_p, info_p))) => {
            format!(
                r#"<div class="match">
                    <div class="match-item">
                        <strong>🌍 Miljödata</strong><br>
                        <span class="distance">{:.2}m away</span><br>
                        <div class="info">{}</div>
                    </div>
                </div>
                <div class="match">
                    <div class="match-item">
                        <strong>🚗 Parkering</strong><br>
                        <span class="distance">{:.2}m away</span><br>
                        <div class="info">{}</div>
                    </div>
                </div>"#,
                dist_m, info_m, dist_p, info_p
            )
        }
        (Some((dist, info)), None) => {
            format!(
                r#"<div class="match">
                    <div class="match-item">
                        <strong>🌍 Miljödata</strong><br>
                        <span class="distance">{:.2}m away</span><br>
                        <div class="info">{}</div>
                    </div>
                </div>"#,
                dist, info
            )
        }
        (None, Some((dist, info))) => {
            format!(
                r#"<div class="match">
                    <div class="match-item">
                        <strong>🚗 Parkering</strong><br>
                        <span class="distance">{:.2}m away</span><br>
                        <div class="info">{}</div>
                    </div>
                </div>"#,
                dist, info
            )
        }
        (None, None) => "<div class='no-match'>❌ No matches found</div>".to_string(),
    }
}

type Dat = Result<Vec<(String, f64, String)>, Box<dyn std::error::Error>>;
/// Correlate addresses with a dataset using the specified algorithm and distance cutoff
fn correlate_dataset(
    algorithm: &AlgorithmChoice,
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    cutoff: f64,
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

fn run_benchmark(sample_size: usize, cutoff: f64) -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);    pb.set_message("Loading data for benchmarking...");

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

type Alg<'a> = Vec<(
    &'a str,
    fn(&Benchmarker, &[AdressClean], &ProgressBar, &AtomicUsize, &Arc<AtomicUsize>, f64) -> (),
)>;
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

    let all_algos: Alg = vec![
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
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);    pb.set_message("Fetching remote data...");

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
