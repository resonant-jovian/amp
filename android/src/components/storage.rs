//! Persistent storage for Android app using Parquet format
//!
//! Provides local storage for user addresses using parquet files instead of SharedPreferences.
//! Implements backup rotation for data safety.
//!
//! # Storage Locations
//! - **User addresses**: `local.parquet` (main file)
//! - **Backup**: `local.parquet.backup` (previous version)
//!
//! # Backup Strategy
//! On write:
//! 1. Delete old backup if exists
//! 2. Rename current local.parquet to local.parquet.backup
//! 3. Write new local.parquet
//!
//! On read (if local.parquet missing):
//! 1. Try to read local.parquet.backup
//! 2. Duplicate backup to local.parquet
//! 3. If neither exists, create empty files
//!
//! # Platform Support
//! - **Android**: Full file storage using app's internal data directory
//! - **Other platforms**: In-memory mock storage for testing
//!
//! # Thread Safety
//! All storage operations are synchronized using a Mutex to prevent data races
//! when multiple UI components access storage simultaneously.
//!
//! # Examples
//! ```no_run
//! use amp_android::storage;
//! use amp_android::ui::StoredAddress;
//!
//! // Read stored addresses (thread-safe)
//! let addresses = storage::read_addresses_from_device();
//!
//! // Add new address
//! let mut addresses = addresses;
//! addresses.push(StoredAddress {
//!     id: 1,
//!     street: "Storgatan".to_string(),
//!     street_number: "10".to_string(),
//!     postal_code: "22100".to_string(),
//!     valid: true,
//!     active: true,
//!     matched_entry: None,
//! });
//!
//! // Save back to storage (thread-safe)
//! storage::write_addresses_to_device(&addresses).ok();
//! ```
use crate::ui::StoredAddress;
#[allow(unused_imports)]
use amp_core::parquet::{build_local_parquet, read_local_parquet};
#[allow(unused_imports)]
use amp_core::structs::{LocalData, DBParams, DB};
use chrono::{Datelike, Timelike};
#[allow(unused_imports)]
use std::fs::File;
#[allow(unused_imports)]
use std::fs::{self};
use std::path::PathBuf;
use std::sync::Mutex;

/// Thread-safe storage mutex to prevent concurrent access issues
static STORAGE_LOCK: Mutex<()> = Mutex::new(());

#[cfg(target_os = "android")]
const LOCAL_PARQUET_NAME: &str = "local.parquet";

#[cfg(target_os = "android")]
const BACKUP_PARQUET_NAME: &str = "local.parquet.backup";

/// Get app-specific storage directory that's writable on Android
///
/// Returns the app's internal data directory on Android.
/// On other platforms, uses current directory for testing.
#[cfg(target_os = "android")]
fn get_storage_dir() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("APP_FILES_DIR") {
        let path = PathBuf::from(dir);
        eprintln!("[Storage] Using APP_FILES_DIR: {:?}", path);
        return Ok(path);
    }

    let app_dir = PathBuf::from("/data/local/tmp/amp_storage");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).map_err(|e| {
            format!(
                "[Storage] Failed to create storage dir {:?}: {}",
                app_dir, e
            )
        })?;
        eprintln!("[Storage] Created storage dir: {:?}", app_dir);
    }
    Ok(app_dir)
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn get_storage_dir() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))
}

/// Get path to main parquet file
#[cfg(target_os = "android")]
fn get_local_parquet_path() -> Result<PathBuf, String> {
    let mut path = get_storage_dir()?;
    path.push(LOCAL_PARQUET_NAME);
    Ok(path)
}

/// Get path to backup parquet file
#[cfg(target_os = "android")]
fn get_backup_parquet_path() -> Result<PathBuf, String> {
    let mut path = get_storage_dir()?;
    path.push(BACKUP_PARQUET_NAME);
    Ok(path)
}

/// Create empty parquet file with LocalData schema
///
/// Creates a file with a single dummy row (empty address with valid=false, active=false).
/// This dummy row is filtered out when reading, resulting in an empty address list.
/// This approach ensures the parquet file always has valid schema structure.
#[cfg(target_os = "android")]
fn create_empty_parquet(path: &PathBuf) -> Result<(), String> {
    let dummy = LocalData {
        valid: false,
        active: false,
        postnummer: None,
        adress: String::new(),
        gata: None,
        gatunummer: None,
        info: None,
        tid: None,
        dag: None,
        taxa: None,
        antal_platser: None,
        typ_av_parkering: None,
    };

    let buffer = build_local_parquet(vec![dummy])
        .map_err(|e| format!("[Storage] Failed to build empty parquet: {}", e))?;

    fs::write(path, buffer).map_err(|e| {
        format!(
            "[Storage] Failed to write empty parquet to {:?}: {}",
            path, e
        )
    })?;

    eprintln!("[Storage] Created empty parquet at {:?}", path);
    Ok(())
}

/// Ensure storage files exist, create if necessary
///
/// Handles three cases:
/// 1. Neither file exists: Create both with dummy data
/// 2. Only backup exists: Restore from backup
/// 3. Only local exists: Create backup from local
#[cfg(target_os = "android")]
fn ensure_storage_files() -> Result<(), String> {
    let local_path = get_local_parquet_path()?;
    let backup_path = get_backup_parquet_path()?;

    let local_exists = local_path.exists();
    let backup_exists = backup_path.exists();

    if !local_exists && !backup_exists {
        eprintln!("[Storage] No storage files found, creating empty files");
        create_empty_parquet(&local_path)?;
        create_empty_parquet(&backup_path)?;
    } else if !local_exists && backup_exists {
        eprintln!("[Storage] local.parquet missing, duplicating from backup");
        fs::copy(&backup_path, &local_path)
            .map_err(|e| format!("[Storage] Failed to duplicate backup: {}", e))?;
    } else if local_exists && !backup_exists {
        eprintln!("[Storage] backup missing, creating from local.parquet");
        fs::copy(&local_path, &backup_path)
            .map_err(|e| format!("[Storage] Failed to create backup: {}", e))?;
    }

    Ok(())
}

/// Convert StoredAddress to LocalData for parquet storage
///
/// Preserves all address fields and matched parking data including:
/// - Basic address info (street, number, postal code)
/// - Match validity and active state
/// - Parking zone data (taxa, antal_platser, typ_av_parkering)
/// - Time restrictions (tid, dag) extracted from DB timestamps
/// - Environmental info
///
/// The tid (time range) and dag (day) are extracted from the matched_entry's
/// start_time and end_time timestamps, ensuring complete persistence of the match.
#[cfg(target_os = "android")]
fn to_local_data(addr: &StoredAddress) -> LocalData {
    let (dag, tid, info, taxa, antal_platser, typ_av_parkering) =
        if let Some(ref entry) = addr.matched_entry {
            // Extract day and time from the DB entry's timestamps
            let start_swedish = entry.start_time_swedish();
            let end_swedish = entry.end_time_swedish();
            
            let dag_value = Some(start_swedish.day() as u8);
            let tid_value = Some(format!(
                "{:02}{:02}-{:02}{:02}",
                start_swedish.hour(),
                start_swedish.minute(),
                end_swedish.hour(),
                end_swedish.minute()
            ));

            eprintln!(
                "[Storage::to_local_data] Persisting match data: dag={:?}, tid={:?}, taxa={:?}",
                dag_value, tid_value, entry.taxa
            );

            (
                dag_value,
                tid_value,
                entry.info.clone(),
                entry.taxa.clone(),
                entry.antal_platser,
                entry.typ_av_parkering.clone(),
            )
        } else {
            eprintln!("[Storage::to_local_data] No matched_entry to persist");
            (None, None, None, None, None, None)
        };

    LocalData {
        valid: addr.valid,
        active: addr.active,
        postnummer: Some(addr.postal_code.clone()),
        adress: format!("{} {}", addr.street, addr.street_number)
            .trim()
            .to_string(),
        gata: Some(addr.street.clone()),
        gatunummer: Some(addr.street_number.clone()),
        info,
        tid,
        dag,
        taxa,
        antal_platser,
        typ_av_parkering,
    }
}

/// Convert LocalData from parquet to StoredAddress
///
/// Reconstructs a StoredAddress with its matched parking data by:
/// 1. Extracting basic address fields (street, number, postal code)
/// 2. If parking data fields are present in LocalData:
///    - Reconstructs a DB entry using DBParams
///    - Populates matched_entry with the reconstructed DB
/// 3. If parking data is missing:
///    - Optionally attempts to re-match against database
///    - Falls back to None matched_entry
///
/// This ensures that matched parking zone data persists across app restarts.
/// The reconstructed DB entry maintains proper timestamps in Swedish timezone.
///
/// # Arguments
/// * `data` - LocalData read from parquet file
/// * `id` - Unique ID to assign to the StoredAddress
///
/// # Returns
/// StoredAddress with reconstructed matched_entry (if match data was persisted)
#[cfg(target_os = "android")]
fn from_local_data(data: LocalData, id: usize) -> StoredAddress {
    eprintln!("[Storage::from_local_data] === START CONVERSION ===");

    // Extract basic address components
    let (street, street_number) = if let Some(gata) = &data.gata {
        let street_number = data.gatunummer.clone().unwrap_or_default();
        eprintln!(
            "[Storage::from_local_data] Using gata field: '{}' '{}'",
            gata, street_number
        );
        (gata.clone(), street_number)
    } else {
        let parts: Vec<&str> = data.adress.rsplitn(2, ' ').collect();
        if parts.len() == 2 {
            eprintln!(
                "[Storage::from_local_data] Parsed from adress: '{}' '{}'",
                parts[1], parts[0]
            );
            (parts[1].to_string(), parts[0].to_string())
        } else {
            eprintln!(
                "[Storage::from_local_data] Could not parse adress: '{}'",
                data.adress
            );
            (data.adress.clone(), String::new())
        }
    };

    let postal_code = data.postnummer.clone().unwrap_or_default();
    eprintln!(
        "[Storage::from_local_data] Extracted: street='{}', number='{}', postal='{}'",
        street, street_number, postal_code
    );

    // Attempt to reconstruct matched_entry from persisted data
    let matched_entry = if let (
        Some(tid),
        Some(dag),
    ) = (&data.tid, data.dag)
    {
        eprintln!(
            "[Storage::from_local_data] Found persisted match data: tid={}, dag={}, taxa={:?}",
            tid, dag, data.taxa
        );

        // Reconstruct DB entry from persisted fields
        // Use current year and month for timestamp reconstruction
        // (the actual restriction dates are in the parking database)
        use chrono::Utc;
        let now = Utc::now();
        let current_year = now.year();
        let current_month = now.month();

        match DB::from_params(DBParams {
            postnummer: data.postnummer.clone(),
            adress: format!("{} {}", street, street_number),
            gata: Some(street.clone()),
            gatunummer: Some(street_number.clone()),
            info: data.info.clone(),
            dag,
            tid: tid.clone(),
            taxa: data.taxa.clone(),
            antal_platser: data.antal_platser,
            typ_av_parkering: data.typ_av_parkering.clone(),
            year: current_year,
            month: current_month,
        }) {
            Some(db_entry) => {
                eprintln!(
                    "[Storage::from_local_data] ✅ Successfully reconstructed DB entry from persisted data"
                );
                Some(db_entry)
            }
            None => {
                eprintln!(
                    "[Storage::from_local_data] ⚠️ Failed to reconstruct DB entry from tid={}, dag={}",
                    tid, dag
                );
                None
            }
        }
    } else if data.valid {
        eprintln!(
            "[Storage::from_local_data] No persisted match data but address is valid, attempting re-match"
        );
        
        // Fallback: try to re-match against database
        // This handles legacy data that was saved before match persistence was fixed
        use crate::components::matching::match_address;
        match match_address(&street, &street_number, &postal_code) {
            crate::components::matching::MatchResult::Valid(db_entry) => {
                eprintln!(
                    "[Storage::from_local_data] ✅ Re-matched successfully via database lookup"
                );
                Some(*db_entry)
            }
            crate::components::matching::MatchResult::Invalid(err) => {
                eprintln!(
                    "[Storage::from_local_data] ⚠️ Re-match failed: {}",
                    err
                );
                None
            }
        }
    } else {
        eprintln!("[Storage::from_local_data] Address is invalid, no match data expected");
        None
    };

    let stored_address = StoredAddress {
        id,
        street,
        street_number,
        postal_code,
        valid: data.valid,
        active: data.active,
        matched_entry,
    };

    eprintln!(
        "[Storage::from_local_data] === END CONVERSION (matched={}) ===",
        stored_address.matched_entry.is_some()
    );
    stored_address
}

/// Load stored addresses from persistent storage (thread-safe)
///
/// Reads from local.parquet file. If missing, attempts to recover from backup.
/// If neither exists, creates empty files and returns empty vector.
///
/// This operation is synchronized with a Mutex to prevent data races.
///
/// # Returns
/// Vector of stored addresses, empty if none saved or storage unavailable
///
/// # Examples
/// ```no_run
/// let addresses = read_addresses_from_device();
/// println!("Loaded {} addresses", addresses.len());
/// for addr in &addresses {
///     if addr.active {
///         println!("Active: {} {}", addr.street, addr.street_number);
///     }
/// }
/// ```
pub fn read_addresses_from_device() -> Vec<StoredAddress> {
    let _lock = STORAGE_LOCK.lock().unwrap();

    #[cfg(target_os = "android")]
    {
        match load_from_parquet() {
            Ok(addresses) => {
                eprintln!(
                    "[Storage] Loaded {} addresses from parquet",
                    addresses.len()
                );
                addresses
            }
            Err(e) => {
                eprintln!("[Storage] Failed to load addresses: {}", e);
                Vec::new()
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock Storage] read_addresses_from_device (empty)");
        Vec::new()
    }
}

/// Write stored addresses to persistent storage (thread-safe)
///
/// Implements backup rotation:
/// 1. Delete old backup
/// 2. Rename current local.parquet to backup
/// 3. Write new local.parquet
///
/// This operation is synchronized with a Mutex to prevent data races.
///
/// # Arguments
/// * `addresses` - Slice of addresses to persist
///
/// # Returns
/// - `Ok(())` if successful
/// - `Err(message)` if save failed
///
/// # Examples
/// ```no_run
/// use amp_android::storage;
/// use amp_android::ui::StoredAddress;
///
/// let addresses = vec![
///     StoredAddress {
///         id: 1,
///         street: "Storgatan".to_string(),
///         street_number: "10".to_string(),
///         postal_code: "22100".to_string(),
///         valid: true,
///         active: true,
///         matched_entry: None,
///     },
/// ];
///
/// if let Err(e) = storage::write_addresses_to_device(&addresses) {
///     eprintln!("Failed to save: {}", e);
/// }
/// ```
pub fn write_addresses_to_device(addresses: &[StoredAddress]) -> Result<(), String> {
    let _lock = STORAGE_LOCK.lock().unwrap();

    #[cfg(target_os = "android")]
    {
        save_to_parquet(addresses)?;
        eprintln!("[Storage] Saved {} addresses to parquet", addresses.len());
        Ok(())
    }

    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock Storage] Would save {} addresses", addresses.len());
        Ok(())
    }
}

/// Clear all stored addresses (thread-safe)
///
/// Removes all saved addresses from persistent storage by writing empty files.
///
/// This operation is synchronized with a Mutex to prevent data races.
///
/// # Returns
/// - `Ok(())` if successful
/// - `Err(message)` if clear failed
#[allow(dead_code)]
pub fn clear_all_addresses() -> Result<(), String> {
    let _lock = STORAGE_LOCK.lock().unwrap();

    #[cfg(target_os = "android")]
    {
        write_addresses_to_device(&[])?;
        eprintln!("[Storage] Cleared all addresses");
        Ok(())
    }

    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock Storage] Would clear all addresses");
        Ok(())
    }
}

/// Get total number of stored addresses without loading them
///
/// Currently loads all data to count. Could be optimized to read only metadata.
#[allow(dead_code)]
pub fn count_stored_addresses() -> usize {
    read_addresses_from_device().len()
}

/// Load addresses from parquet file
///
/// Internal function that:
/// 1. Ensures storage files exist
/// 2. Opens and reads parquet file
/// 3. Filters out empty/dummy entries
/// 4. Converts LocalData to StoredAddress
/// 5. Reconstructs matched_entry from persisted data (no re-matching needed)
#[cfg(target_os = "android")]
fn load_from_parquet() -> Result<Vec<StoredAddress>, String> {
    eprintln!("[Storage::load_from_parquet] Starting load operation");

    ensure_storage_files()?;

    let local_path = get_local_parquet_path()?;
    eprintln!(
        "[Storage::load_from_parquet] Opening file: {:?}",
        local_path
    );

    let file = File::open(&local_path).map_err(|e| {
        format!(
            "[Storage] Failed to open parquet file {:?}: {}",
            local_path, e
        )
    })?;

    eprintln!("[Storage::load_from_parquet] File opened successfully");

    let local_data = read_local_parquet(file).map_err(|e| {
        format!(
            "[Storage] Failed to read parquet data from {:?}: {}",
            local_path, e
        )
    })?;

    eprintln!(
        "[Storage::load_from_parquet] Read {} LocalData entries from parquet",
        local_data.len(),
    );

    let addresses: Vec<StoredAddress> = local_data
        .into_iter()
        .enumerate()
        .filter(|(idx, data)| {
            let keep = !data.adress.is_empty();
            if !keep {
                eprintln!(
                    "[Storage::load_from_parquet] Filtering out empty address at index {}",
                    idx,
                );
            }
            keep
        })
        .map(|(idx, data)| {
            eprintln!(
                "[Storage::load_from_parquet] Converting entry {}: adress='{}', valid={}, active={}, has_tid={}, has_taxa={}",
                idx,
                data.adress,
                data.valid,
                data.active,
                data.tid.is_some(),
                data.taxa.is_some(),
            );
            from_local_data(data, idx)
        })
        .collect();

    let matched_count = addresses
        .iter()
        .filter(|a| a.matched_entry.is_some())
        .count();

    eprintln!(
        "[Storage::load_from_parquet] Successfully loaded {} addresses ({} with matched_entry)",
        addresses.len(),
        matched_count,
    );

    Ok(addresses)
}

/// Save addresses to parquet file with backup rotation
///
/// Internal function that:
/// 1. Converts StoredAddress to LocalData (persisting match data)
/// 2. Creates dummy entry if list is empty (maintains valid schema)
/// 3. Builds parquet buffer
/// 4. Rotates backups (delete old → rename current → write new)
/// 5. Writes new parquet file
#[cfg(target_os = "android")]
fn save_to_parquet(addresses: &[StoredAddress]) -> Result<(), String> {
    eprintln!(
        "[Storage::save_to_parquet] Starting save operation for {} addresses",
        addresses.len(),
    );

    let local_path = get_local_parquet_path()?;
    let backup_path = get_backup_parquet_path()?;

    eprintln!("[Storage::save_to_parquet] Converting StoredAddress to LocalData");
    let local_data: Vec<LocalData> = addresses
        .iter()
        .enumerate()
        .map(|(idx, addr)| {
            eprintln!(
                "[Storage::save_to_parquet] Converting address {}: {} {}, postal_code={}, valid={}, active={}, has_match={}",
                idx,
                addr.street,
                addr.street_number,
                addr.postal_code,
                addr.valid,
                addr.active,
                addr.matched_entry.is_some(),
            );
            to_local_data(addr)
        })
        .collect();

    let data_to_write = if local_data.is_empty() {
        eprintln!("[Storage::save_to_parquet] No data to write, creating empty placeholder",);
        vec![LocalData {
            valid: false,
            active: false,
            postnummer: None,
            adress: String::new(),
            gata: None,
            gatunummer: None,
            info: None,
            tid: None,
            dag: None,
            taxa: None,
            antal_platser: None,
            typ_av_parkering: None,
        }]
    } else {
        local_data
    };

    eprintln!(
        "[Storage::save_to_parquet] Building parquet buffer for {} entries",
        data_to_write.len(),
    );
    let buffer = build_local_parquet(data_to_write)
        .map_err(|e| format!("[Storage] Failed to build parquet: {}", e))?;

    eprintln!(
        "[Storage::save_to_parquet] Built parquet buffer of {} bytes",
        buffer.len(),
    );

    // Backup rotation
    if backup_path.exists() {
        eprintln!(
            "[Storage::save_to_parquet] Deleting old backup: {:?}",
            backup_path
        );
        fs::remove_file(&backup_path).map_err(|e| {
            format!(
                "[Storage] Failed to delete old backup {:?}: {}",
                backup_path, e
            )
        })?;
    }

    if local_path.exists() {
        eprintln!(
            "[Storage::save_to_parquet] Renaming {:?} to {:?}",
            local_path, backup_path,
        );
        fs::rename(&local_path, &backup_path).map_err(|e| {
            format!(
                "[Storage] Failed to create backup (rename {:?} → {:?}): {}",
                local_path, backup_path, e,
            )
        })?;
    }

    eprintln!(
        "[Storage::save_to_parquet] Writing new file to {:?}",
        local_path
    );
    fs::write(&local_path, buffer).map_err(|e| {
        format!(
            "[Storage] Failed to write parquet file to {:?}: {}",
            local_path, e
        )
    })?;

    eprintln!(
        "[Storage::save_to_parquet] ✅ Successfully wrote {} addresses to {:?} (backup created)",
        addresses.len(),
        local_path,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    /// Test that StoredAddress → LocalData → StoredAddress preserves core fields
    #[test]
    #[cfg(target_os = "android")]
    fn test_to_from_local_data_roundtrip() {
        let original = StoredAddress {
            id: 1,
            street: "Storgatan".to_string(),
            street_number: "10".to_string(),
            postal_code: "22100".to_string(),
            valid: true,
            active: true,
            matched_entry: None,
        };

        let local_data = to_local_data(&original);
        let restored = from_local_data(local_data, 1);

        assert_eq!(original.street, restored.street);
        assert_eq!(original.street_number, restored.street_number);
        assert_eq!(original.postal_code, restored.postal_code);
        assert_eq!(original.valid, restored.valid);
        assert_eq!(original.active, restored.active);
    }

    /// Test that addresses with complex numbers (letters, suffixes) roundtrip correctly
    #[test]
    #[cfg(target_os = "android")]
    fn test_complex_street_number_roundtrip() {
        let test_cases = vec![
            ("Kornettsgatan", "18C"),
            ("Storgatan", "5"),
            ("Parkgatan", "12A"),
            ("Vägen", "100B"),
        ];

        for (street, number) in test_cases {
            let original = StoredAddress {
                id: 1,
                street: street.to_string(),
                street_number: number.to_string(),
                postal_code: "21438".to_string(),
                valid: true,
                active: false,
                matched_entry: None,
            };

            let local_data = to_local_data(&original);
            let restored = from_local_data(local_data, 1);

            assert_eq!(
                original.street, restored.street,
                "Street mismatch for {} {}",
                street, number,
            );
            assert_eq!(
                original.street_number, restored.street_number,
                "Street number mismatch for {} {}",
                street, number,
            );
        }
    }

    /// Test complete save → load roundtrip with multiple addresses
    #[test]
    #[cfg(target_os = "android")]
    fn test_storage_parquet_roundtrip() {
        let _ = clear_all_addresses();

        let addresses = vec![
            StoredAddress {
                id: 1,
                street: "Kornettsgatan".to_string(),
                street_number: "18C".to_string(),
                postal_code: "21438".to_string(),
                valid: true,
                active: false,
                matched_entry: None,
            },
            StoredAddress {
                id: 2,
                street: "Storgatan".to_string(),
                street_number: "5".to_string(),
                postal_code: "22100".to_string(),
                valid: true,
                active: true,
                matched_entry: None,
            },
        ];

        let save_result = write_addresses_to_device(&addresses);
        assert!(
            save_result.is_ok(),
            "Save should succeed: {:?}",
            save_result
        );

        let loaded = read_addresses_from_device();
        assert_eq!(loaded.len(), 2, "Should load 2 addresses");

        assert_eq!(loaded[0].street, "Kornettsgatan");
        assert_eq!(loaded[0].street_number, "18C");
        assert_eq!(loaded[0].postal_code, "21438");
        assert_eq!(loaded[0].valid, true);
        assert_eq!(loaded[0].active, false);

        assert_eq!(loaded[1].street, "Storgatan");
        assert_eq!(loaded[1].street_number, "5");
        assert_eq!(loaded[1].postal_code, "22100");
        assert_eq!(loaded[1].valid, true);
        assert_eq!(loaded[1].active, true);
    }

    /// Test that empty storage files can be created and read without errors
    #[test]
    #[cfg(target_os = "android")]
    fn test_empty_storage_files() {
        let clear_result = clear_all_addresses();
        assert!(
            clear_result.is_ok(),
            "Clear should succeed: {:?}",
            clear_result
        );

        let addresses = read_addresses_from_device();
        assert_eq!(addresses.len(), 0, "Should return empty vector after clear");

        let local_path = get_local_parquet_path().expect("Should get local path");
        let backup_path = get_backup_parquet_path().expect("Should get backup path");

        assert!(
            local_path.exists(),
            "Local parquet should exist after clear"
        );
        assert!(
            backup_path.exists(),
            "Backup parquet should exist after clear"
        );
    }

    /// Test that saving/loading a single address works correctly
    #[test]
    #[cfg(target_os = "android")]
    fn test_single_address() {
        let _ = clear_all_addresses();

        let address = vec![StoredAddress {
            id: 42,
            street: "Testgatan".to_string(),
            street_number: "7".to_string(),
            postal_code: "12345".to_string(),
            valid: false,
            active: true,
            matched_entry: None,
        }];

        let save_result = write_addresses_to_device(&address);
        assert!(save_result.is_ok(), "Save single address should succeed");

        let loaded = read_addresses_from_device();
        assert_eq!(loaded.len(), 1, "Should load 1 address");
        assert_eq!(loaded[0].street, "Testgatan");
        assert_eq!(loaded[0].street_number, "7");
        assert_eq!(loaded[0].postal_code, "12345");
        assert_eq!(loaded[0].valid, false);
        assert_eq!(loaded[0].active, true);
    }

    /// Test that matched_entry data is persisted and restored correctly
    #[test]
    #[cfg(target_os = "android")]
    fn test_matched_entry_persistence() {
        use amp_core::structs::DB;

        let _ = clear_all_addresses();

        // Create a DB entry with known data
        let db_entry = DB::from_dag_tid(
            Some("21438".to_string()),
            "Kornettsgatan 18C".to_string(),
            Some("Kornettsgatan".to_string()),
            Some("18C".to_string()),
            Some("Parkering förbjuden".to_string()),
            15,
            "0800-1200",
            Some("Taxa C".to_string()),
            Some(26),
            Some("Längsgående 6".to_string()),
            2024,
            1,
        )
        .expect("Should create DB entry");

        let original = StoredAddress {
            id: 1,
            street: "Kornettsgatan".to_string(),
            street_number: "18C".to_string(),
            postal_code: "21438".to_string(),
            valid: true,
            active: true,
            matched_entry: Some(db_entry.clone()),
        };

        // Save to storage
        let save_result = write_addresses_to_device(&[original.clone()]);
        assert!(save_result.is_ok(), "Save should succeed");

        // Load from storage
        let loaded = read_addresses_from_device();
        assert_eq!(loaded.len(), 1, "Should load 1 address");

        let restored = &loaded[0];

        // Check basic fields
        assert_eq!(original.street, restored.street);
        assert_eq!(original.street_number, restored.street_number);
        assert_eq!(original.postal_code, restored.postal_code);
        assert_eq!(original.valid, restored.valid);
        assert_eq!(original.active, restored.active);

        // Check matched_entry was restored
        assert!(
            restored.matched_entry.is_some(),
            "matched_entry should be restored"
        );

        let restored_entry = restored.matched_entry.as_ref().unwrap();

        // Verify key match data fields
        assert_eq!(
            db_entry.taxa, restored_entry.taxa,
            "Taxa should be preserved"
        );
        assert_eq!(
            db_entry.antal_platser, restored_entry.antal_platser,
            "Antal platser should be preserved"
        );
        assert_eq!(
            db_entry.typ_av_parkering, restored_entry.typ_av_parkering,
            "Typ av parkering should be preserved"
        );
        assert_eq!(
            db_entry.info, restored_entry.info,
            "Info should be preserved"
        );

        // Verify address fields in DB entry
        assert_eq!(
            db_entry.gata, restored_entry.gata,
            "Gata should be preserved"
        );
        assert_eq!(
            db_entry.gatunummer, restored_entry.gatunummer,
            "Gatunummer should be preserved"
        );
    }

    /// Test that multiple addresses with mixed match states persist correctly
    #[test]
    #[cfg(target_os = "android")]
    fn test_mixed_matched_entries() {
        use amp_core::structs::DB;

        let _ = clear_all_addresses();

        let db_entry = DB::from_dag_tid(
            Some("22100".to_string()),
            "Storgatan 10".to_string(),
            Some("Storgatan".to_string()),
            Some("10".to_string()),
            None,
            20,
            "1200-1600",
            Some("Taxa A".to_string()),
            Some(50),
            Some("Längsgående 8".to_string()),
            2024,
            3,
        )
        .expect("Should create DB entry");

        let addresses = vec![
            // Address with match
            StoredAddress {
                id: 1,
                street: "Storgatan".to_string(),
                street_number: "10".to_string(),
                postal_code: "22100".to_string(),
                valid: true,
                active: true,
                matched_entry: Some(db_entry),
            },
            // Address without match
            StoredAddress {
                id: 2,
                street: "Okänd Gata".to_string(),
                street_number: "99".to_string(),
                postal_code: "99999".to_string(),
                valid: false,
                active: false,
                matched_entry: None,
            },
        ];

        let save_result = write_addresses_to_device(&addresses);
        assert!(save_result.is_ok(), "Save should succeed");

        let loaded = read_addresses_from_device();
        assert_eq!(loaded.len(), 2, "Should load 2 addresses");

        // First address should have matched_entry
        assert!(
            loaded[0].matched_entry.is_some(),
            "First address should have match data"
        );
        assert_eq!(loaded[0].valid, true);

        // Second address should not have matched_entry
        assert!(
            loaded[1].matched_entry.is_none(),
            "Second address should not have match data"
        );
        assert_eq!(loaded[1].valid, false);
    }
}
