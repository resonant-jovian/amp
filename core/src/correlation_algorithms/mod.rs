//! Correlation algorithms module
//! Provides multiple algorithms for correlating addresses with parking data

pub mod distance_based;
pub mod grid_nearest;
pub mod kdtree_spatial;
pub mod overlapping_chunks;
pub mod raycasting;
pub mod rtree_spatial;

use crate::structs::{AdressClean, MiljoeDataClean};

/// Trait for correlation algorithms
/// All algorithms must implement this trait to be compatible with the benchmarking system
pub trait CorrelationAlgo {
    /// Correlate an address with parking lines
    /// Returns (index, distance) of closest match, or None if no match found
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)>;

    /// Get the name of this algorithm for display purposes
    fn name(&self) -> &'static str;
}

pub use distance_based::DistanceBasedAlgo;
pub use grid_nearest::GridNearestAlgo;
pub use kdtree_spatial::KDTreeSpatialAlgo;
pub use overlapping_chunks::OverlappingChunksAlgo;
pub use raycasting::RaycastingAlgo;
pub use rtree_spatial::RTreeSpatialAlgo;
