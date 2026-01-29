//! Distance-based correlation algorithm
//! Uses perpendicular distance from point to line segment
use crate::correlation_algorithms::CorrelationAlgo;
use crate::structs::{AdressClean, MiljoeDataClean};
use rust_decimal::prelude::ToPrimitive;
const MAX_DISTANCE_METERS: f64 = 50.0;
const EARTH_RADIUS_M: f64 = 6371000.0;
pub struct DistanceBasedAlgo;
impl DistanceBasedAlgo {
    fn distance_to_line(
        &self,
        point: [f64; 2],
        line_start: [f64; 2],
        line_end: [f64; 2],
    ) -> f64 {
        let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
        let point_vec = [point[0] - line_start[0], point[1] - line_start[1]];
        let line_len_sq = line_vec[0] * line_vec[0] + line_vec[1] * line_vec[1];
        if line_len_sq == 0.0 {
            return haversine_distance(point, line_start);
        }
        let t = ((point_vec[0] * line_vec[0] + point_vec[1] * line_vec[1]) / line_len_sq)
            .clamp(0.0, 1.0);
        let closest = [line_start[0] + t * line_vec[0], line_start[1] + t * line_vec[1]];
        haversine_distance(point, closest)
    }
}
impl CorrelationAlgo for DistanceBasedAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [address.coordinates[0].to_f64()?, address.coordinates[1].to_f64()?];
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
                let dist = self.distance_to_line(point, line_start, line_end);
                (dist <= MAX_DISTANCE_METERS).then_some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    fn name(&self) -> &'static str {
        "Distance-Based"
    }
}
/// Calculate distance between two points using Haversine formula
/// Returns distance in meters
fn haversine_distance(point1: [f64; 2], point2: [f64; 2]) -> f64 {
    let lat1 = point1[1].to_radians();
    let lat2 = point2[1].to_radians();
    let delta_lat = (point2[1] - point1[1]).to_radians();
    let delta_lon = (point2[0] - point1[0]).to_radians();
    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_M * c
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_haversine_distance() {
        let point1 = [13.0, 55.0];
        let point2 = [13.0, 55.001];
        let dist = haversine_distance(point1, point2);
        assert!((dist - 111.0).abs() < 1.0);
    }
}
