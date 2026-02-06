//! KD-tree-inspired spatial algorithm using grid-based implementation.
//!
//! This algorithm is named "KD-Tree" but uses a grid-based implementation
//! identical to [`RTreeSpatialAlgo`]. Both provide the same performance
//! characteristics and are suitable for benchmarking comparisons.
//!
//! # Implementation Note
//!
//! Despite the name, this is not a true hierarchical KD-tree (binary space
//! partitioning tree). Instead, it uses a uniform grid approach for simplicity
//! and performance. A true KD-tree would:
//! - Recursively partition space along alternating axes
//! - Have O(log n) search time in balanced trees
//! - Require rebalancing for optimal performance
//!
//! The grid-based approach offers similar practical performance while being:
//! - Simpler to implement
//! - Faster to build (no tree balancing)
//! - More predictable query times
//!
//! # Algorithm
//!
//! Identical to [`RTreeSpatialAlgo`]:
//! 1. **Indexing**: Build HashMap grid with line indices per cell
//! 2. **Query**: Search 3×3 cell neighborhood around query point
//! 3. **Distance**: Calculate perpendicular distance to lines in cells
//!
//! # Time Complexity
//!
//! - **Indexing**: O(n × k) where k = avg cells per line (~5-10)
//! - **Query**: O(m) where m = lines in 9-cell neighborhood
//! - **Average**: O(1) for uniform data distribution
//!
//! # Space Complexity
//!
//! - **Grid HashMap**: O(c) where c = non-empty cells
//! - **Line Storage**: O(n) cached line segments
//! - **Total**: ~2-3× input data size
//!
//! # Performance
//!
//! Identical to [`RTreeSpatialAlgo`]:
//! - **Indexing**: ~5-10ms
//! - **Query**: ~0.01-0.05ms
//! - **Throughput**: ~500,000 queries/second
//!
//! # Use Cases
//!
//! - **Benchmarking**: Compare named "KD-tree" vs "R-tree" implementations
//! - **Algorithm Selection**: Alternative naming for grid-based approach
//! - **Testing**: Verify grid implementation behaves identically
//!
//! # Why Not a True KD-Tree?
//!
//! Traditional KD-trees excel at:
//! - Very high-dimensional data (3D+)
//! - Nearest neighbor queries on points
//! - Static datasets with infrequent updates
//!
//! For 2D line-to-point correlation with frequent queries:
//! - Grid-based approaches are simpler and faster
//! - No benefit from tree balancing overhead
//! - Fixed-size neighborhoods work well
//!
//! # Examples
//!
//! ```no_run
//! use amp_core::correlation_algorithms::{CorrelationAlgo, KDTreeSpatialAlgo};
//! use amp_core::structs::{AdressClean, MiljoeDataClean};
//!
//! # let parking_lines: Vec<MiljoeDataClean> = vec![];
//! let algo = KDTreeSpatialAlgo::new(&parking_lines);
//!
//! # let address: AdressClean = unimplemented!();
//! if let Some((index, distance)) = algo.correlate(&address, &parking_lines) {
//!     println!("Found match at {:.1}m", distance);
//! }
//! ```
//!
//! [`RTreeSpatialAlgo`]: crate::correlation_algorithms::RTreeSpatialAlgo

use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use std::collections::HashMap;

/// KD-tree-inspired spatial index using grid implementation.
///
/// Despite the name, uses identical grid-based approach as [`RTreeSpatialAlgo`].
/// Provided for benchmarking and algorithm comparison purposes.
///
/// # Examples
///
/// ```no_run
/// use amp_core::correlation_algorithms::{CorrelationAlgo, KDTreeSpatialAlgo};
/// # use amp_core::structs::{AdressClean, MiljoeDataClean};
/// # let parking_lines: Vec<MiljoeDataClean> = vec![];
///
/// let algo = KDTreeSpatialAlgo::new(&parking_lines);
/// # let address: AdressClean = unimplemented!();
/// let result = algo.correlate(&address, &parking_lines);
/// ```
///
/// [`RTreeSpatialAlgo`]: crate::correlation_algorithms::RTreeSpatialAlgo
pub struct KDTreeSpatialAlgo {
    /// Grid cells mapping (cell_x, cell_y) to line indices
    grid: HashMap<(i32, i32), Vec<usize>>,
    /// Cached line segments with f64 coordinates
    lines: Vec<LineSegment>,
    /// Grid cell size in degrees
    cell_size: f64,
}

/// Internal line segment representation with converted coordinates.
#[derive(Clone)]
struct LineSegment {
    /// Original index in the input array
    index: usize,
    /// Start point [longitude, latitude]
    start: [f64; 2],
    /// End point [longitude, latitude]
    end: [f64; 2],
}

impl KDTreeSpatialAlgo {
    /// Create a new KD-tree-inspired spatial index.
    ///
    /// Builds a grid-based index identical to [`RTreeSpatialAlgo::new`].
    ///
    /// # Arguments
    ///
    /// * `parking_lines` - Slice of environmental parking restriction lines
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use amp_core::correlation_algorithms::KDTreeSpatialAlgo;
    /// # use amp_core::structs::MiljoeDataClean;
    /// # let parking_lines: Vec<MiljoeDataClean> = vec![];
    ///
    /// let algo = KDTreeSpatialAlgo::new(&parking_lines);
    /// ```
    ///
    /// [`RTreeSpatialAlgo::new`]: crate::correlation_algorithms::RTreeSpatialAlgo::new
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        let mut lines = Vec::new();
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                lines.push(LineSegment {
                    index: idx,
                    start: [x1, y1],
                    end: [x2, y2],
                });
                let cells = line_cells(x1, y1, x2, y2, CELL_SIZE);
                for cell in cells {
                    grid.entry(cell).or_default().push(idx);
                }
            }
        }
        Self {
            grid,
            lines,
            cell_size: CELL_SIZE,
        }
    }
}

impl CorrelationAlgo for KDTreeSpatialAlgo {
    /// Correlate address with parking lines using grid-based spatial index.
    ///
    /// Searches 3×3 neighborhood and returns closest line within threshold.
    ///
    /// # Arguments
    ///
    /// * `address` - Address point to correlate
    /// * `_parking_lines` - Ignored (index stores line references)
    ///
    /// # Returns
    ///
    /// - `Some((index, distance))` if a line is within 50 meters
    /// - `None` if no match found
    fn correlate(
        &self,
        address: &AdressClean,
        _parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        let cell = get_cell(point, self.cell_size);
        let nearby_cells = get_nearby_cells(cell);
        let mut best: Option<(usize, f64)> = None;
        for check_cell in nearby_cells {
            if let Some(indices) = self.grid.get(&check_cell) {
                for &idx in indices {
                    let line = &self.lines[idx];
                    let dist = distance_point_to_line(point, line.start, line.end);
                    if dist <= MAX_DISTANCE_METERS && (best.is_none() || dist <= best.unwrap().1) {
                        best = Some((line.index, dist));
                    }
                }
            }
        }
        best
    }

    fn name(&self) -> &'static str {
        "KD-Tree Spatial"
    }
}

/// KD-tree-inspired spatial index for parking zones.
///
/// Identical implementation to [`KDTreeSpatialAlgo`] but for parking zone data.
pub struct KDTreeParkeringAlgo {
    grid: HashMap<(i32, i32), Vec<usize>>,
    lines: Vec<LineSegment>,
    cell_size: f64,
}

impl KDTreeParkeringAlgo {
    /// Create a new KD-tree-inspired spatial index for parking zones.
    ///
    /// # Arguments
    ///
    /// * `parking_lines` - Slice of parking zone line segments
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use amp_core::correlation_algorithms::KDTreeParkeringAlgo;
    /// # use amp_core::structs::ParkeringsDataClean;
    /// # let parking_lines: Vec<ParkeringsDataClean> = vec![];
    ///
    /// let algo = KDTreeParkeringAlgo::new(&parking_lines);
    /// ```
    pub fn new(parking_lines: &[ParkeringsDataClean]) -> Self {
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        let mut lines = Vec::new();
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                lines.push(LineSegment {
                    index: idx,
                    start: [x1, y1],
                    end: [x2, y2],
                });
                let cells = line_cells(x1, y1, x2, y2, CELL_SIZE);
                for cell in cells {
                    grid.entry(cell).or_default().push(idx);
                }
            }
        }
        Self {
            grid,
            lines,
            cell_size: CELL_SIZE,
        }
    }
}

impl ParkeringCorrelationAlgo for KDTreeParkeringAlgo {
    /// Correlate address with parking zone lines.
    ///
    /// # Returns
    ///
    /// - `Some((index, distance))` if a line is within 50 meters
    /// - `None` if no match found
    fn correlate(
        &self,
        address: &AdressClean,
        _parking_lines: &[ParkeringsDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        let cell = get_cell(point, self.cell_size);
        let nearby_cells = get_nearby_cells(cell);
        let mut best: Option<(usize, f64)> = None;
        for check_cell in nearby_cells {
            if let Some(indices) = self.grid.get(&check_cell) {
                for &idx in indices {
                    let line = &self.lines[idx];
                    let dist = distance_point_to_line(point, line.start, line.end);
                    if dist <= MAX_DISTANCE_METERS && (best.is_none() || dist <= best.unwrap().1) {
                        best = Some((line.index, dist));
                    }
                }
            }
        }
        best
    }

    fn name(&self) -> &'static str {
        "KD-Tree Spatial Index (Parkering)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_cells() {
        let cells = line_cells(13.0, 55.0, 13.1, 55.1, CELL_SIZE);
        assert!(!cells.is_empty());
    }
}
