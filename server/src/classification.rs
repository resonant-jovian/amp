use crate::ui::AlgorithmChoice;
use amp_core::api::api;
use amp_core::benchmark::Benchmarker;
use amp_core::checksum::DataChecksum;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, GridNearestAlgo, KDTreeSpatialAlgo,
    OverlappingChunksAlgo, RTreeSpatialAlgo, RaycastingAlgo,
};
use amp_core::structs::{AdressClean, CorrelationResult, MiljoeDataClean};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Standard distance cutoff used everywhere (was 50m, now 20m)
pub const STANDARD_CUTOFF_METERS: f64 = 20.0;

pub fn run_test_mode_legacy(
    algorithm: AlgorithmChoice,
    cutoff: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "[TUI] Launching browser-based test mode with {:?} (cutoff {:.1}m).\nThis will open browser windows; close them when you're done reviewing.",
        algorithm, cutoff
    );

    let (addresses, miljodata, parkering): (
        Vec<AdressClean>,
        Vec<MiljoeDataClean>,
        Vec<MiljoeDataClean>,
    ) = api()?;

    let _ = correlate_dataset_for_browser(algorithm, &addresses, &miljodata, &parkering, cutoff)?;

    Ok(())
}

pub fn run_benchmark_legacy(cutoff: f64) -> Result<(), Box<dyn std::error::Error>> {
    let sample_size = 500_usize;

    println!(
        "[TUI] Running benchmark for all algorithms (sample size {}, cutoff {:.1}m)...",
        sample_size, cutoff
    );

    let (addresses, zones) = amp_core::api::api_miljo_only()?;
    let benchmarker = Benchmarker::new(addresses, zones);

    let selected_algos = vec![
        "Distance-Based",
        "Raycasting",
        "Overlapping Chunks",
        "R-Tree",
        "KD-Tree",
        "Grid",
    ];

    let multi_pb = MultiProgress::new();
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
            pb.set_message("running...");
            pb
        })
        .collect();

    let results = benchmark_selected_with_progress(&benchmarker, sample_size, &selected_algos, &pbs);

    for pb in pbs {
        pb.finish_and_clear();
    }

    Benchmarker::print_results(&results);

    Ok(())
}

pub fn run_check_updates_legacy() -> Result<(), Box<dyn std::error::Error>> {
    let checksum_file = "checksums.json";

    println!("[TUI] Checking for data updates ({}).", checksum_file);
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
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

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

/// Correlate datasets for the browser-based visual testing harness.
fn correlate_dataset_for_browser(
    algorithm: AlgorithmChoice,
    addresses: &[AdressClean],
    miljodata: &[MiljoeDataClean],
    parkering: &[MiljoeDataClean],
    cutoff: f64,
) -> Result<Vec<CorrelationResult>, Box<dyn std::error::Error>> {
    let total = addresses.len();
    let counter = Arc::new(AtomicUsize::new(0));

    let counter1 = counter.clone();
    let miljo_results = correlate_dataset(
        &algorithm,
        addresses,
        miljodata,
        cutoff,
        &counter1,
        total,
    )?;

    let counter2 = counter.clone();
    let parkering_results = correlate_dataset(
        &algorithm,
        addresses,
        parkering,
        cutoff,
        &counter2,
        total,
    )?;

    Ok(merge_results(addresses, &miljo_results, &parkering_results))
}

/// Correlate addresses with a dataset using the specified algorithm, honoring the given cutoff.
fn correlate_dataset(
    algorithm: &AlgorithmChoice,
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    cutoff: f64,
    counter: &Arc<AtomicUsize>,
    total: usize,
) -> Result<Vec<(String, f64, String)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    match algorithm {
        AlgorithmChoice::DistanceBased => {
            let algo = DistanceBasedAlgo;
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones) {
                    if dist <= cutoff {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                }
                let cnt = counter.fetch_add(1, Ordering::Relaxed) + 1;
                if cnt % 10_000 == 0 || cnt == total {
                    let pct = (cnt as f64 / total as f64) * 100.0;
                    println!("[test] Progress: {:.1}% ({}/{})", pct, cnt, total);
                }
            }
        }
        AlgorithmChoice::Raycasting => {
            let algo = RaycastingAlgo;
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones) {
                    if dist <= cutoff {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                }
                let cnt = counter.fetch_add(1, Ordering::Relaxed) + 1;
                if cnt % 10_000 == 0 || cnt == total {
                    let pct = (cnt as f64 / total as f64) * 100.0;
                    println!("[test] Progress: {:.1}% ({}/{})", pct, cnt, total);
                }
            }
        }
        AlgorithmChoice::OverlappingChunks => {
            let algo = OverlappingChunksAlgo::new(zones);
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones) {
                    if dist <= cutoff {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                }
                let cnt = counter.fetch_add(1, Ordering::Relaxed) + 1;
                if cnt % 10_000 == 0 || cnt == total {
                    let pct = (cnt as f64 / total as f64) * 100.0;
                    println!("[test] Progress: {:.1}% ({}/{})", pct, cnt, total);
                }
            }
        }
        AlgorithmChoice::RTree => {
            let algo = RTreeSpatialAlgo::new(zones);
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones) {
                    if dist <= cutoff {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                }
                let cnt = counter.fetch_add(1, Ordering::Relaxed) + 1;
                if cnt % 10_000 == 0 || cnt == total {
                    let pct = (cnt as f64 / total as f64) * 100.0;
                    println!("[test] Progress: {:.1}% ({}/{})", pct, cnt, total);
                }
            }
        }
        AlgorithmChoice::KDTree => {
            let algo = KDTreeSpatialAlgo::new(zones);
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones) {
                    if dist <= cutoff {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                }
                let cnt = counter.fetch_add(1, Ordering::Relaxed) + 1;
                if cnt % 10_000 == 0 || cnt == total {
                    let pct = (cnt as f64 / total as f64) * 100.0;
                    println!("[test] Progress: {:.1}% ({}/{})", pct, cnt, total);
                }
            }
        }
        AlgorithmChoice::Grid => {
            let algo = GridNearestAlgo::new(zones);
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones) {
                    if dist <= cutoff {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                }
                let cnt = counter.fetch_add(1, Ordering::Relaxed) + 1;
                if cnt % 10_000 == 0 || cnt == total {
                    let pct = (cnt as f64 / total as f64) * 100.0;
                    println!("[test] Progress: {:.1}% ({}/{})", pct, cnt, total);
                }
            }
        }
    }

    Ok(results)
}

/// Merge correlation results from two datasets into the core `CorrelationResult` type.
fn merge_results(
    addresses: &[AdressClean],
    miljo_results: &[(String, f64, String)],
    parkering_results: &[(String, f64, String)],
) -> Vec<CorrelationResult> {
    use std::collections::HashMap;

    let miljo_map: HashMap<_, _> = miljo_results
        .iter()
        .map(|(addr, dist, info)| (addr.clone(), (*dist, info.clone())))
        .collect();

    let parkering_map: HashMap<_, _> = parkering_results
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
    let mut pb_idx = 0;

    let all_algos: Vec<(&str, fn(&Benchmarker, &[AdressClean], &ProgressBar, &AtomicUsize) -> ())> = vec![
        ("Distance-Based", |bm, addrs, pb, matches| {
            let algo = DistanceBasedAlgo;
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, "Distance-Based");
        }),
        ("Raycasting", |bm, addrs, pb, matches| {
            let algo = RaycastingAlgo;
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, "Raycasting");
        }),
        ("Overlapping Chunks", |bm, addrs, pb, matches| {
            let algo = OverlappingChunksAlgo::new(&bm.parking_lines);
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, "Overlapping Chunks");
        }),
        ("R-Tree", |bm, addrs, pb, matches| {
            let algo = RTreeSpatialAlgo::new(&bm.parking_lines);
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, "R-Tree");
        }),
        ("KD-Tree", |bm, addrs, pb, matches| {
            let algo = KDTreeSpatialAlgo::new(&bm.parking_lines);
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, "KD-Tree");
        }),
        ("Grid", |bm, addrs, pb, matches| {
            let algo = GridNearestAlgo::new(&bm.parking_lines);
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, "Grid");
        }),
    ];

    for (name, run_fn) in all_algos.iter() {
        if !selected_algos.contains(name) {
            continue;
        }

        pbs[pb_idx].set_message("running...");

        let start = Instant::now();
        let matches = AtomicUsize::new(0);

        run_fn(benchmarker, addresses_to_test, &pbs[pb_idx], &matches);

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
    _name: &str,
) {
    addresses.par_iter().for_each(|address| {
        if algo.correlate(address, parking_lines).is_some() {
            matches.fetch_add(1, Ordering::Relaxed);
        }

        let count = pb.position() + 1;
        if count.is_multiple_of(5) || count == addresses.len() as u64 {
            pb.set_position(count);
        }
    });

    pb.set_position(addresses.len() as u64);
}
