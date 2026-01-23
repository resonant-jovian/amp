//! Correlation algorithms module
//! Provides multiple algorithms for correlating addresses with parking data

pub mod distance_based;
pub mod raycasting;
pub mod overlapping_chunks;
pub mod rtree_spatial;
pub mod quadtree_spatial;
pub mod kdtree_spatial;
pub mod grid_nearest;

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
pub use raycasting::RaycastingAlgo;
pub use overlapping_chunks::OverlappingChunksAlgo;
pub use rtree_spatial::RTreeSpatialAlgo;
pub use quadtree_spatial::QuadtreeSpatialAlgo;
pub use kdtree_spatial::KDTreeSpatialAlgo;
pub use grid_nearest::GridNearestAlgo;
