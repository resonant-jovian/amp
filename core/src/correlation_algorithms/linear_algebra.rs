//! Linear algebra-based correlation algorithm
//! Uses vector projections and dot products for efficient distance calculation

use crate::structs::{AdressClean, MiljoeDataClean};
use crate::correlation_algorithms::CorrelationAlgo;
use rust_decimal::prelude::ToPrimitive;

pub struct LinearAlgebraAlgo;

/// Dot product of two 2D vectors
#[inline]
fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// Vector magnitude
#[inline]
fn magnitude(v: [f64; 2]) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// Normalize a vector
fn normalize(v: [f64; 2]) -> [f64; 2] {
    let mag = magnitude(v);
    if mag > 0.0 {
        [v[0] / mag, v[1] / mag]
    } else {
        [0.0, 0.0]
    }
}

impl LinearAlgebraAlgo {
    /// Calculate distance from point to line using vector projection
    fn distance_via_projection(&self, point: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> f64 {
        // Line direction vector
        let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
        
        // Vector from line start to point
        let to_point = [point[0] - line_start[0], point[1] - line_start[1]];
        
        // Project point onto line
        let line_len_sq = dot(line_vec, line_vec);
        
        if line_len_sq == 0.0 {
            // Degenerate line (point)
            return magnitude(to_point);
        }
        
        // Parameter t represents position along line [0, 1]
        let t = (dot(to_point, line_vec) / line_len_sq).max(0.0).min(1.0);
        
        // Closest point on line segment
        let closest = [
            line_start[0] + t * line_vec[0],
            line_start[1] + t * line_vec[1],
        ];
        
        // Distance from point to closest point
        let diff = [point[0] - closest[0], point[1] - closest[1]];
        magnitude(diff)
    }
}

impl CorrelationAlgo for LinearAlgebraAlgo {
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
                
                let dist = self.distance_via_projection(point, line_start, line_end);
                Some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    
    fn name(&self) -> &'static str {
        "Linear Algebra"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product() {
        assert_eq!(dot([1.0, 0.0], [0.0, 1.0]), 0.0);
        assert_eq!(dot([1.0, 0.0], [1.0, 0.0]), 1.0);
        assert_eq!(dot([3.0, 4.0], [1.0, 1.0]), 7.0);
    }
    
    #[test]
    fn test_magnitude() {
        assert_eq!(magnitude([3.0, 4.0]), 5.0);
        assert_eq!(magnitude([0.0, 0.0]), 0.0);
    }
    
    #[test]
    fn test_normalize() {
        let v = normalize([3.0, 4.0]);
        assert!((v[0] - 0.6).abs() < 0.001);
        assert!((v[1] - 0.8).abs() < 0.001);
    }
    
    #[test]
    fn test_distance_projection() {
        let algo = LinearAlgebraAlgo;
        let dist = algo.distance_via_projection([0.0, 1.0], [0.0, 0.0], [10.0, 0.0]);
        assert!((dist - 1.0).abs() < 0.001);
    }
}
