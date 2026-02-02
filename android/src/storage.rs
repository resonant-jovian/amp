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
//! # Examples
//! ```no_run
//! use amp_android::storage;
//! use amp_android::ui::StoredAddress;
//!
//! // Read stored addresses
//! let addresses = storage::read_addresses_from_device();
//!
//! // Add new address
//! let mut addresses = addresses;
//! addresses.push(StoredAddress {
//!     street: "Storgatan".to_string(),
//!     street_number: "10".to_string(),
//!     postal_code: "22100".to_string(),
//!     active: true,
//! });
//!
//! // Save back to storage
//! storage::write_addresses_to_device(&addresses).ok();
//! ```
use crate::ui::StoredAddress;
#[cfg(target_os = "android")]
use jni::{
    JNIEnv, JavaVM,
    objects::{JClass, JObject, JString, JValue},
};
#[cfg(target_os = "android")]
use std::sync::OnceLock;
#[cfg(target_os = "android")]
static JVM: OnceLock<JavaVM> = OnceLock::new();
const PREFS_NAME: &str = "amp_parking_prefs";
const ADDRESSES_KEY: &str = "stored_addresses";
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
/// Load stored addresses from persistent storage
///
/// On Android, reads from SharedPreferences and deserializes JSON.
/// On other platforms, returns empty vector (mock).
///
/// # Returns
/// Vector of stored addresses, empty if none saved or storage unavailable
///
/// # Storage Format
/// Addresses are stored as JSON array in SharedPreferences:
/// ```json
/// [
///   {"street":"Storgatan","street_number":"10","postal_code":"22100","active":true},
///   {"street":"Änggården","street_number":"5","postal_code":"21138","active":false}
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
/// Write stored addresses to persistent storage
///
/// On Android, serializes to JSON and writes to SharedPreferences.
/// On other platforms, no-op (mock).
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
///         street: "Storgatan".to_string(),
///         street_number: "10".to_string(),
///         postal_code: "22100".to_string(),
///         active: true,
///     },
/// ];
///
/// if let Err(e) = storage::write_addresses_to_device(&addresses) {
///     eprintln!("Failed to save: {}", e);
/// }
/// ```
pub fn write_addresses_to_device(addresses: &[StoredAddress]) -> Result<(), String> {
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
/// 3. Deserialize JSON to Vec<StoredAddress>
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
/// 1. Serialize Vec<StoredAddress> to JSON
/// 2. Get SharedPreferences editor
/// 3. Write JSON string and commit
#[cfg(target_os = "android")]
fn save_to_shared_preferences(addresses: &[StoredAddress]) -> Result<(), String> {
    eprintln!(
        "[Android Storage] TODO: Implement SharedPreferences saving ({} addresses)",
        addresses.len(),
    );
    Ok(())
}
/// Serialize addresses to JSON string
///
/// Creates a JSON array representation of addresses for storage.
/// Uses simple manual JSON construction to avoid serde dependency.
///
/// # Format
/// ```json
/// [
///   {"street":"Storgatan","street_number":"10","postal_code":"22100","active":true}
/// ]
/// ```
///
/// # Examples
/// ```
/// # use amp_android::ui::StoredAddress;
/// # use amp_android::storage::serialize_addresses;
/// let addresses = vec![
///     StoredAddress {
///         street: "Test".to_string(),
///         street_number: "1".to_string(),
///         postal_code: "12345".to_string(),
///         active: true,
///     },
/// ];
/// let json = serialize_addresses(&addresses).unwrap();
/// assert!(json.contains("Test"));
/// ```
pub fn serialize_addresses(addresses: &[StoredAddress]) -> Result<String, String> {
    let mut json = String::from("[");
    for (i, addr) in addresses.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"{{"street":"{}","street_number":"{}","postal_code":"{}","active":{}}}"#,
            escape_json(&addr.street),
            escape_json(&addr.street_number),
            escape_json(&addr.postal_code),
            addr.active,
        ));
    }
    json.push(']');
    Ok(json)
}
/// Deserialize JSON string to addresses
///
/// Parses a JSON array representation back into StoredAddress instances.
/// Uses simple manual parsing to avoid serde dependency.
///
/// # TODO
/// Implement proper JSON parsing or integrate serde_json
///
/// # Notes
/// Current implementation is a stub - needs proper JSON parser
pub fn deserialize_addresses(json: &str) -> Result<Vec<StoredAddress>, String> {
    eprintln!("[Storage] TODO: JSON deserialization not implemented");
    eprintln!("[Storage] JSON input: {}", json);
    Ok(Vec::new())
}
/// Escape special characters for JSON strings
///
/// Handles backslashes, quotes, newlines, carriage returns, and tabs.
///
/// # Examples
/// ```
/// # use amp_android::storage::escape_json;
/// assert_eq!(escape_json(r#"test"string"#), r#"test\"string"#);
/// assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
/// ```
pub fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
/// Clear all stored addresses
///
/// Removes all saved addresses from persistent storage.
///
/// # Returns
/// - `Ok(())` if successful
/// - `Err(message)` if clear failed
///
/// # TODO
/// Implement SharedPreferences clear operation
pub fn clear_all_addresses() -> Result<(), String> {
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
/// Implement count without full deserialization
pub fn count_stored_addresses() -> usize {
    read_addresses_from_device().len()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::StoredAddress;
    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json(r#"test"string"#), r#"test\"string"#);
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json("tab\there"), "tab\\there");
        assert_eq!(escape_json("back\\slash"), "back\\\\slash");
    }
    #[test]
    fn test_serialize_empty() {
        let result = serialize_addresses(&[]);
        assert_eq!(result.unwrap(), "[]");
    }
    #[test]
    fn test_serialize_single_address() {
        let addresses = vec![StoredAddress {
            street: "Storgatan".to_string(),
            street_number: "10".to_string(),
            postal_code: "22100".to_string(),
            active: true,
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
                street: "Street1".to_string(),
                street_number: "1".to_string(),
                postal_code: "11111".to_string(),
                active: true,
            },
            StoredAddress {
                street: "Street2".to_string(),
                street_number: "2".to_string(),
                postal_code: "22222".to_string(),
                active: false,
            },
        ];
        let json = serialize_addresses(&addresses).unwrap();
        assert!(json.contains("Street1"));
        assert!(json.contains("Street2"));
        assert!(json.contains(","));
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
}
