use rayon::prelude::*;
use rust_decimal::{Decimal, MathematicalOps};
use std::str::FromStr;
use rust_decimal::prelude::ToPrimitive;
use crate::structs::*;

pub fn correlation(points: Vec<AdressClean>, lines: Vec<MiljoeDataClean>) -> Vec<AdressInfo> {
    let results = find_closest_lines(&points, &lines);

    let mut correlation = vec![];

    for (i, res) in results.iter().enumerate() {
        match res {
            Some((line_index, dist)) => {
                if dist < &Decimal::from_str("0.001").unwrap() {
                    correlation.push(AdressInfo {
                        relevant: true,
                        postnummer: points[i].postnummer,
                        adress: points[i].adress.clone(),
                        gata: points[i].gata.clone(),
                        gatunummer: points[i].gatunummer.clone(),
                        info: lines[*line_index].info.clone(),
                        tid: lines[*line_index].tid.clone(),
                        dag: lines[*line_index].dag.clone(),
                    });
                }
                else {
                    correlation.push(AdressInfo {
                        relevant: false,
                        postnummer: points[i].postnummer,
                        adress: points[i].adress.clone(),
                        gata: points[i].gata.clone(),
                        gatunummer: points[i].gatunummer.clone(),
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
    point: [Decimal; 2],
    line_seg_start: [Decimal; 2],
    line_seg_end: [Decimal; 2],
) -> Decimal {
    // Convert to f64 for calculations
    let p_x = point[0].to_f64().unwrap_or(0.0);
    let p_y = point[1].to_f64().unwrap_or(0.0);
    let a_x = line_seg_start[0].to_f64().unwrap_or(0.0);
    let a_y = line_seg_start[1].to_f64().unwrap_or(0.0);
    let b_x = line_seg_end[0].to_f64().unwrap_or(0.0);
    let b_y = line_seg_end[1].to_f64().unwrap_or(0.0);

    // AB = B - A
    let ab_x = b_x - a_x;
    let ab_y = b_y - a_y;
    // AP = P - A
    let ap_x = p_x - a_x;
    let ap_y = p_y - a_y;

    // |AB|^2
    let ab_len_sq = ab_x * ab_x + ab_y * ab_y;

    // Degenerate segment (A == B)
    if ab_len_sq == 0.0 {
        let result = ap_x * ap_x + ap_y * ap_y;
        return Decimal::from_f64_retain(result).unwrap_or(Decimal::ZERO);
    }

    // Projection parameter clamped to [0, 1]
    let t = ((ap_x * ab_x + ap_y * ab_y) / ab_len_sq).clamp(0.0, 1.0);

    // Closest point on the segment
    let closest_x = a_x + t * ab_x;
    let closest_y = a_y + t * ab_y;

    // Squared distance from P to the closest point
    let diff_x = p_x - closest_x;
    let diff_y = p_y - closest_y;
    let result = diff_x * diff_x + diff_y * diff_y;

    Decimal::from_f64_retain(result).unwrap_or(Decimal::ZERO)
}

/// Find the closest line index + distance for each point in parallel
pub fn find_closest_lines(
    points: &[AdressClean],
    lines: &[MiljoeDataClean],
) -> Vec<Option<(usize, Decimal)>> {
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
                .map(|(i, dist_sq)| (i, dist_sq.sqrt().unwrap_or_default()))
        })
        .collect()
}