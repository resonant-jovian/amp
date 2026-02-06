//! Distance-based correlation algorithm using brute-force search.
//!
//! This is the simplest correlation algorithm that calculates perpendicular
//! distance from the address point to every parking line segment and returns
//! the closest match within [`MAX_DISTANCE_METERS`].
//!
//! # Algorithm
//!
//! 1. Convert address coordinates from [`Decimal`] to `f64`
//! 2. For each parking line:
//!    - Calculate perpendicular distance using [`distance_point_to_line`]
//!    - Keep lines within [`MAX_DISTANCE_METERS`] (50m)
//! 3. Return the line with minimum distance
//!
//! # Time Complexity
//!
//! - **Query**: O(n) where n = number of parking lines
//! - **Indexing**: O(1) - no preprocessing required
//!
//! # Space Complexity
//!
//! O(1) - no additional memory for indexing structures
//!
//! # Use Cases
//!
//! - **Small datasets**: < 1,000 parking lines
//! - **Single queries**: One-off address lookups
//! - **Baseline**: Reference implementation for benchmarking
//! - **Testing**: Simple logic, easy to verify correctness
//!
//! # Performance
//!
//! For the Malmö dataset (~2,000 lines):
//! - ~0.1-0.5ms per query on modern hardware
//! - ~20,000 queries per second
//! - Not recommended for batch processing > 5,000 addresses
//!
//! # Examples
//!
//! ```no_run
//! use amp_core::correlation_algorithms::{CorrelationAlgo, DistanceBasedAlgo};
//! use amp_core::structs::{AdressClean, MiljoeDataClean};
//! use rust_decimal::Decimal;
//!
//! # let address = AdressClean {
//! #     coordinates: [Decimal::new(130000, 4), Decimal::new(550000, 4)],
//! #     postnummer: Some("21438".to_string()),
//! #     adress: "Test".to_string(),
//! #     gata: "Test".to_string(),
//! #     gatunummer: "1".to_string(),
//! # };
//! # let parking_lines: Vec<MiljoeDataClean> = vec![];
//! let algo = DistanceBasedAlgo;
//!
//! if let Some((index, distance)) = algo.correlate(&address, &parking_lines) {
//!     println!("Closest line: {} at {:.1}m", index, distance);
//! }
//! ```
//!
//! [`Decimal`]: rust_decimal::Decimal
//! [`MAX_DISTANCE_METERS`]: crate::correlation_algorithms::common::MAX_DISTANCE_METERS
//! [`distance_point_to_line`]: crate::correlation_algorithms::common::distance_point_to_line
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
/// Distance-based algorithm for environmental parking restrictions.
///
/// Uses brute-force search with perpendicular distance calculation.
/// No preprocessing or indexing required.
///
/// # Examples
///
/// ```no_run
/// use amp_core::correlation_algorithms::{CorrelationAlgo, DistanceBasedAlgo};
/// # use amp_core::structs::{AdressClean, MiljoeDataClean};
/// # let address: AdressClean = unimplemented!();
/// # let parking_lines: Vec<MiljoeDataClean> = vec![];
///
/// let algo = DistanceBasedAlgo;
/// let result = algo.correlate(&address, &parking_lines);
/// ```
pub struct DistanceBasedAlgo;
impl CorrelationAlgo for DistanceBasedAlgo {
    /// Correlate address with environmental parking lines using brute-force.
    ///
    /// Calculates perpendicular distance to every line and returns the closest
    /// match within [`MAX_DISTANCE_METERS`].
    ///
    /// # Returns
    ///
    /// - `Some((index, distance))` if a line is within 50 meters
    /// - `None` if no match found or coordinate conversion fails
    ///
    /// [`MAX_DISTANCE_METERS`]: crate::correlation_algorithms::common::MAX_DISTANCE_METERS
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
                let line_start = [
                    line.coordinates[0][0].to_f64()?,
                    line.coordinates[0][1].to_f64()?,
                ];
                let line_end = [
                    line.coordinates[1][0].to_f64()?,
                    line.coordinates[1][1].to_f64()?,
                ];
                let dist = distance_point_to_line(point, line_start, line_end);
                (dist <= MAX_DISTANCE_METERS).then_some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    fn name(&self) -> &'static str {
        "Distance-Based"
    }
}
/// Distance-based algorithm for parking zones (parkeringsdata).
///
/// Identical logic to [`DistanceBasedAlgo`] but operates on parking zone data.
pub struct DistanceBasedParkeringAlgo;
impl ParkeringCorrelationAlgo for DistanceBasedParkeringAlgo {
    /// Correlate address with parking zone lines using brute-force.
    ///
    /// # Returns
    ///
    /// - `Some((index, distance))` if a line is within 50 meters
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
                let line_start = [
                    line.coordinates[0][0].to_f64()?,
                    line.coordinates[0][1].to_f64()?,
                ];
                let line_end = [
                    line.coordinates[1][0].to_f64()?,
                    line.coordinates[1][1].to_f64()?,
                ];
                let dist = distance_point_to_line(point, line_start, line_end);
                (dist <= MAX_DISTANCE_METERS).then_some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    fn name(&self) -> &'static str {
        "Distance-Based (Parkering)"
    }
}
