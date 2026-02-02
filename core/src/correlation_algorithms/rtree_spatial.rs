//! R-tree spatial indexing algorithm
//! Uses rstar crate for O(log n) nearest-neighbor queries
//! Best performance for large datasets (1000+ parking zones)
use crate::correlation_algorithms::common::*;
use crate::correlation_algorithms::{CorrelationAlgo, ParkeringCorrelationAlgo};
use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use crate::{extract_line_coordinates, extract_point_coordinates};
use rstar::{PointDistance, RTree, AABB};

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
        let dist = distance_point_to_line(*point, self.start, self.end);
        dist * dist
    }
}

impl RTreeSpatialAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        let segments: Vec<IndexedLineSegment> = parking_lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let (start, end) = extract_line_coordinates!(line)?;
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
        let point = extract_point_coordinates!(address)?;
        let nearest = self.rtree.nearest_neighbor(&point)?;
        let dist = distance_point_to_line(point, nearest.start, nearest.end);

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
                let (start, end) = extract_line_coordinates!(line)?;
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
        let point = extract_point_coordinates!(address)?;
        let nearest = self.rtree.nearest_neighbor(&point)?;
        let dist = distance_point_to_line(point, nearest.start, nearest.end);

        (dist <= MAX_DISTANCE_METERS).then_some((nearest.index, dist))
    }

    fn name(&self) -> &'static str {
        "R-Tree Spatial Index (Parkering)"
    }
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
