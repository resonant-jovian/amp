//! Raycasting correlation algorithm
//! Casts rays in 360° from address point with 50m lifetime
//! Returns closest line that intersects any ray

use crate::structs::{AdressClean, MiljoeDataClean};
use crate::correlation_algorithms::CorrelationAlgo;
use rust_decimal::prelude::ToPrimitive;
use std::f64::consts::PI;

const RAY_LIFETIME: f64 = 50.0; // meters as specified
const RAY_COUNT: usize = 36; // 10-degree increments

pub struct RaycastingAlgo;

struct Ray {
    origin: [f64; 2],
    direction: [f64; 2],
    max_distance: f64,
}

impl Ray {
    /// Check if ray intersects line segment
    /// Returns distance along ray if intersection found
    fn intersects_line(&self, line_start: [f64; 2], line_end: [f64; 2]) -> Option<f64> {
        let dx = line_end[0] - line_start[0];
        let dy = line_end[1] - line_start[1];
        
        // Determinant for parallel check
        let det = -self.direction[0] * dy + self.direction[1] * dx;
        
        if det.abs() < 1e-10 {
            return None; // Parallel or colinear
        }
        
        // Parametric intersection
        let u = ((self.origin[0] - line_start[0]) * dy - 
                 (self.origin[1] - line_start[1]) * dx) / det;
        let v = ((self.origin[0] - line_start[0]) * self.direction[1] - 
                 (self.origin[1] - line_start[1]) * self.direction[0]) / det;
        
        // Check if intersection is within ray lifetime and line segment
        if u >= 0.0 && u <= self.max_distance && v >= 0.0 && v <= 1.0 {
            Some(u)
        } else {
            None
        }
    }
}

impl CorrelationAlgo for RaycastingAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let origin = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        
        let mut closest: Option<(usize, f64)> = None;
        
        // Cast rays in all directions
        for i in 0..RAY_COUNT {
            let angle = (i as f64) * (2.0 * PI / RAY_COUNT as f64);
            let ray = Ray {
                origin,
                direction: [angle.cos(), angle.sin()],
                max_distance: RAY_LIFETIME,
            };
            
            // Check intersection with all parking lines
            for (idx, parking) in parking_lines.iter().enumerate() {
                let line_start = [
                    parking.coordinates[0][0].to_f64()?,
                    parking.coordinates[0][1].to_f64()?,
                ];
                let line_end = [
                    parking.coordinates[1][0].to_f64()?,
                    parking.coordinates[1][1].to_f64()?,
                ];
                
                if let Some(dist) = ray.intersects_line(line_start, line_end) {
                    match closest {
                        None => closest = Some((idx, dist)),
                        Some((_, current_dist)) if dist < current_dist => {
                            closest = Some((idx, dist));
                        }
                        _ => {}
                    }
                }
            }
        }
        
        closest
    }
    
    fn name(&self) -> &'static str {
        "Raycasting (50m)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_intersection_perpendicular() {
        let ray = Ray {
            origin: [0.0, 0.0],
            direction: [1.0, 0.0],
            max_distance: 10.0,
        };
        
        let result = ray.intersects_line([5.0, -1.0], [5.0, 1.0]);
        assert!(result.is_some());
        assert!((result.unwrap() - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_ray_intersection_out_of_range() {
        let ray = Ray {
            origin: [0.0, 0.0],
            direction: [1.0, 0.0],
            max_distance: 3.0,
        };
        
        let result = ray.intersects_line([5.0, -1.0], [5.0, 1.0]);
        assert!(result.is_none()); // Beyond ray lifetime
    }
    
    #[test]
    fn test_ray_parallel_line() {
        let ray = Ray {
            origin: [0.0, 0.0],
            direction: [1.0, 0.0],
            max_distance: 10.0,
        };
        
        let result = ray.intersects_line([0.0, 1.0], [10.0, 1.0]);
        assert!(result.is_none()); // Parallel lines don't intersect
    }
}
