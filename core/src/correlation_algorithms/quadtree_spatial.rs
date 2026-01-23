//! Quadtree spatial partitioning algorithm
//! Recursively divides space into 4 quadrants for efficient spatial queries
//! Good for non-uniform data distributions

use crate::structs::{AdressClean, MiljoeDataClean};
use crate::correlation_algorithms::CorrelationAlgo;
use rust_decimal::prelude::ToPrimitive;

const MAX_DEPTH: u32 = 10;
const MAX_ITEMS_PER_NODE: usize = 8;

pub struct QuadtreeSpatialAlgo {
    root: QuadNode,
}

struct QuadNode {
    bounds: Bounds,
    items: Vec<(usize, [f64; 2], [f64; 2])>, // (index, start, end)
    children: Option<Box<[QuadNode; 4]>>,
}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Bounds {
    fn contains_point(&self, point: [f64; 2]) -> bool {
        point[0] >= self.min_x
            && point[0] <= self.max_x
            && point[1] >= self.min_y
            && point[1] <= self.max_y
    }
    
    fn intersects_line(&self, start: [f64; 2], end: [f64; 2]) -> bool {
        let line_min_x = start[0].min(end[0]);
        let line_max_x = start[0].max(end[0]);
        let line_min_y = start[1].min(end[1]);
        let line_max_y = start[1].max(end[1]);
        
        !(line_max_x < self.min_x
            || line_min_x > self.max_x
            || line_max_y < self.min_y
            || line_min_y > self.max_y)
    }
    
    fn subdivide(&self) -> [Bounds; 4] {
        let mid_x = (self.min_x + self.max_x) / 2.0;
        let mid_y = (self.min_y + self.max_y) / 2.0;
        
        [
            // NW
            Bounds {
                min_x: self.min_x,
                min_y: mid_y,
                max_x: mid_x,
                max_y: self.max_y,
            },
            // NE
            Bounds {
                min_x: mid_x,
                min_y: mid_y,
                max_x: self.max_x,
                max_y: self.max_y,
            },
            // SW
            Bounds {
                min_x: self.min_x,
                min_y: self.min_y,
                max_x: mid_x,
                max_y: mid_y,
            },
            // SE
            Bounds {
                min_x: mid_x,
                min_y: self.min_y,
                max_x: self.max_x,
                max_y: mid_y,
            },
        ]
    }
}

impl QuadNode {
    fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            items: Vec::new(),
            children: None,
        }
    }
    
    fn insert(
        &mut self,
        index: usize,
        start: [f64; 2],
        end: [f64; 2],
        depth: u32,
    ) {
        if !self.bounds.intersects_line(start, end) {
            return;
        }
        
        if self.children.is_none() {
            self.items.push((index, start, end));
            
            // Subdivide if over capacity and not at max depth
            if self.items.len() > MAX_ITEMS_PER_NODE && depth < MAX_DEPTH {
                self.subdivide_node(depth);
            }
        } else {
            // Insert into children
            if let Some(ref mut children) = self.children {
                for child in children.iter_mut() {
                    child.insert(index, start, end, depth + 1);
                }
            }
        }
    }
    
    fn subdivide_node(&mut self, depth: u32) {
        let sub_bounds = self.bounds.subdivide();
        let mut children = Box::new([
            QuadNode::new(sub_bounds[0]),
            QuadNode::new(sub_bounds[1]),
            QuadNode::new(sub_bounds[2]),
            QuadNode::new(sub_bounds[3]),
        ]);
        
        // CRITICAL FIX: Redistribute existing items to children
        let items_to_redistribute = std::mem::take(&mut self.items);
        
        for (idx, start, end) in items_to_redistribute {
            for child in children.iter_mut() {
                child.insert(idx, start, end, depth + 1);
            }
        }
        
        self.children = Some(children);
    }
    
    fn query_point(&self, point: [f64; 2], results: &mut Vec<usize>) {
        if !self.bounds.contains_point(point) {
            return;
        }
        
        // Add items from this node
        for (idx, _, _) in &self.items {
            results.push(*idx);
        }
        
        // Recursively query children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_point(point, results);
            }
        }
    }
}

impl QuadtreeSpatialAlgo {
    pub fn new(parking_lines: &[MiljoeDataClean]) -> Self {
        // Calculate bounds
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        
        for line in parking_lines {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                min_x = min_x.min(x1).min(x2);
                min_y = min_y.min(y1).min(y2);
                max_x = max_x.max(x1).max(x2);
                max_y = max_y.max(y1).max(y2);
            }
        }
        
        // Add 10% padding
        let width = max_x - min_x;
        let height = max_y - min_y;
        min_x -= width * 0.1;
        min_y -= height * 0.1;
        max_x += width * 0.1;
        max_y += height * 0.1;
        
        let bounds = Bounds { min_x, min_y, max_x, max_y };
        let mut root = QuadNode::new(bounds);
        
        // Insert all lines
        for (idx, line) in parking_lines.iter().enumerate() {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                line.coordinates[0][0].to_f64(),
                line.coordinates[0][1].to_f64(),
                line.coordinates[1][0].to_f64(),
                line.coordinates[1][1].to_f64(),
            ) {
                root.insert(idx, [x1, y1], [x2, y2], 0);
            }
        }
        
        Self { root }
    }
}

impl CorrelationAlgo for QuadtreeSpatialAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        parking_lines: &[MiljoeDataClean],
    ) -> Option<(usize, f64)> {
        let point = [
            address.coordinates[0].to_f64()?,
            address.coordinates[1].to_f64()?,
        ];
        
        let mut candidates = Vec::new();
        self.root.query_point(point, &mut candidates);
        
        // Remove duplicates (lines may be in multiple nodes)
        candidates.sort_unstable();
        candidates.dedup();
        
        // Find closest among candidates
        candidates
            .into_iter()
            .filter_map(|idx| {
                let line = parking_lines.get(idx)?;
                let start = [
                    line.coordinates[0][0].to_f64()?,
                    line.coordinates[0][1].to_f64()?,
                ];
                let end = [
                    line.coordinates[1][0].to_f64()?,
                    line.coordinates[1][1].to_f64()?,
                ];
                
                let dist = distance_point_to_line(point, start, end);
                Some((idx, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    
    fn name(&self) -> &'static str {
        "Quadtree Spatial"
    }
}

fn distance_point_to_line(point: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> f64 {
    let line_vec = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let point_vec = [point[0] - line_start[0], point[1] - line_start[1]];
    
    let line_len_sq = line_vec[0] * line_vec[0] + line_vec[1] * line_vec[1];
    
    if line_len_sq == 0.0 {
        let dx = point[0] - line_start[0];
        let dy = point[1] - line_start[1];
        return (dx * dx + dy * dy).sqrt();
    }
    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        };
        
        assert!(bounds.contains_point([5.0, 5.0]));
        assert!(!bounds.contains_point([15.0, 5.0]));
    }
    
    #[test]
    fn test_bounds_subdivide() {
        let bounds = Bounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        };
        
        let sub = bounds.subdivide();
        assert_eq!(sub[0].max_x, 5.0); // NW
        assert_eq!(sub[1].min_x, 5.0); // NE
    }
}
