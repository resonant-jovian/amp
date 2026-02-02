//! Distance-based correlation algorithm
//! Uses perpendicular distance from point to line segment
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};

pub struct DistanceBasedAlgo;

impl CorrelationAlgo for DistanceBasedAlgo {
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

/// Distance-based algorithm for parkering data
pub struct DistanceBasedParkeringAlgo;

impl ParkeringCorrelationAlgo for DistanceBasedParkeringAlgo {
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
