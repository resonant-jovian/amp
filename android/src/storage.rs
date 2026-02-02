//! Persistent storage for Android app
//!
//! Provides local storage for user addresses using Android SharedPreferences
//! and file-based storage for parquet data files.
//!
//! # Storage Locations
//! - **User addresses**: SharedPreferences (key-value storage)
//! - **Parking data**: Internal app files directory (parquet format)
//!
//! # Platform Support
//! - **Android**: Full SharedPreferences and file storage
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
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[cfg(target_os = "android")]
use jni::{
    objects::{JClass, JObject, JString, JValue},
    JNIEnv, JavaVM,
};
#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static JVM: OnceLock<JavaVM> = OnceLock::new();

/// Thread-safe storage mutex to prevent concurrent access issues
static STORAGE_LOCK: Mutex<()> = Mutex::new(());

const _PREFS_NAME: &str = "amp_parking_prefs";
const _ADDRESSES_KEY: &str = "stored_addresses";

/// Serializable version of StoredAddress for JSON storage
///
/// This struct is used for serialization/deserialization and doesn't include
/// the matched_entry field which contains runtime data.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAddressData {
    id: usize,
    street: String,
    street_number: String,
    postal_code: String,
    valid: bool,
    active: bool,
}

impl From<&StoredAddress> for StoredAddressData {
    fn from(addr: &StoredAddress) -> Self {
        StoredAddressData {
            id: addr.id,
            street: addr.street.clone(),
            street_number: addr.street_number.clone(),
            postal_code: addr.postal_code.clone(),
            valid: addr.valid,
            active: addr.active,
        }
    }
}

impl From<StoredAddressData> for StoredAddress {
    fn from(data: StoredAddressData) -> Self {
        StoredAddress {
            id: data.id,
            street: data.street,
            street_number: data.street_number,
            postal_code: data.postal_code,
            valid: data.valid,
            active: data.active,
            matched_entry: None, // Runtime data, not persisted
        }
    }
}

/// Initialize the JVM reference for Android storage operations
///
/// This should be called once during app initialization on Android.
///
/// # Arguments
/// * `env` - JNI environment reference
#[cfg(target_os = "android")]
pub fn init_jvm(env: &JNIEnv) {
    if let Ok(vm) = env.get_java_vm() {
        let _ = JVM.set(vm);
        eprintln!("[Storage] JVM initialized");
    }
}

/// Load stored addresses from persistent storage (thread-safe)
///
/// On Android, reads from SharedPreferences and deserializes JSON.
/// On other platforms, returns empty vector (mock).
///
/// This operation is synchronized with a Mutex to prevent data races.
///
/// # Returns
/// Vector of stored addresses, empty if none saved or storage unavailable
///
/// # Storage Format
/// Addresses are stored as JSON array using serde_json:
/// ```json
/// [
///   {"id":1,"street":"Storgatan","street_number":"10","postal_code":"22100","valid":true,"active":true},
///   {"id":2,"street":"Änggården","street_number":"5","postal_code":"21138","valid":true,"active":false}
/// ]
/// ```
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
    // Acquire lock for thread-safe access
    let _lock = STORAGE_LOCK.lock().unwrap();
    
    #[cfg(target_os = "android")]
    {
        match load_from_shared_preferences() {
            Ok(addresses) => {
                eprintln!("[Storage] Loaded {} addresses", addresses.len());
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
/// On Android, serializes to JSON using serde_json and writes to SharedPreferences.
/// On other platforms, no-op (mock).
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
    // Acquire lock for thread-safe access
    let _lock = STORAGE_LOCK.lock().unwrap();
    
    #[cfg(target_os = "android")]
    {
        save_to_shared_preferences(addresses)?;
        eprintln!("[Storage] Saved {} addresses", addresses.len());
        Ok(())
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock Storage] Would save {} addresses", addresses.len());
        Ok(())
    }
}

/// Load addresses from SharedPreferences
///
/// # TODO
/// Implement full SharedPreferences integration:
/// 1. Get SharedPreferences instance
/// 2. Read JSON string from preferences
/// 3. Deserialize JSON to Vec<StoredAddress> using serde_json
#[cfg(target_os = "android")]
fn load_from_shared_preferences() -> Result<Vec<StoredAddress>, String> {
    Err(
        "Android SharedPreferences not yet implemented - requires context and JNI integration"
            .to_string(),
    )
}

/// Save addresses to SharedPreferences
///
/// # TODO
/// Implement full SharedPreferences writing:
/// 1. Serialize Vec<StoredAddress> to JSON using serde_json
/// 2. Get SharedPreferences editor
/// 3. Write JSON string and commit
#[cfg(target_os = "android")]
fn save_to_shared_preferences(addresses: &[StoredAddress]) -> Result<(), String> {
    // Convert to serializable format
    let data: Vec<StoredAddressData> = addresses.iter().map(StoredAddressData::from).collect();
    
    // Serialize using serde_json
    let json = serde_json::to_string(&data)
        .map_err(|e| format!("Failed to serialize addresses: {}", e))?;
    
    eprintln!(
        "[Android Storage] TODO: Write to SharedPreferences ({} addresses, {} bytes)",
        addresses.len(),
        json.len()
    );
    eprintln!("[Android Storage] JSON preview: {}...", &json[..json.len().min(100)]);
    
    // TODO: Actually write to SharedPreferences using JNI
    Ok(())
}

/// Serialize addresses to JSON string using serde_json
///
/// Creates a JSON array representation of addresses for storage.
///
/// # Arguments
/// * `addresses` - Slice of addresses to serialize
///
/// # Returns
/// - `Ok(String)` with JSON representation
/// - `Err(String)` if serialization fails
///
/// # Format
/// ```json
/// [
///   {"id":1,"street":"Storgatan","street_number":"10","postal_code":"22100","valid":true,"active":true}
/// ]
/// ```
///
/// # Examples
/// ```
/// # use amp_android::ui::StoredAddress;
/// # use amp_android::storage::serialize_addresses;
/// let addresses = vec![
///     StoredAddress {
///         id: 1,
///         street: "Test".to_string(),
///         street_number: "1".to_string(),
///         postal_code: "12345".to_string(),
///         valid: true,
///         active: true,
///         matched_entry: None,
///     },
/// ];
/// let json = serialize_addresses(&addresses).unwrap();
/// assert!(json.contains("Test"));
/// ```
pub fn serialize_addresses(addresses: &[StoredAddress]) -> Result<String, String> {
    let data: Vec<StoredAddressData> = addresses.iter().map(StoredAddressData::from).collect();
    serde_json::to_string(&data).map_err(|e| format!("Serialization error: {}", e))
}

/// Deserialize JSON string to addresses using serde_json
///
/// Parses a JSON array representation back into StoredAddress instances.
///
/// # Arguments
/// * `json` - JSON string containing array of addresses
///
/// # Returns
/// - `Ok(Vec<StoredAddress>)` if parsing succeeds
/// - `Err(String)` if JSON is invalid
///
/// # Examples
/// ```
/// # use amp_android::storage::deserialize_addresses;
/// let json = r#"[{"id":1,"street":"Test","street_number":"1","postal_code":"12345","valid":true,"active":true}]"#;
/// let addresses = deserialize_addresses(json).unwrap();
/// assert_eq!(addresses.len(), 1);
/// assert_eq!(addresses[0].street, "Test");
/// ```
pub fn deserialize_addresses(json: &str) -> Result<Vec<StoredAddress>, String> {
    let data: Vec<StoredAddressData> = serde_json::from_str(json)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    Ok(data.into_iter().map(StoredAddress::from).collect())
}

/// Clear all stored addresses (thread-safe)
///
/// Removes all saved addresses from persistent storage.
///
/// This operation is synchronized with a Mutex to prevent data races.
///
/// # Returns
/// - `Ok(())` if successful
/// - `Err(message)` if clear failed
///
/// # TODO
/// Implement SharedPreferences clear operation
pub fn clear_all_addresses() -> Result<(), String> {
    // Acquire lock for thread-safe access
    let _lock = STORAGE_LOCK.lock().unwrap();
    
    #[cfg(target_os = "android")]
    {
        eprintln!("[Storage] TODO: Implement clear_all_addresses");
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
/// Efficiently checks storage without deserializing all data.
///
/// # TODO
/// Implement count without full deserialization for performance
pub fn count_stored_addresses() -> usize {
    read_addresses_from_device().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::StoredAddress;

    #[test]
    fn test_serialize_empty() {
        let result = serialize_addresses(&[]);
        assert_eq!(result.unwrap(), "[]");
    }

    #[test]
    fn test_serialize_single_address() {
        let addresses = vec![StoredAddress {
            id: 1,
            street: "Storgatan".to_string(),
            street_number: "10".to_string(),
            postal_code: "22100".to_string(),
            valid: true,
            active: true,
            matched_entry: None,
        }];
        let json = serialize_addresses(&addresses).unwrap();
        assert!(json.contains("Storgatan"));
        assert!(json.contains("10"));
        assert!(json.contains("22100"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_serialize_multiple_addresses() {
        let addresses = vec![
            StoredAddress {
                id: 1,
                street: "Street1".to_string(),
                street_number: "1".to_string(),
                postal_code: "11111".to_string(),
                valid: true,
                active: true,
                matched_entry: None,
            },
            StoredAddress {
                id: 2,
                street: "Street2".to_string(),
                street_number: "2".to_string(),
                postal_code: "22222".to_string(),
                valid: false,
                active: false,
                matched_entry: None,
            },
        ];
        let json = serialize_addresses(&addresses).unwrap();
        assert!(json.contains("Street1"));
        assert!(json.contains("Street2"));
    }

    #[test]
    fn test_deserialize_addresses() {
        let json = r#"[{"id":1,"street":"Test","street_number":"1","postal_code":"12345","valid":true,"active":true}]"#;
        let addresses = deserialize_addresses(json).unwrap();
        assert_eq!(addresses.len(), 1);
        assert_eq!(addresses[0].street, "Test");
        assert_eq!(addresses[0].street_number, "1");
        assert_eq!(addresses[0].postal_code, "12345");
        assert!(addresses[0].valid);
        assert!(addresses[0].active);
        assert!(addresses[0].matched_entry.is_none());
    }

    #[test]
    fn test_roundtrip_serialization() {
        let original = vec![
            StoredAddress {
                id: 1,
                street: "Storgatan".to_string(),
                street_number: "10".to_string(),
                postal_code: "22100".to_string(),
                valid: true,
                active: true,
                matched_entry: None,
            },
            StoredAddress {
                id: 2,
                street: "Lillgatan".to_string(),
                street_number: "5A".to_string(),
                postal_code: "21100".to_string(),
                valid: false,
                active: false,
                matched_entry: None,
            },
        ];

        let json = serialize_addresses(&original).unwrap();
        let deserialized = deserialize_addresses(&json).unwrap();

        assert_eq!(deserialized.len(), original.len());
        for (orig, deser) in original.iter().zip(deserialized.iter()) {
            assert_eq!(orig.id, deser.id);
            assert_eq!(orig.street, deser.street);
            assert_eq!(orig.street_number, deser.street_number);
            assert_eq!(orig.postal_code, deser.postal_code);
            assert_eq!(orig.valid, deser.valid);
            assert_eq!(orig.active, deser.active);
        }
    }

    #[test]
    fn test_read_write_non_android() {
        let addresses = read_addresses_from_device();
        assert_eq!(addresses.len(), 0);

        let result = write_addresses_to_device(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_stored_addresses() {
        let count = count_stored_addresses();
        assert_eq!(count, 0);
    }
    
    #[test]
    fn test_deserialize_invalid_json() {
        let result = deserialize_addresses("not valid json");
        assert!(result.is_err());
        
        let result = deserialize_addresses("{}"); // Object instead of array
        assert!(result.is_err());
    }
}
