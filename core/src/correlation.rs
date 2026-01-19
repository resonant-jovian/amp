use nalgebra::Vector2;
use rayon::prelude::*;
use std::f64;

use crate::structs::*;

pub fn correlation(points: Vec<AdressClean>, lines: Vec<MiljoeDataClean>) -> Vec<AdressInfo> {
    let results = find_closest_lines(&points, &lines);

    let mut correlation = vec![];

    for (i, res) in results.iter().enumerate() {
        match res {
            Some((line_index, dist)) => {
                if dist < &0.001 {
                    correlation.push(AdressInfo {
                        relevant: true,
                        postnummer: points[*line_index].postnummer,
                        adress: points[*line_index].adress.clone(),
                        gata: points[*line_index].gata.clone(),
                        gatunummer: points[*line_index].gatunummer.clone(),
                        info: lines[*line_index].info.clone(),
                        tid: lines[*line_index].tid.clone(),
                        dag: lines[*line_index].dag.clone(),
                    });
                }
                else {
                    correlation.push(AdressInfo {
                        relevant: false,
                        postnummer: points[*line_index].postnummer,
                        adress: points[*line_index].adress.clone(),
                        gata: points[*line_index].gata.clone(),
                        gatunummer: points[*line_index].gatunummer.clone(),
                        info: Default::default(),
                        tid: Default::default(),
                        dag: Default::default(),
                    });
                }
            }
            None => println!("Point {} has no closest line", i),
        }
    }
    correlation
}

/// Squared distance from point to line segment
fn distance_point_to_line_squared(
    point: [f64; 2],
    line_seg_start: [f64; 2],
    line_seg_end: [f64; 2],
) -> f64 {
    // Convert arrays to nalgebra vectors
    let p = Vector2::new(point[0], point[1]);
    let a = Vector2::new(line_seg_start[0], line_seg_start[1]);
    let b = Vector2::new(line_seg_end[0], line_seg_end[1]);

    // AB = B - A
    let ab = b - a;
    // AP = P - A
    let ap = p - a;

    // |AB|^2
    let ab_len_sq = ab.dot(&ab);

    // Degenerate segment (A == B)
    if ab_len_sq == 0.0 {
        return ap.dot(&ap);
    }

    // Projection parameter clamped to [0, 1]
    let t = (ap.dot(&ab) / ab_len_sq).clamp(0.0, 1.0);

    // Closest point on the segment
    let closest = a + t * ab;

    // Squared distance from P to the closest point
    let diff = p - closest;
    diff.dot(&diff)
}

/// Find the closest line index + distance for each point in parallel
pub fn find_closest_lines(
    points: &[AdressClean],
    lines: &[MiljoeDataClean],
) -> Vec<Option<(usize, f64)>> {
    points
        .par_iter()
        .map(|point| {
            lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    (
                        i,
                        distance_point_to_line_squared(
                            point.coordinates,
                            line.coordinates[0],
                            line.coordinates[1],
                        ),
                    )
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(i, dist_sq)| (i, dist_sq.sqrt()))
        })
        .collect()
}