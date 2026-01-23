//! Grid-based nearest neighbor algorithm
//! Simple uniform grid partitioning without overlap
//! Different from OverlappingChunks: no overlap, smaller fixed cells

use crate::correlation_algorithms::CorrelationAlgo;
use crate::structs::{AdressClean, MiljoeDataClean};
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::f64::consts::PI;

const CELL_SIZE: f64 = 0.0005; // ~50m in degrees at Malmö latitude
const MAX_DISTANCE_METERS: f64 = 50.0;
const EARTH_RADIUS_M: f64 = 6371000.0;

pub struct GridNearestAlgo {
    grid: HashMap<(i32, i32), Vec<usize>>,
    cell_size: f64,
}

impl GridNearestAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();

        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                // Get all cells this line passes through
                let cells = Self::line_cells(x1, y1, x2, y2, CELL_SIZE);

                for cell in cells {
                    grid.entry(cell).or_insert_with(Vec::new).push(idx);
                }
            }
        }

        Self {
            grid,
            cell_size: CELL_SIZE,
        }
    }

    /// Get all grid cells a line segment passes through using DDA algorithm
    fn line_cells(x1: f64, y1: f64, x2: f64, y2: f64, cell_size: f64) -> Vec<(i32, i32)> {
        let mut cells = Vec::new();

        let cell_x1 = (x1 / cell_size).floor() as i32;
        let cell_y1 = (y1 / cell_size).floor() as i32;
        let cell_x2 = (x2 / cell_size).floor() as i32;
        let cell_y2 = (y2 / cell_size).floor() as i32;

        // Simple approach: add start, end, and midpoint cells
        cells.push((cell_x1, cell_y1));
        cells.push((cell_x2, cell_y2));

        // Add cells along the line
        let mid_x = (x1 + x2) / 2.0;
        let mid_y = (y1 + y2) / 2.0;
        let mid_cell_x = (mid_x / cell_size).floor() as i32;
        let mid_cell_y = (mid_y / cell_size).floor() as i32;
        cells.push((mid_cell_x, mid_cell_y));

        // For longer lines, add more intermediate points
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

        cells.sort();
        cells.dedup();
        cells
    }

    fn get_cell(point: [f64; 2], cell_size: f64) -> (i32, i32) {
        (
            (point[0] / cell_size).floor() as i32,
            (point[1] / cell_size).floor() as i32,
        )
    }

    fn get_nearby_cells(cell: (i32, i32)) -> Vec<(i32, i32)> {
        let mut cells = Vec::with_capacity(9);
        for dx in -1..=1 {
            for dy in -1..=1 {
                cells.push((cell.0 + dx, cell.1 + dy));
            }
        }
        cells
    }
}

impl CorrelationAlgo for GridNearestAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];

        let cell = Self::get_cell(point, self.cell_size);
        let nearby_cells = Self::get_nearby_cells(cell);

        let mut best: Option<(usize, f64)> = None;

        for check_cell in nearby_cells {
            if let Some(indices) = self.grid.get(&check_cell) {
                for &idx in indices {
                    let line = &parking_lines[idx];

                    let start = [
                        line.coordinates[0][0].to_f64()?,
                        line.coordinates[0][1].to_f64()?,
                    ];
                    let end = [
                        line.coordinates[1][0].to_f64()?,
                        line.coordinates[1][1].to_f64()?,
                    ];

                    let dist = distance_point_to_line(point, start, end);

                    // Only consider if within threshold
                    if dist <= MAX_DISTANCE_METERS {
                        match best {
                            None => best = Some((idx, dist)),
                            Some((_, best_dist)) if dist < best_dist => {
                                best = Some((idx, dist));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        best
    }

    fn name(&self) -> &'static str {
        "Grid Nearest Neighbor"
    }
}

fn distance_point_to_line(point: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> f64 {
    let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let point_vec = [point[0] - line_start[0], point[1] - line_start[1]];

    let line_len_sq = line_vec[0] * line_vec[0] + line_vec[1] * line_vec[1];

    if line_len_sq == 0.0 {
        return haversine_distance(point, line_start);
    }

    let t = ((point_vec[0] * line_vec[0] + point_vec[1] * line_vec[1]) / line_len_sq)
        .max(0.0)
        .min(1.0);

    let closest = [
        line_start[0] + t * line_vec[0],
        line_start[1] + t * line_vec[1],
    ];

    haversine_distance(point, closest)
}

fn haversine_distance(point1: [f64; 2], point2: [f64; 2]) -> f64 {
    let lat1 = point1[1] * PI / 180.0;
    let lat2 = point2[1] * PI / 180.0;
    let delta_lat = (point2[1] - point1[1]) * PI / 180.0;
    let delta_lon = (point2[0] - point1[0]) * PI / 180.0;

    let a =
        (delta_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_M * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cell() {
        let cell = GridNearestAlgo::get_cell([13.1, 55.6], CELL_SIZE);
        assert!(cell.0 > 0);
        assert!(cell.1 > 0);
    }
}
