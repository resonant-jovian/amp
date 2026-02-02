//! Distance-based correlation algorithm
//! Uses perpendicular distance from point to line segment
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use crate::{extract_line_coordinates, extract_point_coordinates};

pub struct DistanceBasedAlgo;

impl CorrelationAlgo for DistanceBasedAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = extract_point_coordinates!(address)?;

        parking_lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let (line_start, line_end) = extract_line_coordinates!(line)?;
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
        let point = extract_point_coordinates!(address)?;

        parking_lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let (line_start, line_end) = extract_line_coordinates!(line)?;
                let dist = distance_point_to_line(point, line_start, line_end);
                (dist <= MAX_DISTANCE_METERS).then_some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    fn name(&self) -> &'static str {
        "Distance-Based (Parkering)"
    }
}
