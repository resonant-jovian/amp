//! Common utilities and macros for correlation algorithms
//!
//! This module contains shared functions and macros used across multiple
//! correlation algorithm implementations to reduce code duplication.

// Re-export ToPrimitive so macros can use it
pub use rust_decimal::prelude::ToPrimitive;

/// Constants shared across algorithms
pub const MAX_DISTANCE_METERS: f64 = 50.0;
pub const EARTH_RADIUS_M: f64 = 6371000.0;
pub const CELL_SIZE: f64 = 0.0005;

/// Calculate distance between two points using Haversine formula
/// Returns distance in meters
///
/// # Arguments
/// * `point1` - First point as [longitude, latitude]
/// * `point2` - Second point as [longitude, latitude]
pub fn haversine_distance(point1: [f64; 2], point2: [f64; 2]) -> f64 {
    let lat1 = point1[1].to_radians();
    let lat2 = point2[1].to_radians();
    let delta_lat = (point2[1] - point1[1]).to_radians();
    let delta_lon = (point2[0] - point1[0]).to_radians();
    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_M * c
}

/// Calculate perpendicular distance from point to line segment
///
/// # Arguments
/// * `point` - Query point as [longitude, latitude]
/// * `line_start` - Start of line segment as [longitude, latitude]
/// * `line_end` - End of line segment as [longitude, latitude]
pub fn distance_point_to_line(
    point: [f64; 2],
    line_start: [f64; 2],
    line_end: [f64; 2],
) -> f64 {
    let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let point_vec = [point[0] - line_start[0], point[1] - line_start[1]];
    let line_len_sq = line_vec[0] * line_vec[0] + line_vec[1] * line_vec[1];

    if line_len_sq == 0.0 {
        return haversine_distance(point, line_start);
    }

    let t = ((point_vec[0] * line_vec[0] + point_vec[1] * line_vec[1]) / line_len_sq)
        .clamp(0.0, 1.0);

    let closest = [
        line_start[0] + t * line_vec[0],
        line_start[1] + t * line_vec[1],
    ];

    haversine_distance(point, closest)
}

/// Get all grid cells a line segment passes through using DDA algorithm
pub fn line_cells(x1: f64, y1: f64, x2: f64, y2: f64, cell_size: f64) -> Vec<(i32, i32)> {
    let mut cells = Vec::new();
    let cell_x1 = (x1 / cell_size).floor() as i32;
    let cell_y1 = (y1 / cell_size).floor() as i32;
    let cell_x2 = (x2 / cell_size).floor() as i32;
    let cell_y2 = (y2 / cell_size).floor() as i32;

    cells.push((cell_x1, cell_y1));
    cells.push((cell_x2, cell_y2));

    // Add midpoint
    let mid_x = (x1 + x2) / 2.0;
    let mid_y = (y1 + y2) / 2.0;
    let mid_cell_x = (mid_x / cell_size).floor() as i32;
    let mid_cell_y = (mid_y / cell_size).floor() as i32;
    cells.push((mid_cell_x, mid_cell_y));

    // Add intermediate points
    let dx = (cell_x2 - cell_x1).abs();
    let dy = (cell_y2 - cell_y1).abs();
    let steps = dx.max(dy).max(1);

    for i in 1..steps {
        let t = i as f64 / steps as f64;
        let x = x1 + t * (x2 - x1);
        let y = y1 + t * (y2 - y1);
        let cx = (x / cell_size).floor() as i32;
        let cy = (y / cell_size).floor() as i32;
        cells.push((cx, cy));
    }

    cells.sort_unstable();
    cells.dedup();
    cells
}

/// Get grid cell for a point
pub fn get_cell(point: [f64; 2], cell_size: f64) -> (i32, i32) {
    (
        (point[0] / cell_size).floor() as i32,
        (point[1] / cell_size).floor() as i32,
    )
}

/// Get 9 cells surrounding and including the given cell (3x3 neighborhood)
pub fn get_nearby_cells(cell: (i32, i32)) -> Vec<(i32, i32)> {
    let mut cells = Vec::with_capacity(9);
    for dx in -1..=1 {
        for dy in -1..=1 {
            cells.push((cell.0 + dx, cell.1 + dy));
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let point1 = [13.0, 55.0];
        let point2 = [13.0, 55.001];
        let dist = haversine_distance(point1, point2);
        // Distance should be approximately 111 meters (1 degree lat ≈ 111km)
        assert!((dist - 111.0).abs() < 1.0);
    }

    #[test]
    fn test_distance_point_to_line() {
        let point = [13.0, 55.0];
        let line_start = [13.0, 55.001];
        let line_end = [13.001, 55.001];
        let dist = distance_point_to_line(point, line_start, line_end);
        // Point is perpendicular to line, should be ~111m
        assert!((dist - 111.0).abs() < 10.0);
    }

    #[test]
    fn test_get_cell() {
        let cell = get_cell([13.1, 55.6], CELL_SIZE);
        assert!(cell.0 > 0);
        assert!(cell.1 > 0);
    }

    #[test]
    fn test_get_nearby_cells() {
        let cells = get_nearby_cells((10, 20));
        assert_eq!(cells.len(), 9);
        assert!(cells.contains(&(10, 20))); // Center
        assert!(cells.contains(&(9, 19)));   // Corner
        assert!(cells.contains(&(11, 21)));  // Opposite corner
    }

    #[test]
    fn test_line_cells() {
        let cells = line_cells(13.0, 55.0, 13.1, 55.1, CELL_SIZE);
        assert!(!cells.is_empty());
        // Line should pass through multiple cells
        assert!(cells.len() > 2);
    }
}
