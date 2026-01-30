//! Overlapping chunks (spatial grid) algorithm
//! Divides world into grid cells with overlap to handle edge cases
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
const CHUNK_SIZE: f64 = 0.001;
const OVERLAP: f64 = 0.0005;
const MAX_DISTANCE_METERS: f64 = 50.0;
const EARTH_RADIUS_M: f64 = 6371000.0;
pub struct OverlappingChunksAlgo {
    grid: SpatialGrid,
}
pub struct SpatialGrid {
    chunks: HashMap<(i32, i32), Vec<usize>>,
    cell_size: f64,
}
impl OverlappingChunksAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        Self {
            grid: SpatialGrid::new(parking_lines),
        }
    }
}
impl SpatialGrid {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        let mut chunks: HashMap<_, Vec<usize>> = HashMap::new();
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = (
                line.coordinates[0][0].min(line.coordinates[1][0]).to_f64(),
                line.coordinates[0][1].min(line.coordinates[1][1]).to_f64(),
                line.coordinates[0][0].max(line.coordinates[1][0]).to_f64(),
                line.coordinates[0][1].max(line.coordinates[1][1]).to_f64(),
            ) {
                let start_cell_x = ((min_x - OVERLAP) / CHUNK_SIZE).floor() as i32;
                let start_cell_y = ((min_y - OVERLAP) / CHUNK_SIZE).floor() as i32;
                let end_cell_x = ((max_x + OVERLAP) / CHUNK_SIZE).ceil() as i32;
                let end_cell_y = ((max_y + OVERLAP) / CHUNK_SIZE).ceil() as i32;
                for cx in start_cell_x..=end_cell_x {
                    for cy in start_cell_y..=end_cell_y {
                        chunks.entry((cx, cy)).or_default().push(idx);
                    }
                }
            }
        }
        SpatialGrid {
            chunks,
            cell_size: CHUNK_SIZE,
        }
    }
    pub fn query_nearby(&self, point: [f64; 2]) -> Vec<usize> {
        let cell_x = (point[0] / self.cell_size).floor() as i32;
        let cell_y = (point[1] / self.cell_size).floor() as i32;
        let mut candidates = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(indices) = self.chunks.get(&(cell_x + dx, cell_y + dy)) {
                    candidates.extend(indices.iter());
                }
            }
        }
        candidates
    }
}
impl CorrelationAlgo for OverlappingChunksAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        let candidates = self.grid.query_nearby(point);
        candidates
            .into_iter()
            .filter_map(|idx| {
                let line = &parking_lines[idx];
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
        "Overlapping Chunks"
    }
}
/// Overlapping chunks algorithm for parkering data
pub struct OverlappingChunksParkeringAlgo {
    chunks: HashMap<(i32, i32), Vec<usize>>,
    cell_size: f64,
}
impl OverlappingChunksParkeringAlgo {
    pub fn new(parking_lines: &[ParkeringsDataClean]) -> Self {
        let mut chunks: HashMap<_, Vec<usize>> = HashMap::new();
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = (
                line.coordinates[0][0].min(line.coordinates[1][0]).to_f64(),
                line.coordinates[0][1].min(line.coordinates[1][1]).to_f64(),
                line.coordinates[0][0].max(line.coordinates[1][0]).to_f64(),
                line.coordinates[0][1].max(line.coordinates[1][1]).to_f64(),
            ) {
                let start_cell_x = ((min_x - OVERLAP) / CHUNK_SIZE).floor() as i32;
                let start_cell_y = ((min_y - OVERLAP) / CHUNK_SIZE).floor() as i32;
                let end_cell_x = ((max_x + OVERLAP) / CHUNK_SIZE).ceil() as i32;
                let end_cell_y = ((max_y + OVERLAP) / CHUNK_SIZE).ceil() as i32;
                for cx in start_cell_x..=end_cell_x {
                    for cy in start_cell_y..=end_cell_y {
                        chunks.entry((cx, cy)).or_default().push(idx);
                    }
                }
            }
        }
        OverlappingChunksParkeringAlgo {
            chunks,
            cell_size: CHUNK_SIZE,
        }
    }
    pub fn query_nearby(&self, point: [f64; 2]) -> Vec<usize> {
        let cell_x = (point[0] / self.cell_size).floor() as i32;
        let cell_y = (point[1] / self.cell_size).floor() as i32;
        let mut candidates = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(indices) = self.chunks.get(&(cell_x + dx, cell_y + dy)) {
                    candidates.extend(indices.iter());
                }
            }
        }
        candidates
    }
}
impl ParkeringCorrelationAlgo for OverlappingChunksParkeringAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[ParkeringsDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        let candidates = self.query_nearby(point);
        candidates
            .into_iter()
            .filter_map(|idx| {
                let line = &parking_lines[idx];
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
        "Overlapping Chunks (Parkering)"
    }
}
/// Calculate perpendicular distance from point to line segment using Haversine
fn distance_point_to_line(point: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> f64 {
    let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let point_vec = [point[0] - line_start[0], point[1] - line_start[1]];
    let line_len_sq = line_vec[0] * line_vec[0] + line_vec[1] * line_vec[1];
    if line_len_sq == 0.0 {
        return haversine_distance(point, line_start);
    }
    let t =
        ((point_vec[0] * line_vec[0] + point_vec[1] * line_vec[1]) / line_len_sq).clamp(0.0, 1.0);
    let closest = [
        line_start[0] + t * line_vec[0],
        line_start[1] + t * line_vec[1],
    ];
    haversine_distance(point, closest)
}
fn haversine_distance(point1: [f64; 2], point2: [f64; 2]) -> f64 {
    let lat1 = point1[1].to_radians();
    let lat2 = point2[1].to_radians();
    let delta_lat = (point2[1] - point1[1]).to_radians();
    let delta_lon = (point2[0] - point1[0]).to_radians();
    let a =
        (delta_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_M * c
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_spatial_grid_cell_calculation() {
        let point = [13.15, 55.25];
        let cell_x = (point[0] / CHUNK_SIZE).floor() as i32;
        let cell_y = (point[1] / CHUNK_SIZE).floor() as i32;
        assert!(cell_x > 0);
        assert!(cell_y > 0);
    }
}
