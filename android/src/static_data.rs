//! Static parking restriction data
//!
//! Provides access to the pre-computed parking restriction correlations
//! loaded from the local.parquet file in the app assets.
use crate::components::file::read_local_data;
use std::collections::HashMap;
use std::sync::OnceLock;
/// Cached parking data
static PARKING_DATA: OnceLock<HashMap<String, StaticAddressEntry>> = OnceLock::new();
/// Represents a parking restriction address entry
///
/// Contains all relevant information about parking restrictions
/// for a specific address location.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticAddressEntry {
    /// Street name (Swedish: gata)
    pub gata: String,
    /// Street number
    pub gatunummer: String,
    /// Postal code
    pub postnummer: String,
    /// Day of restriction (0=Monday, 6=Sunday)
    pub dag: u8,
    /// Time range string (e.g., "09:00-17:00")
    pub tid: String,
    /// GPS coordinates [latitude, longitude]
    pub coordinates: [f64; 2],
}
/// Get static parking data
///
/// Returns a reference to the cached parking restriction database.
/// Data is loaded lazily on first access from local.parquet file.
///
/// # Returns
/// Static reference to HashMap mapping address keys to entries
///
/// # Panics
/// Panics if parking data cannot be loaded (e.g., file missing)
///
/// # Examples
/// ```no_run
/// let data = get_static_data();
/// for (key, entry) in data.iter() {
///     println!("{}: {} {}", key, entry.gata, entry.gatunummer);
/// }
/// ```
pub fn get_static_data() -> &'static HashMap<String, StaticAddressEntry> {
    PARKING_DATA.get_or_init(|| match load_parking_data() {
        Ok(data) => {
            eprintln!("Loaded {} parking entries", data.len());
            data
        }
        Err(e) => {
            eprintln!("Failed to load parking data: {}", e);
            HashMap::new()
        }
    })
}
/// Load parking data from local.parquet file
///
/// Reads the LocalData from the app assets and converts it to
/// a HashMap for fast address lookups.
fn load_parking_data() -> anyhow::Result<HashMap<String, StaticAddressEntry>> {
    let local_data = read_local_data()?;
    let mut map = HashMap::new();
    for item in local_data {
        let gata = match item.gata {
            Some(g) => g,
            None => continue,
        };
        let gatunummer = match item.gatunummer {
            Some(gn) => gn,
            None => continue,
        };
        let postnummer = match item.postnummer {
            Some(pn) => pn,
            None => continue,
        };
        let dag = match item.dag {
            Some(d) => d,
            None => continue,
        };
        let tid = item.tid.unwrap_or_else(|| String::from("00:00-23:59"));
        let lat = item.lat.unwrap_or(0.0);
        let lon = item.lon.unwrap_or(0.0);
        let key = format!("{}_{}_{}", gata, gatunummer, postnummer);
        let entry = StaticAddressEntry {
            gata,
            gatunummer,
            postnummer,
            dag,
            tid,
            coordinates: [lat, lon],
        };
        map.insert(key, entry);
    }
    Ok(map)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_static_address_entry_creation() {
        let entry = StaticAddressEntry {
            gata: "Storgatan".to_string(),
            gatunummer: "10".to_string(),
            postnummer: "22100".to_string(),
            dag: 1,
            tid: "09:00-17:00".to_string(),
            coordinates: [57.7089, 11.9746],
        };
        assert_eq!(entry.gata, "Storgatan");
        assert_eq!(entry.coordinates[0], 57.7089);
    }
}
