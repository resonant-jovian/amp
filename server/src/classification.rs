use crate::ui::AlgorithmChoice;
use amp_core::api::api;
use amp_core::benchmark::Benchmarker;
use amp_core::checksum::DataChecksum;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, GridNearestAlgo, KDTreeSpatialAlgo, OverlappingChunksAlgo,
    RTreeSpatialAlgo, RaycastingAlgo,
};
use amp_core::structs::{AdressClean, CorrelationResult, MiljoeDataClean};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Standard distance cutoff used everywhere (was 50m, now 20m)
pub const _STANDARD_CUTOFF_METERS: f64 = 20.0;

/// Type alias for correlation result tuples: (address, distance_meters, zone_info)
#[allow(dead_code)]
type CorrelationTuple = (String, f64, String);

/// Type alias for benchmark algorithm function
type BenchmarkAlgoFn = fn(&Benchmarker, &[AdressClean], &ProgressBar, &AtomicUsize) -> ();

#[allow(dead_code)]
pub fn run_test_mode_legacy(
    _algorithm: AlgorithmChoice,
    _cutoff: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    // All output now goes through TUI, no println! here
    // This function is called from ui.rs and logs appear in the TUI details panel
    let (_addresses, _miljodata, _parkering): (
        Vec<AdressClean>,
        Vec<MiljoeDataClean>,
        Vec<MiljoeDataClean>,
    ) = api()?;

    // Results displayed in TUI, not printed to stdout
    Ok(())
}

pub fn run_benchmark_legacy(_cutoff: f64) -> Result<(), Box<dyn std::error::Error>> {
    let sample_size = 500_usize;

    // All benchmark output goes through TUI logging, not println!
    let (_addresses, _zones) = amp_core::api::api_miljo_only()?;
    let _benchmarker = Benchmarker::new(_addresses, _zones);

    // Progress and results displayed in TUI benchmark tab
    Ok(())
}

pub fn run_check_updates_legacy() -> Result<(), Box<dyn std::error::Error>> {
    let checksum_file = "checksums.json";

    // All status output goes through TUI, not println!
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // No println! - TUI displays status
        let _old_checksums = DataChecksum::load_from_file(checksum_file).ok();

        let mut new_checksums = DataChecksum::new(
            "https://opendata.malmo.se/@fastighets-och-gatukontoret/miljoparkering/73490f00-0d71-4b17-903c-f77ab7664a53".to_string(),
            "https://opendata.malmo.se/@fastighets-och-gatukontoret/parkeringsavgifter/1a6bd68b-30ca-40a5-9d62-01e2a566982e".to_string(),
            "https://opendata.malmo.se/@stadsbyggnadskontoret/adresser/caf1cee8-9af2-4a75-8fb7-f1d7cb11daeb".to_string(),
        );

        let _pb = ProgressBar::new_spinner();
        // No visible output - handled by TUI
        new_checksums.update_from_remote().await?;

        // Save checksums silently, TUI shows results
        new_checksums.save_to_file(checksum_file)?;

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

/// Correlate datasets for the browser-based visual testing harness.
/// All output goes through TUI logging, not stdout.
#[allow(dead_code)]
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
    let miljo_results =
        correlate_dataset(&algorithm, addresses, miljodata, cutoff, &counter1, total)?;

    let counter2 = counter.clone();
    let parkering_results =
        correlate_dataset(&algorithm, addresses, parkering, cutoff, &counter2, total)?;

    Ok(merge_results(addresses, &miljo_results, &parkering_results))
}

/// Correlate addresses with a dataset using the specified algorithm, honoring the given cutoff.
/// Progress is tracked internally, no println! output.
#[allow(dead_code)]
fn correlate_dataset(
    algorithm: &AlgorithmChoice,
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    cutoff: f64,
    _counter: &Arc<AtomicUsize>,
    _total: usize,
) -> Result<Vec<CorrelationTuple>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    match algorithm {
        AlgorithmChoice::DistanceBased => {
            let algo = DistanceBasedAlgo;
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones)
                    && dist <= cutoff
                {
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                    results.push((addr.adress.clone(), dist, info));
                }
            }
        }
        AlgorithmChoice::Raycasting => {
            let algo = RaycastingAlgo;
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones)
                    && dist <= cutoff
                {
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                    results.push((addr.adress.clone(), dist, info));
                }
            }
        }
        AlgorithmChoice::OverlappingChunks => {
            let algo = OverlappingChunksAlgo::new(zones);
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones)
                    && dist <= cutoff
                {
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                    results.push((addr.adress.clone(), dist, info));
                }
            }
        }
        AlgorithmChoice::RTree => {
            let algo = RTreeSpatialAlgo::new(zones);
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones)
                    && dist <= cutoff
                {
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                    results.push((addr.adress.clone(), dist, info));
                }
            }
        }
        AlgorithmChoice::KDTree => {
            let algo = KDTreeSpatialAlgo::new(zones);
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones)
                    && dist <= cutoff
                {
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                    results.push((addr.adress.clone(), dist, info));
                }
            }
        }
        AlgorithmChoice::Grid => {
            let algo = GridNearestAlgo::new(zones);
            for addr in addresses {
                if let Some((idx, dist)) = algo.correlate(addr, zones)
                    && dist <= cutoff
                {
                    let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                    results.push((addr.adress.clone(), dist, info));
                }
            }
        }
    }

    Ok(results)
}

/// Merge correlation results from two datasets into the core `CorrelationResult` type.
#[allow(dead_code)]
fn merge_results(
    addresses: &[AdressClean],
    miljo_results: &[CorrelationTuple],
    parkering_results: &[CorrelationTuple],
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

    let all_algos: Vec<(&str, BenchmarkAlgoFn)> = vec![
        ("Distance-Based", |bm, addrs, pb, matches| {
            let algo = DistanceBasedAlgo;
            run_single_benchmark(
                &algo,
                addrs,
                &bm.parking_lines,
                pb,
                matches,
                "Distance-Based",
            );
        }),
        ("Raycasting", |bm, addrs, pb, matches| {
            let algo = RaycastingAlgo;
            run_single_benchmark(&algo, addrs, &bm.parking_lines, pb, matches, "Raycasting");
        }),
        ("Overlapping Chunks", |bm, addrs, pb, matches| {
            let algo = OverlappingChunksAlgo::new(&bm.parking_lines);
            run_single_benchmark(
                &algo,
                addrs,
                &bm.parking_lines,
                pb,
                matches,
                "Overlapping Chunks",
            );
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
