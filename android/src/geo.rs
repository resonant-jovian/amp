//! Geolocation and coordinate-based address matching
//!
//! Provides functions to match GPS coordinates to parking restriction addresses
//! using distance calculations.
use crate::matching::get_parking_data;
use crate::static_data::StaticAddressEntry;
/// Earth's radius in kilometers (mean radius)
const EARTH_RADIUS_KM: f64 = 6371.0;
/// Find address by GPS coordinates
///
/// Searches the parking restriction database for the closest address
/// within a reasonable distance of the given coordinates.
///
/// # Arguments
/// * `lat` - Latitude in decimal degrees
/// * `lon` - Longitude in decimal degrees
///
/// # Returns
/// Some(StaticAddressEntry) if address found within 100m, None otherwise
///
/// # Examples
/// ```no_run
/// if let Some(address) = find_address_by_coordinates(57.7089, 11.9746) {
///     println!("Found: {} {}", address.gata, address.gatunummer);
/// }
/// ```
pub fn find_address_by_coordinates(lat: f64, lon: f64) -> Option<StaticAddressEntry> {
    let data = get_parking_data();
    let max_distance_km = 0.1;
    let mut closest: Option<(f64, StaticAddressEntry)> = None;
    for entry in data.values() {
        let entry_lat = entry.coordinates[0];
        let entry_lon = entry.coordinates[1];
        let distance = haversine_distance(lat, lon, entry_lat, entry_lon);
        if distance <= max_distance_km {
            match &closest {
                Some((prev_dist, _)) if distance < *prev_dist => {
                    closest = Some((distance, entry.clone()));
                }
                None => {
                    closest = Some((distance, entry.clone()));
                }
                _ => {}
            }
        }
    }
    closest.map(|(_, entry)| entry)
}
/// Calculate Haversine distance between two coordinates
///
/// Uses the Haversine formula to calculate the great-circle distance
/// between two points on a sphere given their longitudes and latitudes.
///
/// # Arguments
/// * `lat1` - Latitude of first point (decimal degrees)
/// * `lon1` - Longitude of first point (decimal degrees)
/// * `lat2` - Latitude of second point (decimal degrees)
/// * `lon2` - Longitude of second point (decimal degrees)
///
/// # Returns
/// Distance in kilometers
///
/// # Examples
/// ```
/// use amp_android::geo::haversine_distance;
///
/// // Distance between two points in Göteborg
/// let distance = haversine_distance(57.7089, 11.9746, 57.7065, 11.9673);
/// assert!(distance < 1.0); // Less than 1 km
/// ```
pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lon1_rad = lon1.to_radians();
    let lat2_rad = lat2.to_radians();
    let lon2_rad = lon2.to_radians();
    let dlat = lat2_rad - lat1_rad;
    let dlon = lon2_rad - lon1_rad;
    let a =
        (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_KM * c
}
/// Find all addresses within radius
///
/// Returns all parking restriction addresses within the specified radius
/// of the given coordinates, sorted by distance.
///
/// # Arguments
/// * `lat` - Latitude in decimal degrees
/// * `lon` - Longitude in decimal degrees
/// * `radius_km` - Search radius in kilometers
///
/// # Returns
/// Vector of (distance, address) tuples sorted by distance (closest first)
pub fn find_addresses_within_radius(
    lat: f64,
    lon: f64,
    radius_km: f64,
) -> Vec<(f64, StaticAddressEntry)> {
    let data = get_parking_data();
    let mut results: Vec<(f64, StaticAddressEntry)> = Vec::new();
    for entry in data.values() {
        let entry_lat = entry.coordinates[0];
        let entry_lon = entry.coordinates[1];
        let distance = haversine_distance(lat, lon, entry_lat, entry_lon);
        if distance <= radius_km {
            results.push((distance, entry.clone()));
        }
    }
    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    results
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_haversine_distance_same_point() {
        let distance = haversine_distance(57.7089, 11.9746, 57.7089, 11.9746);
        assert!(distance < 0.001);
    }
    #[test]
    fn test_haversine_distance_known() {
        let distance = haversine_distance(57.7089, 11.9746, 59.3293, 18.0686);
        assert!(distance > 460.0 && distance < 480.0);
    }
    #[test]
    fn test_haversine_distance_small() {
        let distance = haversine_distance(57.7089, 11.9746, 57.7099, 11.9746);
        assert!(distance > 0.1 && distance < 0.2);
    }
}
