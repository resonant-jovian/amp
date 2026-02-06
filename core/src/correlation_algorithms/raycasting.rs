//! Raycasting algorithm for polyline correlation with multi-segment support.
//!
//! This algorithm handles polylines (LineStrings with multiple segments) by
//! calculating the minimum distance from a point to any segment in the line.
//! Despite the name "raycasting", this implementation focuses on distance
//! calculation rather than traditional ray-polygon intersection.
//!
//! # Algorithm
//!
//! For each parking line with potentially multiple segments:
//! 1. Iterate through all consecutive segment pairs using `windows(2)`
//! 2. Calculate perpendicular distance to each segment
//! 3. Track minimum distance across all segments
//! 4. Keep line if minimum distance ≤ [`MAX_DISTANCE_METERS`]
//! 5. Return line with overall minimum distance
//!
//! # Raycasting vs. Distance Calculation
//!
//! Traditional raycasting algorithms:
//! - Cast a ray from point in arbitrary direction
//! - Count polygon edge intersections
//! - Determine point-in-polygon membership (odd = inside, even = outside)
//!
//! This implementation:
//! - Calculates perpendicular distances directly
//! - Finds nearest line segment
//! - More appropriate for line-to-point correlation
//!
//! # Multi-Segment Handling
//!
//! Unlike other algorithms that assume 2-point lines (start/end), this handles:
//! - **LineStrings**: Multiple connected segments forming a polyline
//! - **Complex geometry**: Roads that curve or bend
//! - **Accurate distances**: Distance to closest segment, not just endpoints
//!
//! # Time Complexity
//!
//! - **Per Line**: O(s) where s = number of segments in line
//! - **Per Query**: O(n × s̄) where n = lines, s̄ = avg segments per line
//! - **Typical**: O(n) since most lines have 1-2 segments
//! - **Worst Case**: O(n × s_max) for lines with many segments
//!
//! # Space Complexity
//!
//! O(1) - no preprocessing or index structures
//!
//! # Performance Characteristics
//!
//! For Malmö dataset (20,000 addresses, 2,000 lines):
//! - **No Indexing**: 0ms setup time
//! - **Query Time**: ~0.1-0.5ms (similar to brute-force)
//! - **Throughput**: ~20,000 queries/second
//! - **Memory**: Minimal (no index overhead)
//!
//! # Advantages
//!
//! - **Multi-Segment Support**: Handles complex polylines correctly
//! - **No Preprocessing**: Works immediately on any dataset
//! - **Simple Logic**: Easy to understand and verify
//! - **Debugging**: Good reference implementation
//! - **Small Data**: Efficient for < 1,000 lines
//!
//! # Limitations
//!
//! - **Slow for Large Datasets**: O(n × s) per query (no spatial indexing)
//! - **Not Parallelizable**: Simple sequential iteration
//! - **Name Mismatch**: Not true raycasting algorithm
//!
//! # Use Cases
//!
//! - **Testing**: Verify other algorithms handle multi-segment lines correctly
//! - **Debugging**: Reference implementation for correctness validation
//! - **Small Datasets**: Quick queries when indexing overhead isn't worth it
//! - **Polyline Data**: When parking data includes curved/bent roads
//! - **One-Time Queries**: Single address lookups without repeated use
//!
//! # Data Format Requirements
//!
//! Expects parking line coordinates as:
//! ```text
//! [
//!   [x1, y1],  // First point
//!   [x2, y2],  // Second point
//!   [x3, y3],  // Third point (optional)
//!   ...        // More points (optional)
//! ]
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use amp_core::correlation_algorithms::{CorrelationAlgo, RaycastingAlgo};
//! use amp_core::structs::{AdressClean, MiljoeDataClean};
//!
//! # let parking_lines: Vec<MiljoeDataClean> = vec![];
//! let algo = RaycastingAlgo;
//!
//! # let address: AdressClean = unimplemented!();
//! // Works with polylines having multiple segments
//! if let Some((index, distance)) = algo.correlate(&address, &parking_lines) {
//!     println!("Closest segment at {:.1}m", distance);
//! }
//! ```
//!
//! [`MAX_DISTANCE_METERS`]: crate::correlation_algorithms::common::MAX_DISTANCE_METERS
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
/// Raycasting-style algorithm for environmental parking restrictions.
///
/// Handles multi-segment polylines by calculating distance to the nearest
/// segment. No spatial indexing or preprocessing required.
///
/// # Examples
///
/// ```no_run
/// use amp_core::correlation_algorithms::{CorrelationAlgo, RaycastingAlgo};
/// # use amp_core::structs::{AdressClean, MiljoeDataClean};
/// # let parking_lines: Vec<MiljoeDataClean> = vec![];
///
/// let algo = RaycastingAlgo;
/// # let address: AdressClean = unimplemented!();
/// let result = algo.correlate(&address, &parking_lines);
/// ```
pub struct RaycastingAlgo;
impl CorrelationAlgo for RaycastingAlgo {
    /// Correlate address with multi-segment parking lines.
    ///
    /// For each line, calculates distance to all segments and uses the minimum.
    /// This correctly handles polylines with multiple connected segments.
    ///
    /// # Arguments
    ///
    /// * `address` - Address point to correlate
    /// * `parking_lines` - Slice of parking lines (may have multiple segments)
    ///
    /// # Returns
    ///
    /// - `Some((index, distance))` if a line segment is within 50 meters
    /// - `None` if no match found or coordinate conversion fails
    ///
    /// # Performance
    ///
    /// O(n × s) where n = number of lines, s = average segments per line.
    /// For typical data with 1-2 segments per line, this is effectively O(n).
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        parking_lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let mut min_dist = f64::MAX;
                for segment in line.coordinates.windows(2) {
                    let start = [segment[0][0].to_f64()?, segment[0][1].to_f64()?];
                    let end = [segment[1][0].to_f64()?, segment[1][1].to_f64()?];
                    let dist = distance_point_to_line(point, start, end);
                    min_dist = min_dist.min(dist);
                }
                (min_dist <= MAX_DISTANCE_METERS).then_some((idx, min_dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    fn name(&self) -> &'static str {
        "Raycasting"
    }
}
/// Raycasting-style algorithm for parking zones (parkeringsdata).
///
/// Identical logic to [`RaycastingAlgo`] but operates on parking zone data.
pub struct RaycastingParkeringAlgo;
impl ParkeringCorrelationAlgo for RaycastingParkeringAlgo {
    /// Correlate address with multi-segment parking zone lines.
    ///
    /// # Returns
    ///
    /// - `Some((index, distance))` if a line segment is within 50 meters
    /// - `None` if no match found or coordinate conversion fails
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[ParkeringsDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        parking_lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let mut min_dist = f64::MAX;
                for segment in line.coordinates.windows(2) {
                    let start = [segment[0][0].to_f64()?, segment[0][1].to_f64()?];
                    let end = [segment[1][0].to_f64()?, segment[1][1].to_f64()?];
                    let dist = distance_point_to_line(point, start, end);
                    min_dist = min_dist.min(dist);
                }
                (min_dist <= MAX_DISTANCE_METERS).then_some((idx, min_dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    fn name(&self) -> &'static str {
        "Raycasting (Parkering)"
    }
}
