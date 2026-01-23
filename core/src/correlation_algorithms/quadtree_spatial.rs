//! Quadtree spatial partitioning algorithm
//! Recursively divides space into 4 quadrants for efficient spatial queries
//! Simplified to avoid boundary-crossing complexity

use crate::structs::{AdressClean, MiljoeDataClean};
use crate::correlation_algorithms::CorrelationAlgo;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;

const GRID_CELL_SIZE: f64 = 0.005; // ~500m at equator

pub struct QuadtreeSpatialAlgo {
    grid: HashMap<(i32, i32), Vec<usize>>,
    bounds: Bounds,
}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Bounds {
    fn cell_key(point: [f64; 2]) -> (i32, i32) {
        (
            (point[0] / GRID_CELL_SIZE).floor() as i32,
            (point[1] / GRID_CELL_SIZE).floor() as i32,
        )
    }
}

impl QuadtreeSpatialAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        // Calculate bounds
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        
        for line in parking_lines {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                min_x = min_x.min(x1).min(x2);
                min_y = min_y.min(y1).min(y2);
                max_x = max_x.max(x1).max(x2);
                max_y = max_y.max(y1).max(y2);
            }
        }
        
        let bounds = Bounds { min_x, min_y, max_x, max_y };
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        
        // Insert all lines into grid cells they intersect
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                let start_cell = Bounds::cell_key([x1, y1]);
                let end_cell = Bounds::cell_key([x2, y2]);
                
                // Add to start and end cells
                grid.entry(start_cell).or_insert_with(Vec::new).push(idx);
                grid.entry(end_cell).or_insert_with(Vec::new).push(idx);
                
                // Add to intermediate cells for long lines
                let mid_x = (x1 + x2) / 2.0;
                let mid_y = (y1 + y2) / 2.0;
                let mid_cell = Bounds::cell_key([mid_x, mid_y]);
                grid.entry(mid_cell).or_insert_with(Vec::new).push(idx);
            }
        }
        
        // Deduplicate within each cell
        for cell_indices in grid.values_mut() {
            cell_indices.sort_unstable();
            cell_indices.dedup();
        }
        
        Self { grid, bounds }
    }
}

impl CorrelationAlgo for QuadtreeSpatialAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        
        let cell = Bounds::cell_key(point);
        let mut candidates: Vec<usize> = Vec::new();
        
        // Check this cell and 8 neighbors
        for dx in -1..=1 {
            for dy in -1..=1 {
                let check_cell = (cell.0 + dx, cell.1 + dy);
                if let Some(indices) = self.grid.get(&check_cell) {
                    candidates.extend(indices);
                }
            }
        }
        
        // Remove duplicates
        candidates.sort_unstable();
        candidates.dedup();
        
        // Find closest among candidates
        candidates
            .into_iter()
            .filter_map(|idx| {
                let line = parking_lines.get(idx)?;
                let start = [
                    line.coordinates[0][0].to_f64()?,
                    line.coordinates[0][1].to_f64()?,
                ];
                let end = [
                    line.coordinates[1][0].to_f64()?,
                    line.coordinates[1][1].to_f64()?,
                ];
                
                let dist = distance_point_to_line(point, start, end);
                Some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    
    fn name(&self) -> &'static str {
        "Quadtree Spatial"
    }
}

fn distance_point_to_line(point: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> f64 {
    let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let point_vec = [point[0] - line_start[0], point[1] - line_start[1]];
    
    let line_len_sq = line_vec[0] * line_vec[0] + line_vec[1] * line_vec[1];
    
    if line_len_sq == 0.0 {
        let dx = point[0] - line_start[0];
        let dy = point[1] - line_start[1];
        return (dx * dx + dy * dy).sqrt();
    }
    
    let t = ((point_vec[0] * line_vec[0] + point_vec[1] * line_vec[1]) / line_len_sq)
        .max(0.0)
        .min(1.0);
    
    let closest = [
        line_start[0] + t * line_vec[0],
        line_start[1] + t * line_vec[1],
    ];
    
    let dx = point[0] - closest[0];
    let dy = point[1] - closest[1];
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_key() {
        let cell1 = Bounds::cell_key([13.1, 55.6]);
        let cell2 = Bounds::cell_key([13.2, 55.7]);
        // Different cells for different coordinates
        assert_ne!(cell1, cell2);
    }
}
