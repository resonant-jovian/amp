//! Grid-based nearest neighbor algorithm
//! Simple uniform grid partitioning without overlap
//! Different from OverlappingChunks: no overlap, smaller fixed cells
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use std::collections::HashMap;
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
                let cells = line_cells(x1, y1, x2, y2, CELL_SIZE);
                for cell in cells {
                    grid.entry(cell).or_default().push(idx);
                }
            }
        }
        Self {
            grid,
            cell_size: CELL_SIZE,
        }
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
        let cell = get_cell(point, self.cell_size);
        let nearby_cells = get_nearby_cells(cell);
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
                    if dist <= MAX_DISTANCE_METERS && (best.is_none() || dist < best.unwrap().1) {
                        best = Some((idx, dist));
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
/// Grid-based nearest neighbor for parkering data
pub struct GridNearestParkeringAlgo {
    grid: HashMap<(i32, i32), Vec<usize>>,
    cell_size: f64,
}
impl GridNearestParkeringAlgo {
    pub fn new(parking_lines: &[ParkeringsDataClean]) -> Self {
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                let cells = line_cells(x1, y1, x2, y2, CELL_SIZE);
                for cell in cells {
                    grid.entry(cell).or_default().push(idx);
                }
            }
        }
        Self {
            grid,
            cell_size: CELL_SIZE,
        }
    }
}
impl ParkeringCorrelationAlgo for GridNearestParkeringAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[ParkeringsDataClean],
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
                    if dist <= MAX_DISTANCE_METERS && (best.is_none() || dist < best.unwrap().1) {
                        best = Some((idx, dist));
                    }
                }
            }
        }
        best
    }
    fn name(&self) -> &'static str {
        "Grid Nearest (Parkering)"
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_cell() {
        let cell = get_cell([13.1, 55.6], CELL_SIZE);
        assert!(cell.0 > 0);
        assert!(cell.1 > 0);
    }
}
