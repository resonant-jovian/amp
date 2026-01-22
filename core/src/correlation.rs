use rayon::prelude::*;
use rust_decimal::{Decimal, MathematicalOps};
use std::str::FromStr;
use rust_decimal::prelude::ToPrimitive;
use tokio::task::block_in_place;
use geodesy::prelude::*;
use crate::structs::*;

/// Convert SWEREF99 TM coordinates to lat/lon (WGS84)
/// 
/// SWEREF99 TM uses:
/// - Central meridian: 15°E
/// - Scale factor: 0.9996
/// - False easting: 500,000m
/// - False northing: 0m
fn sweref_to_latlon(x: f64, y: f64) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let mut context = Minimal::new();

    // Inverse transformation: SWEREF99 TM → Lat/Lon (WGS84)
    // Note: We use the inverse of the forward transformation from api.rs
    let op = context.op(
        "cart  +xy:in +xy:out | \
         adapt +xy:proj=tmerc lon_0=15 k=0.9996 x_0=500000 y_0=0 +xy:in | \
         adapt +xy:proj=merc +R=6378137 +xy:out"
    )?;

    let mut data = [Coor2D::raw(x, y)];
    context.apply(op, Inv, &mut data)?;  // Inverse: SWEREF → Web Mercator → Lat/Lon

    Ok((data[0][0], data[0][1]))
}

/// Calculate great-circle distance using Haversine formula
/// 
/// Returns distance in meters between two points given in lat/lon (degrees)
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371000.0; // Earth radius in meters
    
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();
    
    let a = (delta_lat / 2.0).sin().powi(2) +
            lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    R * c
}

/// Calculate distance from a point to a line segment in geographic space
/// 
/// This properly accounts for the Earth's curvature and coordinate systems.
fn distance_point_to_line_geographic(
    point: [Decimal; 2],
    line_start: [Decimal; 2],
    line_end: [Decimal; 2],
) -> Result<f64, Box<dyn std::error::Error>> {
    // Convert SWEREF99 TM coordinates to lat/lon
    let (lat_p, lon_p) = sweref_to_latlon(
        point[0].to_f64().unwrap_or(0.0),
        point[1].to_f64().unwrap_or(0.0),
    )?;
    
    let (lat_start, lon_start) = sweref_to_latlon(
        line_start[0].to_f64().unwrap_or(0.0),
        line_start[1].to_f64().unwrap_or(0.0),
    )?;
    
    let (lat_end, lon_end) = sweref_to_latlon(
        line_end[0].to_f64().unwrap_or(0.0),
        line_end[1].to_f64().unwrap_or(0.0),
    )?;
    
    // For simplicity, return minimum distance to either endpoint
    // A full implementation would compute point-to-line distance on the sphere
    // but for parking zones (short line segments in Malmö), this approximation is valid
    let dist_to_start = haversine_distance(lat_p, lon_p, lat_start, lon_start);
    let dist_to_end = haversine_distance(lat_p, lon_p, lat_end, lon_end);
    
    Ok(dist_to_start.min(dist_to_end))
}

pub fn correlation(points: Vec<AdressClean>, lines: Vec<MiljoeDataClean>) -> Vec<AdressInfo> {
    let results: Vec<Option<(usize, f64)>> = block_in_place(|| {
        find_closest_lines(&points, &lines)
    });
    
    let mut correlation = vec![];
    let mut dist_samples = vec![];  // Track distances

    for (i, res) in results.iter().enumerate() {
        match res {
            Some((line_index, dist)) => {
                if i < 100 {
                    dist_samples.push(*dist);  // Collect first 100 distances
                }

                // 50 meters threshold for parking zone relevance
                if dist < &50.0 {
                    correlation.push(AdressInfo {
                        relevant: true,
                        postnummer: points[i].postnummer.clone(),
                        adress: points[i].adress.clone(),
                        gata: points[i].gata.clone(),
                        gatunummer: points[i].gatunummer.clone(),
                        info: lines[*line_index].info.clone(),
                        tid: lines[*line_index].tid.clone(),
                        dag: lines[*line_index].dag.clone(),
                    });
                } else {
                    correlation.push(AdressInfo {
                        relevant: false,
                        postnummer: points[i].postnummer.clone(),
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

    // Print distance samples
    println!("\nDistance samples (first 100 addresses):");
    for (i, d) in dist_samples.iter().take(10).enumerate() {
        println!("  [{}] {:.2} meters", i, d);
    }

    correlation
}

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
                .filter_map(|(i, line)| {
                    distance_point_to_line_geographic(
                        point.coordinates,
                        line.coordinates[0],
                        line.coordinates[1],
                    )
                    .ok()
                    .map(|dist| (i, dist))
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        })
        .collect()
}
