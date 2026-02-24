//! Geolocation and coordinate-based address matching.
//!
//! Provides a fast O(k) GPS-to-address lookup using a grid-based spatial
//! index (same approach as KDTreeSpatialAlgo in amp_core) built from the
//! ~60k address points in adresser.parquet. The grid is built once in
//! parallel with rayon and cached for the lifetime of the process.
//!
//! # Algorithm
//!
//! 1. Build: map each address point to its grid cell (rayon par_iter)
//! 2. Lookup: get cell for GPS coordinate, expand to 3×3 neighbourhood,
//!    check all address points in those cells with haversine distance
//! 3. Apply 20 m cutoff — same threshold used in the server correlation
//!
//! # Examples
//! ```no_run
//! if let Some(addr) = find_address_by_coordinates(55.5897, 13.0001) {
//!     println!("Nearest address: {} {}", addr.gata, addr.gatunummer);
//! }
//! ```
use crate::components::static_data::load_ref_data;
use amp_core::correlation_algorithms::common::{
    CELL_SIZE, get_cell, get_nearby_cells, haversine_distance,
};
use amp_core::structs::AdressClean;
use rayon::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::sync::OnceLock;
/// Cutoff for GPS-to-address-centroid matching.
///
/// Uses 100 m rather than the server's 20 m line-to-point threshold because:
/// - getLastKnownLocation() can be stale by minutes/seconds
/// - Phone GPS accuracy is typically 5–20 m (worse indoors/near buildings)
/// - The stored coordinate is the property centroid, not the street kerb
const GPS_CUTOFF_METERS: f64 = 50.0;
/// Cached grid spatial index over all address points.
static ADDRESS_GRID: OnceLock<AddressGrid> = OnceLock::new();
/// Grid-based spatial index for point-to-point nearest-neighbour queries.
///
/// Identical grid strategy to `KDTreeSpatialAlgo` / `RTreeSpatialAlgo` in
/// amp_core, but adapted for point queries instead of line queries.
struct AddressGrid {
    /// Grid cell → list of address indices
    grid: HashMap<(i32, i32), Vec<usize>>,
    /// Cached [longitude, latitude] coordinates for fast distance checks
    points: Vec<[f64; 2]>,
}
impl AddressGrid {
    /// Build the grid index in parallel with rayon.
    fn build(addresses: &[AdressClean]) -> Self {
        let points: Vec<[f64; 2]> = addresses
            .par_iter()
            .map(|addr| {
                [
                    addr.coordinates[0].to_f64().unwrap_or(0.0),
                    addr.coordinates[1].to_f64().unwrap_or(0.0),
                ]
            })
            .collect();
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (idx, &point) in points.iter().enumerate() {
            if point[0] == 0.0 && point[1] == 0.0 {
                continue;
            }
            let cell = get_cell(point, CELL_SIZE);
            grid.entry(cell).or_default().push(idx);
        }
        Self { grid, points }
    }
    /// Return the index of the nearest address within `GPS_CUTOFF_METERS`, or None.
    fn find_nearest(&self, lat: f64, lon: f64) -> Option<usize> {
        let query = [lon, lat];
        let cell = get_cell(query, CELL_SIZE);
        let nearby = get_nearby_cells(cell);
        let mut best: Option<(f64, usize)> = None;
        for check_cell in nearby {
            if let Some(indices) = self.grid.get(&check_cell) {
                for &idx in indices {
                    let dist = haversine_distance(query, self.points[idx]);
                    if dist <= GPS_CUTOFF_METERS && best.is_none_or(|(d, _)| dist < d) {
                        best = Some((dist, idx));
                    }
                }
            }
        }
        best.map(|(_, idx)| idx)
    }
}
fn get_address_grid() -> &'static AddressGrid {
    ADDRESS_GRID.get_or_init(|| {
        let addresses = load_ref_data();
        AddressGrid::build(addresses)
    })
}
/// Find the nearest address to the given GPS coordinates.
///
/// Loads the full address reference dataset (adresser.parquet, ~60k entries),
/// builds a grid spatial index on first call (cached thereafter), then
/// does an O(k) neighbourhood lookup for the query point.
///
/// # Arguments
/// * `lat` - Latitude in decimal degrees (WGS84)
/// * `lon` - Longitude in decimal degrees (WGS84)
///
/// # Returns
/// `Some(AdressClean)` if an address is found within 20 m, `None` otherwise.
pub fn find_address_by_coordinates(lat: f64, lon: f64) -> Option<AdressClean> {
    eprintln!("[Geo] Looking up address at lat={}, lon={}", lat, lon);
    let addresses = load_ref_data();
    let grid = get_address_grid();
    let result = grid
        .find_nearest(lat, lon)
        .map(|idx| addresses[idx].clone());
    if result.is_some() {
        eprintln!(
            "[Geo] Found address: {:?}",
            result.as_ref().map(|a| &a.adress)
        );
    } else {
        eprintln!("[Geo] No address found within {}m", GPS_CUTOFF_METERS);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_find_address_no_crash() {
        let result = find_address_by_coordinates(0.0, 0.0);
        assert!(result.is_none());
    }
    #[test]
    fn test_address_grid_builds() {
        let grid = get_address_grid();
        let addresses = load_ref_data();
        assert!(!grid.grid.is_empty(), "Grid should have entries");
        assert_eq!(grid.points.len(), addresses.len());
    }
}
