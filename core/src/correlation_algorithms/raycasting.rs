//! Raycasting correlation algorithm
//! Uses 36 rays (every 10 degrees) to find intersections with parking zones

use crate::structs::{AdressClean, MiljoeDataClean};
use crate::correlation_algorithms::CorrelationAlgo;
use rust_decimal::prelude::ToPrimitive;
use std::f64::consts::PI;

const MAX_DISTANCE_METERS: f64 = 50.0;
const RAY_ANGLES: usize = 36; // Every 10 degrees

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
        
        let mut min_distance = f64::INFINITY;
        let mut closest_index = None;
        
        // Cast rays in all directions
        for i in 0..RAY_ANGLES {
            let angle = (i as f64 * 360.0 / RAY_ANGLES as f64) * PI / 180.0;
            let ray_end = [
                point[0] + angle.cos() * MAX_DISTANCE_METERS * 2.0,
                point[1] + angle.sin() * MAX_DISTANCE_METERS * 2.0,
            ];
            
            for (idx, line) in parking_lines.iter().enumerate() {
                let line_start = [
                    line.coordinates[0][0].to_f64()?,
                    line.coordinates[0][1].to_f64()?,
                ];
                let line_end = [
                    line.coordinates[1][0].to_f64()?,
                    line.coordinates[1][1].to_f64()?,
                ];
                
                if let Some(intersection) = ray_intersects_line(point, ray_end, line_start, line_end) {
                    let dx = point[0] - intersection[0];
                    let dy = point[1] - intersection[1];
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    if dist < min_distance && dist <= MAX_DISTANCE_METERS {
                        min_distance = dist;
                        closest_index = Some(idx);
                    }
                }
            }
        }
        
        closest_index.map(|idx| (idx, min_distance))
    }
    
    fn name(&self) -> &'static str {
        "Raycasting"
    }
}

fn ray_intersects_line(
    ray_start: [f64; 2],
    ray_end: [f64; 2],
    line_start: [f64; 2],
    line_end: [f64; 2],
) -> Option<[f64; 2]> {
    let r_dx = ray_end[0] - ray_start[0];
    let r_dy = ray_end[1] - ray_start[1];
    let s_dx = line_end[0] - line_start[0];
    let s_dy = line_end[1] - line_start[1];
    
    let denominator = r_dx * s_dy - r_dy * s_dx;
    
    if denominator.abs() < 1e-10 {
        return None; // Parallel
    }
    
    let t = ((line_start[0] - ray_start[0]) * s_dy - (line_start[1] - ray_start[1]) * s_dx) / denominator;
    let u = ((line_start[0] - ray_start[0]) * r_dy - (line_start[1] - ray_start[1]) * r_dx) / denominator;
    
    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        Some([
            ray_start[0] + t * r_dx,
            ray_start[1] + t * r_dy,
        ])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_intersection() {
        let intersection = ray_intersects_line(
            [0.0, 0.0], [10.0, 10.0],
            [0.0, 10.0], [10.0, 0.0]
        );
        assert!(intersection.is_some());
        let point = intersection.unwrap();
        assert!((point[0] - 5.0).abs() < 0.001);
        assert!((point[1] - 5.0).abs() < 0.001);
    }
}
