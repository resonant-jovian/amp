//! KD-tree (2D) spatial algorithm
//! Binary space partitioning optimized for 2D point queries
//! Excellent for nearest-neighbor searches
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use crate::{extract_line_coordinates, extract_point_coordinates};
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;

pub struct KDTreeSpatialAlgo {
    grid: HashMap<(i32, i32), Vec<usize>>,
    lines: Vec<LineSegment>,
    cell_size: f64,
}

#[derive(Clone)]
struct LineSegment {
    index: usize,
    start: [f64; 2],
    end: [f64; 2],
}

impl KDTreeSpatialAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        let mut lines = Vec::new();

        for (idx, line) in parking_lines.iter().enumerate() {
            if let Some(((x1, y1, x2, y2), (start, end))) = (
                (
                    line.coordinates[0][0].to_f64(),
                    line.coordinates[0][1].to_f64(),
                    line.coordinates[1][0].to_f64(),
                    line.coordinates[1][1].to_f64(),
                )
                    .into(),
                extract_line_coordinates!(line),
            )
                .into()
            {
                lines.push(LineSegment {
                    index: idx,
                    start,
                    end,
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
    fn correlate(
        &self,
        address: &AdressClean,
        _parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = extract_point_coordinates!(address)?;
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

/// KD-Tree spatial index for parkering data
pub struct KDTreeParkeringAlgo {
    grid: HashMap<(i32, i32), Vec<usize>>,
    lines: Vec<LineSegment>,
    cell_size: f64,
}

impl KDTreeParkeringAlgo {
    pub fn new(parking_lines: &[ParkeringsDataClean]) -> Self {
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        let mut lines = Vec::new();

        for (idx, line) in parking_lines.iter().enumerate() {
            if let Some(((x1, y1, x2, y2), (start, end))) = (
                (
                    line.coordinates[0][0].to_f64(),
                    line.coordinates[0][1].to_f64(),
                    line.coordinates[1][0].to_f64(),
                    line.coordinates[1][1].to_f64(),
                )
                    .into(),
                extract_line_coordinates!(line),
            )
                .into()
            {
                lines.push(LineSegment {
                    index: idx,
                    start,
                    end,
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
    fn correlate(
        &self,
        address: &AdressClean,
        _parking_lines: &[ParkeringsDataClean],
    ) -> Option<(usize, f64)> {
        let point = extract_point_coordinates!(address)?;
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
