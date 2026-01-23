//! Distance-based correlation algorithm
//! Uses perpendicular distance from point to line segment

use crate::structs::{AdressClean, MiljoeDataClean};
use crate::correlation_algorithms::CorrelationAlgo;
use rust_decimal::prelude::ToPrimitive;

const MAX_DISTANCE_METERS: f64 = 50.0;

pub struct DistanceBasedAlgo;

impl DistanceBasedAlgo {
    fn distance_to_line(&self, point: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> f64 {
        let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
        let point_vec = [point[0] - line_start[0], point[1] - line_start[1]];
        
        let line_len_sq = line_vec[0] * line_vec[0] + line_vec[1] * line_vec[1];
        
        if line_len_sq == 0.0 {
            // Degenerate line (point)
            let dx = point[0] - line_start[0];
            let dy = point[1] - line_start[1];
            return (dx * dx + dy * dy).sqrt();
        }
        
        // Project point onto line
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
}

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
        
        parking_lines.iter()
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
                
                // Only include if within threshold
                if dist <= MAX_DISTANCE_METERS {
                    Some((idx, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    
    fn name(&self) -> &'static str {
        "Distance-Based"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_to_point() {
        let algo = DistanceBasedAlgo;
        let dist = algo.distance_to_line([0.0, 0.0], [5.0, 0.0], [5.0, 0.0]);
        assert!((dist - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_distance_to_line() {
        let algo = DistanceBasedAlgo;
        let dist = algo.distance_to_line([0.0, 1.0], [0.0, 0.0], [10.0, 0.0]);
        assert!((dist - 1.0).abs() < 0.001);
    }
}
