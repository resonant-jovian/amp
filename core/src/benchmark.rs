//! Performance benchmarking for correlation algorithms
use crate::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, GridNearestAlgo, KDTreeSpatialAlgo,
    OverlappingChunksAlgo, RTreeSpatialAlgo, RaycastingAlgo,
};
use crate::structs::{AdressClean, MiljoeDataClean};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
#[derive(Debug)]
pub struct BenchmarkResult {
    pub algorithm_name: String,
    pub total_duration: Duration,
    pub avg_per_address: Duration,
    pub addresses_processed: usize,
    pub matches_found: usize,
}
pub struct Benchmarker {
    pub addresses: Vec<AdressClean>,
    pub parking_lines: Vec<MiljoeDataClean>,
}
impl Benchmarker {
    pub fn new(
        addresses: Vec<AdressClean>,
        parking_lines: Vec<MiljoeDataClean>,
    ) -> Self {
        Self { addresses, parking_lines }
    }
    /// Run benchmark for a specific algorithm (parallelized)
    pub fn benchmark_algorithm<A: CorrelationAlgo + Sync>(
        &self,
        algo: &A,
        sample_size: Option<usize>,
    ) -> BenchmarkResult {
        let sample_size = sample_size.unwrap_or(self.addresses.len());
        let addresses_to_test = &self.addresses[..sample_size.min(self.addresses.len())];
        let start = Instant::now();
        let matches = AtomicUsize::new(0);
        addresses_to_test
            .par_iter()
            .for_each(|address| {
                if algo.correlate(address, &self.parking_lines).is_some() {
                    matches.fetch_add(1, Ordering::Relaxed);
                }
            });
        let total_duration = start.elapsed();
        let avg_per_address = total_duration / addresses_to_test.len() as u32;
        BenchmarkResult {
            algorithm_name: algo.name().to_string(),
            total_duration,
            avg_per_address,
            addresses_processed: addresses_to_test.len(),
            matches_found: matches.load(Ordering::Relaxed),
        }
    }
    /// Run all algorithms and compare
    pub fn benchmark_all(&self, sample_size: Option<usize>) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        let algo = DistanceBasedAlgo;
        results.push(self.benchmark_algorithm(&algo, sample_size));
        let algo = RaycastingAlgo;
        results.push(self.benchmark_algorithm(&algo, sample_size));
        let algo = OverlappingChunksAlgo::new(&self.parking_lines);
        results.push(self.benchmark_algorithm(&algo, sample_size));
        let algo = RTreeSpatialAlgo::new(&self.parking_lines);
        results.push(self.benchmark_algorithm(&algo, sample_size));
        let algo = KDTreeSpatialAlgo::new(&self.parking_lines);
        results.push(self.benchmark_algorithm(&algo, sample_size));
        let algo = GridNearestAlgo::new(&self.parking_lines);
        results.push(self.benchmark_algorithm(&algo, sample_size));
        results
    }
    /// Print benchmark results in a formatted table
    pub fn print_results(results: &[BenchmarkResult]) {
        println!(
            "\n{:<25} {:<15} {:<20} {:<15} {:<15}",
            "Algorithm",
            "Total Time",
            "Avg per Address",
            "Processed",
            "Matches",
        );
        println!("{}", "-".repeat(95));
        for result in results {
            println!(
                "{:<25} {:<15.2?} {:<20.2?} {:<15} {:<15}",
                result.algorithm_name,
                result.total_duration,
                result.avg_per_address,
                result.addresses_processed,
                result.matches_found,
            );
        }
        if let Some(fastest) = results.iter().min_by_key(|r| r.total_duration) {
            println!(
                "\n✓ Fastest: {} ({:.2?})",
                fastest.algorithm_name,
                fastest.total_duration,
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult {
            algorithm_name: "Test".to_string(),
            total_duration: Duration::from_secs(1),
            avg_per_address: Duration::from_millis(10),
            addresses_processed: 100,
            matches_found: 85,
        };
        assert_eq!(result.algorithm_name, "Test");
        assert_eq!(result.addresses_processed, 100);
    }
}
