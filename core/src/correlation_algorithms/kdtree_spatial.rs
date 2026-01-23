//! KD-tree (2D) spatial algorithm
//! Binary space partitioning optimized for 2D point queries
//! Excellent for nearest-neighbor searches

use crate::correlation_algorithms::CorrelationAlgo;
use crate::structs::{AdressClean, MiljoeDataClean};
use rust_decimal::prelude::ToPrimitive;

const MAX_LEAF_SIZE: usize = 8;
const MAX_DISTANCE_METERS: f64 = 50.0;
const EARTH_RADIUS_M: f64 = 6371000.0;

pub struct KDTreeSpatialAlgo {
    root: Option<Box<KDNode>>,
    lines: Vec<LineSegment>,
}

#[derive(Clone)]
struct LineSegment {
    index: usize,
    start: [f64; 2],
    end: [f64; 2],
    midpoint: [f64; 2],
}

struct KDNode {
    axis: usize, // 0 for x, 1 for y
    split_value: f64,
    indices: Vec<usize>, // Indices into lines array
    left: Option<Box<KDNode>>,
    right: Option<Box<KDNode>>,
}

impl KDNode {
    fn build(segments: &mut [(usize, [f64; 2])], depth: usize) -> Option<Box<Self>> {
        if segments.is_empty() {
            return None;
        }

        let axis = depth % 2;

        if segments.len() <= MAX_LEAF_SIZE {
            // Leaf node
            return Some(Box::new(KDNode {
                axis,
                split_value: 0.0,
                indices: segments.iter().map(|(idx, _)| *idx).collect(),
                left: None,
                right: None,
            }));
        }

        // Sort by current axis and split at median
        segments.sort_by(|a, b| a.1[axis].partial_cmp(&b.1[axis]).unwrap());
        let mid = segments.len() / 2;
        let split_value = segments[mid].1[axis];

        let (left_data, right_data) = segments.split_at_mut(mid);

        Some(Box::new(KDNode {
            axis,
            split_value,
            indices: Vec::new(),
            left: Self::build(left_data, depth + 1),
            right: Self::build(right_data, depth + 1),
        }))
    }

    fn query_nearest(
        &self,
        point: [f64; 2],
        lines: &[LineSegment],
        best: &mut Option<(usize, f64)>,
    ) {
        if !self.indices.is_empty() {
            // Leaf node - check all segments
            for &idx in &self.indices {
                let line = &lines[idx];
                let dist = distance_point_to_line(point, line.start, line.end);

                // Only consider if within threshold
                if dist <= MAX_DISTANCE_METERS && (best.is_none() || dist < best.unwrap().1) {
                    *best = Some((line.index, dist));
                }
            }
            return;
        }

        // Internal node - traverse tree
        let diff = point[self.axis] - self.split_value;
        let (primary, secondary) = if diff < 0.0 {
            (&self.left, &self.right)
        } else {
            (&self.right, &self.left)
        };

        // Search primary side
        if let Some(node) = primary {
            node.query_nearest(point, lines, best);
        }

        // Check if we need to search secondary side
        // Convert degree difference to approximate meters for comparison
        let diff_meters = diff.abs() * 111000.0; // Rough approximation
        let should_search_secondary = best.is_none() || diff_meters < best.unwrap().1;

        if should_search_secondary {
            if let Some(node) = secondary {
                node.query_nearest(point, lines, best);
            }
        }
    }
}

impl KDTreeSpatialAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        let lines: Vec<LineSegment> = parking_lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let start = [
                    line.coordinates[0][0].to_f64()?,
                    line.coordinates[0][1].to_f64()?,
                ];
                let end = [
                    line.coordinates[1][0].to_f64()?,
                    line.coordinates[1][1].to_f64()?,
                ];
                let midpoint = [(start[0] + end[0]) / 2.0, (start[1] + end[1]) / 2.0];

                Some(LineSegment {
                    index: idx,
                    start,
                    end,
                    midpoint,
                })
            })
            .collect();

        // Build KD-tree using line midpoints
        let mut indexed_midpoints: Vec<(usize, [f64; 2])> = lines
            .iter()
            .enumerate()
            .map(|(i, seg)| (i, seg.midpoint))
            .collect();

        let root = KDNode::build(&mut indexed_midpoints, 0);

        Self { root, lines }
    }
}

impl CorrelationAlgo for KDTreeSpatialAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        _parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];

        let mut best = None;

        if let Some(ref root) = self.root {
            root.query_nearest(point, &self.lines, &mut best);
        }

        best
    }

    fn name(&self) -> &'static str {
        "KD-Tree Spatial"
    }
}

fn distance_point_to_line(point: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> f64 {
    let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let point_vec = [point[0] - line_start[0], point[1] - line_start[1]];

    let line_len_sq = line_vec[0] * line_vec[0] + line_vec[1] * line_vec[1];

    if line_len_sq == 0.0 {
        return haversine_distance(point, line_start);
    }

    let t = ((point_vec[0] * line_vec[0] + point_vec[1] * line_vec[1]) / line_len_sq)
        .clamp(0.0, 1.0);

    let closest = [
        line_start[0] + t * line_vec[0],
        line_start[1] + t * line_vec[1],
    ];

    haversine_distance(point, closest)
}

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
    fn test_kdtree_build() {
        let mut data = vec![(0, [13.0, 55.0]), (1, [13.1, 55.1]), (2, [13.2, 55.2])];

        let root = KDNode::build(&mut data, 0);
        assert!(root.is_some());
    }
}
