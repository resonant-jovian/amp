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
use amp_core::structs::LocalData;
use std::fs::{self};
use std::path::PathBuf;
use std::sync::Mutex;
#[allow(unused_imports)]
use std::fs::File;
/// Thread-safe storage mutex to prevent concurrent access issues
static STORAGE_LOCK: Mutex<()> = Mutex::new(());

#[cfg(target_os = "android")]
const LOCAL_PARQUET_NAME: &str = "local.parquet";
#[cfg(target_os = "android")]
const BACKUP_PARQUET_NAME: &str = "local.parquet.backup";

/// Get the storage directory path for the Android app
///
/// Returns the app's internal data directory on Android.
/// On other platforms, uses current directory for testing.
#[cfg(target_os = "android")]
fn get_storage_dir() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))
}
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
#[cfg(target_os = "android")]
fn create_empty_parquet(path: &PathBuf) -> Result<(), String> {
    let _empty_data: Vec<LocalData> = Vec::new();
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
        .map_err(|e| format!("Failed to build empty parquet: {}", e))?;
    fs::write(path, buffer).map_err(|e| format!("Failed to write empty parquet: {}", e))?;
    eprintln!("[Storage] Created empty parquet at {:?}", path);
    Ok(())
}
/// Ensure storage files exist, create if necessary
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
            .map_err(|e| format!("Failed to duplicate backup: {}", e))?;
    } else if local_exists && !backup_exists {
        eprintln!("[Storage] backup missing, creating from local.parquet");
        fs::copy(&local_path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    Ok(())
}
/// Convert StoredAddress to LocalData for parquet storage
#[cfg(target_os = "android")]
fn to_local_data(addr: &StoredAddress) -> LocalData {
    let (dag, tid, info, taxa, antal_platser, typ_av_parkering) =
        if let Some(ref entry) = addr.matched_entry {
            (
                None,
                None,
                entry.info.clone(),
                entry.taxa.clone(),
                entry.antal_platser,
                entry.typ_av_parkering.clone(),
            )
        } else {
            (None, None, None, None, None, None)
        };
    LocalData {
        valid: addr.valid,
        active: addr.active,
        postnummer: Some(addr.postal_code.clone()),
        adress: format!("{} {}", addr.street, addr.street_number),
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
#[cfg(target_os = "android")]
fn from_local_data(data: LocalData, id: usize) -> StoredAddress {
    let (street, street_number) = if let Some(gata) = &data.gata {
        let street_number = data.gatunummer.clone().unwrap_or_default();
        (gata.clone(), street_number)
    } else {
        let parts: Vec<&str> = data.adress.rsplitn(2, ' ').collect();
        if parts.len() == 2 {
            (parts[1].to_string(), parts[0].to_string())
        } else {
            (data.adress.clone(), String::new())
        }
    };
    StoredAddress {
        id,
        street,
        street_number,
        postal_code: data.postnummer.unwrap_or_default(),
        valid: data.valid,
        active: data.active,
        matched_entry: None,
    }
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
/// Load addresses from parquet file
#[cfg(target_os = "android")]
fn load_from_parquet() -> Result<Vec<StoredAddress>, String> {
    ensure_storage_files()?;
    let local_path = get_local_parquet_path()?;
    let file =
        File::open(&local_path).map_err(|e| format!("Failed to open parquet file: {}", e))?;
    let local_data =
        read_local_parquet(file).map_err(|e| format!("Failed to read parquet data: {}", e))?;
    let addresses: Vec<StoredAddress> = local_data
        .into_iter()
        .enumerate()
        .filter(|(_, data)| !data.adress.is_empty())
        .map(|(idx, data)| from_local_data(data, idx))
        .collect();
    Ok(addresses)
}
/// Save addresses to parquet file with backup rotation
#[cfg(target_os = "android")]
fn save_to_parquet(addresses: &[StoredAddress]) -> Result<(), String> {
    let local_path = get_local_parquet_path()?;
    let backup_path = get_backup_parquet_path()?;
    let local_data: Vec<LocalData> = addresses.iter().map(to_local_data).collect();
    let data_to_write = if local_data.is_empty() {
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
    let buffer = build_local_parquet(data_to_write)
        .map_err(|e| format!("Failed to build parquet: {}", e))?;
    if backup_path.exists() {
        fs::remove_file(&backup_path).map_err(|e| format!("Failed to delete old backup: {}", e))?;
    }
    if local_path.exists() {
        fs::rename(&local_path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    fs::write(&local_path, buffer).map_err(|e| format!("Failed to write parquet file: {}", e))?;
    eprintln!(
        "[Storage] Wrote {} addresses to {:?} (backup created)",
        addresses.len(),
        local_path,
    );
    Ok(())
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
pub fn count_stored_addresses() -> usize {
    read_addresses_from_device().len()
}
#[cfg(test)]
mod tests {
    use super::*;
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
}
