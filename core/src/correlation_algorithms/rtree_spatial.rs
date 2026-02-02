//! R-tree spatial index algorithm
//! Hierarchical bounding box tree for efficient spatial queries
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};

pub struct RTreeSpatialAlgo;

impl CorrelationAlgo for RTreeSpatialAlgo {
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
                let start = [line.coordinates[0][0].to_f64()?, line.coordinates[0][1].to_f64()?];
                let end = [line.coordinates[1][0].to_f64()?, line.coordinates[1][1].to_f64()?];
                let dist = distance_point_to_line(point, start, end);
                (dist <= MAX_DISTANCE_METERS).then_some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    fn name(&self) -> &'static str {
        "R-Tree Spatial"
    }
}

/// R-tree spatial index for parkering data
pub struct RTreeSpatialParkeringAlgo;

impl ParkeringCorrelationAlgo for RTreeSpatialParkeringAlgo {
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
                let start = [line.coordinates[0][0].to_f64()?, line.coordinates[0][1].to_f64()?];
                let end = [line.coordinates[1][0].to_f64()?, line.coordinates[1][1].to_f64()?];
                let dist = distance_point_to_line(point, start, end);
                (dist <= MAX_DISTANCE_METERS).then_some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    fn name(&self) -> &'static str {
        "R-Tree Spatial (Parkering)"
    }
}
