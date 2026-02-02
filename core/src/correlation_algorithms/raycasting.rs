//! Raycasting polygon containment algorithm
//! Determines if a point lies within or on boundary of polygon
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};

pub struct RaycastingAlgo;

impl CorrelationAlgo for RaycastingAlgo {
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
                let mut min_dist = f64::MAX;

                for segment in line.coordinates.windows(2) {
                    let start = [segment[0][0].to_f64()?, segment[0][1].to_f64()?];
                    let end = [segment[1][0].to_f64()?, segment[1][1].to_f64()?];
                    let dist = distance_point_to_line(point, start, end);
                    min_dist = min_dist.min(dist);
                }

                (min_dist <= MAX_DISTANCE_METERS).then_some((idx, min_dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    fn name(&self) -> &'static str {
        "Raycasting"
    }
}

/// Raycasting algorithm for parkering data
pub struct RaycastingParkeringAlgo;

impl ParkeringCorrelationAlgo for RaycastingParkeringAlgo {
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
                let mut min_dist = f64::MAX;

                for segment in line.coordinates.windows(2) {
                    let start = [segment[0][0].to_f64()?, segment[0][1].to_f64()?];
                    let end = [segment[1][0].to_f64()?, segment[1][1].to_f64()?];
                    let dist = distance_point_to_line(point, start, end);
                    min_dist = min_dist.min(dist);
                }

                (min_dist <= MAX_DISTANCE_METERS).then_some((idx, min_dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    fn name(&self) -> &'static str {
        "Raycasting (Parkering)"
    }
}
