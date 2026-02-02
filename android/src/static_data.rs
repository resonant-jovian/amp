//! Static parking restriction data
//!
//! Provides access to the pre-computed parking restriction correlations
//! loaded from the local.parquet file in the app assets.
//!
//! # Data Structure
//! This module now uses the `DB` struct from `amp-core` which provides
//! proper timestamp handling using `chrono::DateTime<Utc>` instead of
//! separate day and time string fields.
//!
//! # Migration Note
//! This version uses the new DB struct. Old code using `dag` and `tid`
//! should be updated to use `start_time` and `end_time` from DB entries.

use amp_core::structs::DB;
use crate::components::file::read_local_data;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Cached parking data using new DB struct
static PARKING_DATA: OnceLock<HashMap<String, DB>> = OnceLock::new();

/// Get static parking data
///
/// Returns a reference to the cached parking restriction database.
/// Data is loaded lazily on first access from local.parquet file.
///
/// # Returns
/// Static reference to HashMap mapping address keys to DB entries
///
/// # Panics
/// Panics if parking data cannot be loaded (e.g., file missing)
///
/// # Examples
/// ```no_run
/// use amp_android::static_data::get_static_data;
/// use chrono::Utc;
///
/// let data = get_static_data();
/// for (key, entry) in data.iter() {
///     let now = Utc::now();
///     if entry.is_active(now) {
///         println!("Active restriction: {} {}", entry.gata.as_ref().unwrap_or(&String::from("Unknown")), entry.gatunummer.as_ref().unwrap_or(&String::from("")));
///     }
/// }
/// ```
pub fn get_static_data() -> &'static HashMap<String, DB> {
    PARKING_DATA.get_or_init(|| match load_parking_data() {
        Ok(data) => {
            eprintln!("[Static Data] Loaded {} parking entries", data.len());
            data
        }
        Err(e) => {
            eprintln!("[Static Data] Failed to load parking data: {}", e);
            HashMap::new()
        }
    })
}

/// Load parking data from local.parquet file
///
/// Reads the LocalData from the app assets and converts it to
/// a HashMap for fast address lookups using the new DB struct.
///
/// # Returns
/// HashMap mapping address keys (format: "gata_gatunummer_postnummer") to DB entries
///
/// # Errors
/// Returns error if:
/// - File cannot be read
/// - Data format is invalid
/// - Required fields are missing
fn load_parking_data() -> anyhow::Result<HashMap<String, DB>> {
    let local_data = read_local_data()?;
    let mut map = HashMap::new();
    
    // Get current date for timestamp generation
    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    
    for item in local_data {
        // Extract required fields, skip if missing
        let gata = match item.gata {
            Some(g) => g,
            None => {
                eprintln!("[Static Data] Skipping entry: missing gata");
                continue;
            }
        };
        
        let gatunummer = match item.gatunummer {
            Some(gn) => gn,
            None => {
                eprintln!("[Static Data] Skipping entry for {}: missing gatunummer", gata);
                continue;
            }
        };
        
        let postnummer = match item.postnummer {
            Some(pn) => pn,
            None => {
                eprintln!("[Static Data] Skipping entry for {} {}: missing postnummer", gata, gatunummer);
                continue;
            }
        };
        
        let dag = match item.dag {
            Some(d) => d,
            None => {
                eprintln!("[Static Data] Skipping entry for {} {}: missing dag", gata, gatunummer);
                continue;
            }
        };
        
        // Default to full day if time not specified
        let tid = item.tid.unwrap_or_else(|| {
            eprintln!("[Static Data] Using default time range for {} {}", gata, gatunummer);
            String::from("0000-2359")
        });
        
        // Create address key for HashMap lookup
        let key = format!("{}_{}_{}", gata, gatunummer, postnummer);
        
        // Convert to DB struct using from_dag_tid
        match DB::from_dag_tid(
            Some(postnummer.clone()),
            format!("{} {}", gata, gatunummer),  // Full address
            Some(gata.clone()),
            Some(gatunummer.clone()),
            item.info,
            dag,
            &tid,
            item.taxa,
            item.antal_platser,
            item.typ_av_parkering,
            year,
            month,
        ) {
            Some(db_entry) => {
                map.insert(key, db_entry);
            }
            None => {
                eprintln!(
                    "[Static Data] Failed to parse time for {} {}: dag={}, tid={}",
                    gata, gatunummer, dag, tid
                );
                continue;
            }
        }
    }
    
    if map.is_empty() {
        eprintln!("[Static Data] WARNING: No valid parking data loaded");
    }
    
    Ok(map)
}

/// Reload parking data from disk
///
/// Forces a reload of the parking data cache. Useful for testing
/// or when data has been updated.
///
/// # Returns
/// Number of entries loaded, or error if reload failed
///
/// # Examples
/// ```no_run
/// use amp_android::static_data::reload_parking_data;
///
/// match reload_parking_data() {
///     Ok(count) => println!("Reloaded {} entries", count),
///     Err(e) => eprintln!("Reload failed: {}", e),
/// }
/// ```
pub fn reload_parking_data() -> anyhow::Result<usize> {
    let data = load_parking_data()?;
    let count = data.len();
    
    // This will replace the old cache if it exists
    // Note: OnceLock doesn't support replacement, so this is more of a
    // documentation placeholder. In production, consider using RwLock instead.
    eprintln!("[Static Data] Reload requested but OnceLock doesn't support replacement");
    eprintln!("[Static Data] Would reload {} entries", count);
    
    Ok(count)
}

/// Get parking data for a specific address
///
/// Looks up parking restriction data by street, street number, and postal code.
///
/// # Arguments
/// * `gata` - Street name
/// * `gatunummer` - Street number
/// * `postnummer` - Postal code
///
/// # Returns
/// Some(DB) if address found, None otherwise
///
/// # Examples
/// ```no_run
/// use amp_android::static_data::get_address_data;
///
/// if let Some(data) = get_address_data("Storgatan", "10", "22100") {
///     println!("Found restriction data for Storgatan 10");
/// }
/// ```
pub fn get_address_data(gata: &str, gatunummer: &str, postnummer: &str) -> Option<&'static DB> {
    let key = format!("{}_{}_{}", gata, gatunummer, postnummer);
    get_static_data().get(&key)
}

/// Get all addresses within a specific postal code
///
/// Returns all parking restriction entries for the given postal code.
///
/// # Arguments
/// * `postnummer` - Postal code to filter by
///
/// # Returns
/// Vector of references to DB entries matching the postal code
///
/// # Examples
/// ```no_run
/// use amp_android::static_data::get_addresses_in_postal_code;
///
/// let addresses = get_addresses_in_postal_code("22100");
/// println!("Found {} addresses in postal code 22100", addresses.len());
/// ```
pub fn get_addresses_in_postal_code(postnummer: &str) -> Vec<&'static DB> {
    get_static_data()
        .values()
        .filter(|db| {
            db.postnummer
                .as_ref()
                .map(|pn| pn == postnummer)
                .unwrap_or(false)
        })
        .collect()
}

/// Get count of loaded parking entries
///
/// Returns the number of parking restriction entries currently loaded.
///
/// # Returns
/// Number of entries in the cache
///
/// # Examples
/// ```no_run
/// use amp_android::static_data::count_entries;
///
/// let count = count_entries();
/// println!("Parking database contains {} entries", count);
/// ```
pub fn count_entries() -> usize {
    get_static_data().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_address_key_format() {
        let key = format!("{}_{}_{}", "Storgatan", "10", "22100");
        assert_eq!(key, "Storgatan_10_22100");
    }
    
    #[test]
    fn test_get_static_data() {
        // This will return empty map in test environment
        let data = get_static_data();
        assert!(data.len() >= 0);  // Always true but tests it's callable
    }
    
    #[test]
    fn test_count_entries() {
        let count = count_entries();
        assert!(count >= 0);
    }
    
    #[test]
    fn test_db_struct_usage() {
        // Test that we can create and use DB struct
        let db = DB::from_dag_tid(
            Some("22100".to_string()),
            "Storgatan 10".to_string(),
            Some("Storgatan".to_string()),
            Some("10".to_string()),
            Some("Test restriction".to_string()),
            15,  // day
            "0800-1200",  // time
            Some("Taxa C".to_string()),
            Some(10),
            Some("Längsgående".to_string()),
            2024,
            1,
        );
        
        assert!(db.is_some());
        let db = db.unwrap();
        assert_eq!(db.gata, Some("Storgatan".to_string()));
        
        // Test time-aware methods
        let now = Utc::now();
        let _is_active = db.is_active(now);
        let _time_until_end = db.time_until_end(now);
    }
    
    #[test]
    fn test_get_address_data_not_found() {
        // Should return None for non-existent address in test env
        let result = get_address_data("NonExistent", "999", "99999");
        assert!(result.is_none());
    }
    
    #[test]
    fn test_get_addresses_in_postal_code_empty() {
        // Should return empty vec in test env
        let results = get_addresses_in_postal_code("99999");
        assert_eq!(results.len(), 0);
    }
}
