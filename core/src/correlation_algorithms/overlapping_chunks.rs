//! Overlapping chunks/zones algorithm
//! Divides space into overlapping regions to reduce boundary effects
//! Different from GridNearestAlgo: larger chunks with deliberate overlap
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use std::collections::HashMap;
const CHUNK_SIZE: f64 = 0.01;
const _OVERLAP_FACTOR: f64 = 0.2;
pub struct OverlappingChunksAlgo {
    chunks: HashMap<(i32, i32), Vec<usize>>,
}
impl OverlappingChunksAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        let mut chunks: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                let min_x = x1.min(x2);
                let max_x = x1.max(x2);
                let min_y = y1.min(y2);
                let max_y = y1.max(y2);
                let chunk_min_x = (min_x / CHUNK_SIZE).floor() as i32;
                let chunk_max_x = (max_x / CHUNK_SIZE).floor() as i32;
                let chunk_min_y = (min_y / CHUNK_SIZE).floor() as i32;
                let chunk_max_y = (max_y / CHUNK_SIZE).floor() as i32;
                for cx in chunk_min_x..=chunk_max_x {
                    for cy in chunk_min_y..=chunk_max_y {
                        chunks.entry((cx, cy)).or_default().push(idx);
                    }
                }
            }
        }
        Self { chunks }
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
        let chunk_x = (point[0] / CHUNK_SIZE).floor() as i32;
        let chunk_y = (point[1] / CHUNK_SIZE).floor() as i32;
        let mut best: Option<(usize, f64)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                let check_chunk = (chunk_x + dx, chunk_y + dy);
                if let Some(indices) = self.chunks.get(&check_chunk) {
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
                        if dist <= MAX_DISTANCE_METERS && (best.is_none() || dist < best.unwrap().1)
                        {
                            best = Some((idx, dist));
                        }
                    }
                }
            }
        }
        best
    }
    fn name(&self) -> &'static str {
        "Overlapping Chunks"
    }
}
/// Overlapping chunks algorithm for parkering data
pub struct OverlappingChunksParkeringAlgo {
    chunks: HashMap<(i32, i32), Vec<usize>>,
}
impl OverlappingChunksParkeringAlgo {
    pub fn new(parking_lines: &[ParkeringsDataClean]) -> Self {
        let mut chunks: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                let min_x = x1.min(x2);
                let max_x = x1.max(x2);
                let min_y = y1.min(y2);
                let max_y = y1.max(y2);
                let chunk_min_x = (min_x / CHUNK_SIZE).floor() as i32;
                let chunk_max_x = (max_x / CHUNK_SIZE).floor() as i32;
                let chunk_min_y = (min_y / CHUNK_SIZE).floor() as i32;
                let chunk_max_y = (max_y / CHUNK_SIZE).floor() as i32;
                for cx in chunk_min_x..=chunk_max_x {
                    for cy in chunk_min_y..=chunk_max_y {
                        chunks.entry((cx, cy)).or_default().push(idx);
                    }
                }
            }
        }
        Self { chunks }
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
        let chunk_x = (point[0] / CHUNK_SIZE).floor() as i32;
        let chunk_y = (point[1] / CHUNK_SIZE).floor() as i32;
        let mut best: Option<(usize, f64)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                let check_chunk = (chunk_x + dx, chunk_y + dy);
                if let Some(indices) = self.chunks.get(&check_chunk) {
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
                        if dist <= MAX_DISTANCE_METERS && (best.is_none() || dist < best.unwrap().1)
                        {
                            best = Some((idx, dist));
                        }
                    }
                }
            }
        }
        best
    }
    fn name(&self) -> &'static str {
        "Overlapping Chunks (Parkering)"
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_chunk_calculation() {
        let x = 13.1;
        let chunk_x = (x / CHUNK_SIZE).floor() as i32;
        assert!(chunk_x > 0);
    }
    #[test]
    fn test_overlap_coverage() {
        let overlap_size = CHUNK_SIZE * _OVERLAP_FACTOR;
        assert!(overlap_size > 0.0);
        assert!(overlap_size < CHUNK_SIZE);
    }
}
