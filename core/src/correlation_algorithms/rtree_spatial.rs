//! R-tree spatial indexing algorithm
//! Uses rstar crate for O(log n) nearest-neighbor queries
//! Best performance for large datasets (1000+ parking zones)
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use rstar::{AABB, PointDistance, RTree};
use rust_decimal::prelude::ToPrimitive;
const MAX_DISTANCE_METERS: f64 = 50.0;
const EARTH_RADIUS_M: f64 = 6371000.0;
pub struct RTreeSpatialAlgo {
    rtree: RTree<IndexedLineSegment>,
}
#[derive(Debug, Clone)]
struct IndexedLineSegment {
    index: usize,
    start: [f64; 2],
    end: [f64; 2],
}
impl rstar::RTreeObject for IndexedLineSegment {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        let min_x = self.start[0].min(self.end[0]);
        let min_y = self.start[1].min(self.end[1]);
        let max_x = self.start[0].max(self.end[0]);
        let max_y = self.start[1].max(self.end[1]);
        AABB::from_corners([min_x, min_y], [max_x, max_y])
    }
}
impl PointDistance for IndexedLineSegment {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dist = distance_point_to_line_segment(*point, self.start, self.end);
        dist * dist
    }
}
impl RTreeSpatialAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        let segments: Vec<IndexedLineSegment> = parking_lines
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
                Some(IndexedLineSegment {
                    index: idx,
                    start,
                    end,
                })
            })
            .collect();
        Self {
            rtree: RTree::bulk_load(segments),
        }
    }
}
impl CorrelationAlgo for RTreeSpatialAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        _parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        let nearest = self.rtree.nearest_neighbor(&point)?;
        let dist = distance_point_to_line_segment(point, nearest.start, nearest.end);
        (dist <= MAX_DISTANCE_METERS).then_some((nearest.index, dist))
    }
    fn name(&self) -> &'static str {
        "R-Tree Spatial Index"
    }
}
pub struct RTreeParkeringAlgo {
    rtree: RTree<IndexedLineSegment>,
}

impl RTreeParkeringAlgo {
    pub fn new(parking_lines: &[ParkeringsDataClean]) -> Self {
        let segments: Vec<IndexedLineSegment> = parking_lines
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
                Some(IndexedLineSegment {
                    index: idx,
                    start,
                    end,
                })
            })
            .collect();

        Self {
            rtree: RTree::bulk_load(segments),
        }
    }
}

impl ParkeringCorrelationAlgo for RTreeParkeringAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        _parking_lines: &[ParkeringsDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];

        let nearest = self.rtree.nearest_neighbor(&point)?;
        let dist = distance_point_to_line_segment(point, nearest.start, nearest.end);
        (dist <= MAX_DISTANCE_METERS).then_some((nearest.index, dist))
    }

    fn name(&self) -> &'static str {
        "R-Tree Spatial Index (Parkering)"
    }
}

/// Calculate perpendicular distance from point to line segment using Haversine
fn distance_point_to_line_segment(
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
    let t =
        ((point_vec[0] * line_vec[0] + point_vec[1] * line_vec[1]) / line_len_sq).clamp(0.0, 1.0);
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
    let a =
        (delta_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_M * c
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstar::RTreeObject;
    #[test]
    fn test_rtree_envelope() {
        let seg = IndexedLineSegment {
            index: 0,
            start: [13.0, 55.0],
            end: [13.1, 55.1],
        };
        let env = seg.envelope();
        assert_eq!(env.lower(), [13.0, 55.0]);
        assert_eq!(env.upper(), [13.1, 55.1]);
    }
}
