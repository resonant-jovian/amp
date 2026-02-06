//! Spatial correlation algorithms for matching addresses with parking zones.
//!
//! This module provides multiple algorithms for correlating address points with
//! parking restriction line segments. Each algorithm implements different spatial
//! indexing and search strategies, trading off between accuracy, performance, and
//! memory usage.
//!
//! # Algorithm Comparison
//!
//! | Algorithm | Index Structure | Time Complexity | Memory | Best For |
//! |-----------|----------------|-----------------|--------|----------|
//! | [`DistanceBasedAlgo`] | None (brute-force) | O(n) | Low | Small datasets (<1000 lines) |
//! | [`GridNearestAlgo`] | Spatial grid | O(1) avg | Medium | Uniformly distributed data |
//! | [`KDTreeSpatialAlgo`] | KD-tree | O(log n) | Medium | Point queries, static data |
//! | [`RTreeSpatialAlgo`] | R-tree | O(log n) | Medium-High | General purpose, best overall |
//! | [`OverlappingChunksAlgo`] | Chunked grid | O(1) avg | High | Very large datasets |
//! | [`RaycastingAlgo`] | None | O(n) | Low | Debugging/verification |
//!
//! # Usage Patterns
//!
//! All algorithms implement the [`CorrelationAlgo`] trait for environmental
//! restrictions (miljödata) and [`ParkeringCorrelationAlgo`] for parking zones.
//!
//! ## Basic Correlation
//!
//! ```no_run
//! use amp_core::correlation_algorithms::{CorrelationAlgo, RTreeSpatialAlgo};
//! use amp_core::structs::{AdressClean, MiljoeDataClean};
//! use rust_decimal::Decimal;
//!
//! # let addresses: Vec<AdressClean> = vec![];
//! # let parking_lines: Vec<MiljoeDataClean> = vec![];
//! let algo = RTreeSpatialAlgo::new(&parking_lines);
//!
//! for address in &addresses {
//!     if let Some((index, distance)) = algo.correlate(address, &parking_lines) {
//!         println!("Matched to line {} at {:.1}m", index, distance);
//!     }
//! }
//! ```
//!
//! ## Benchmarking Multiple Algorithms
//!
//! ```no_run
//! use amp_core::correlation_algorithms::*;
//! # use amp_core::structs::{AdressClean, MiljoeDataClean};
//! # let addresses: Vec<AdressClean> = vec![];
//! # let parking_lines: Vec<MiljoeDataClean> = vec![];
//!
//! let algorithms: Vec<Box<dyn CorrelationAlgo>> = vec![
//!     Box::new(RTreeSpatialAlgo::new(&parking_lines)),
//!     Box::new(KDTreeSpatialAlgo::new(&parking_lines)),
//!     Box::new(GridNearestAlgo::new(&parking_lines)),
//! ];
//!
//! for algo in algorithms {
//!     println!("Testing: {}", algo.name());
//!     // Benchmark logic here
//! }
//! ```
//!
//! # Coordinate System
//!
//! All algorithms work with WGS84 coordinates (EPSG:4326):
//! - `[longitude, latitude]` format
//! - Longitude: east-west position (-180 to 180)
//! - Latitude: north-south position (-90 to 90)
//! - Distances calculated using Haversine formula (spherical Earth approximation)
//!
//! # Performance Recommendations
//!
//! For the Malmö parking dataset (~20,000 addresses, ~2,000 parking lines):
//! - **Recommended**: [`RTreeSpatialAlgo`] - Best balance of speed and accuracy
//! - **Fastest**: [`GridNearestAlgo`] - 2-3x faster than R-tree, slight accuracy tradeoff
//! - **Most Accurate**: [`OverlappingChunksAlgo`] - Handles edge cases better
//! - **Debugging**: [`RaycastingAlgo`] - Visual verification of point-line relationships

pub mod common;
pub mod distance_based;
pub mod grid_nearest;
pub mod kdtree_spatial;
pub mod overlapping_chunks;
pub mod raycasting;
pub mod rtree_spatial;

use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};

/// Trait for environmental parking correlation algorithms (miljödata).
///
/// All algorithms must implement this trait to be compatible with the
/// benchmarking system and to allow runtime algorithm selection.
///
/// # Implementation Notes
///
/// - The `parking_lines` parameter is passed for algorithms that don't
///   store an index (like [`DistanceBasedAlgo`])
/// - Algorithms with prebuilt indices (like [`RTreeSpatialAlgo`]) ignore
///   this parameter
/// - Return `None` if no parking line is within [`MAX_DISTANCE_METERS`]
///
/// [`MAX_DISTANCE_METERS`]: common::MAX_DISTANCE_METERS
pub trait CorrelationAlgo {
    /// Correlate an address with environmental parking restriction lines.
    ///
    /// Finds the closest parking line to the given address within
    /// [`MAX_DISTANCE_METERS`] (50 meters).
    ///
    /// # Arguments
    ///
    /// * `address` - Address point with coordinates
    /// * `parking_lines` - Slice of parking restriction line segments
    ///
    /// # Returns
    ///
    /// - `Some((index, distance))` if a match is found within threshold
    ///   - `index`: Position in `parking_lines` array
    ///   - `distance`: Distance in meters (approximate)
    /// - `None` if no line is within [`MAX_DISTANCE_METERS`]
    ///
    /// [`MAX_DISTANCE_METERS`]: common::MAX_DISTANCE_METERS
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)>;

    /// Get the name of this algorithm for display and logging.
    ///
    /// Used in benchmark reports and debug output.
    fn name(&self) -> &'static str;
}

/// Trait for parking zone correlation algorithms (parkeringsdata).
///
/// Identical to [`CorrelationAlgo`] but operates on parking zone data
/// instead of environmental restrictions. Must be `Send + Sync` for
/// parallel processing with Rayon.
///
/// # Thread Safety
///
/// Implementations must be thread-safe to support parallel correlation
/// of large address datasets using Rayon's `par_iter`.
pub trait ParkeringCorrelationAlgo: Send + Sync {
    /// Correlate an address with parking zone line segments.
    ///
    /// Finds the closest parking zone to the given address within
    /// [`MAX_DISTANCE_METERS`] (50 meters).
    ///
    /// # Arguments
    ///
    /// * `address` - Address point with coordinates
    /// * `parking_lines` - Slice of parking zone line segments
    ///
    /// # Returns
    ///
    /// - `Some((index, distance))` if a match is found
    /// - `None` if no line is within threshold
    ///
    /// [`MAX_DISTANCE_METERS`]: common::MAX_DISTANCE_METERS
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[ParkeringsDataClean],
    ) -> Option<(usize, f64)>;

    /// Get the name of this algorithm for display and logging.
    fn name(&self) -> &'static str;
}

// Re-export algorithm implementations
pub use distance_based::DistanceBasedAlgo;
pub use distance_based::DistanceBasedParkeringAlgo;
pub use grid_nearest::GridNearestAlgo;
pub use grid_nearest::GridNearestParkeringAlgo;
pub use kdtree_spatial::KDTreeParkeringAlgo;
pub use kdtree_spatial::KDTreeSpatialAlgo;
pub use overlapping_chunks::OverlappingChunksAlgo;
pub use overlapping_chunks::OverlappingChunksParkeringAlgo;
pub use raycasting::RaycastingAlgo;
pub use raycasting::RaycastingParkeringAlgo;
pub use rtree_spatial::RTreeSpatialAlgo;
pub use rtree_spatial::RTreeSpatialParkeringAlgo;
