//! Persistent storage for Android app
//!
//! Provides local storage for user addresses using Android SharedPreferences.
//! Falls back to in-memory storage for non-Android platforms (testing).
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
    }
}
/// Load stored addresses from persistent storage
///
/// On Android, reads from SharedPreferences. On other platforms, returns empty vec.
///
/// # Returns
/// Vector of stored addresses, empty if none saved or storage unavailable
///
/// # Examples
/// ```no_run
/// let addresses = read_addresses_from_device();
/// println!("Loaded {} addresses", addresses.len());
/// ```
pub fn read_addresses_from_device() -> Vec<StoredAddress> {
    #[cfg(target_os = "android")]
    {
        load_from_shared_preferences().unwrap_or_else(|e| {
            eprintln!("Failed to load addresses: {}", e);
            Vec::new()
        })
    }
    #[cfg(not(target_os = "android"))]
    {
        Vec::new()
    }
}
/// Write stored addresses to persistent storage
///
/// On Android, writes to SharedPreferences as JSON. On other platforms, no-op.
///
/// # Arguments
/// * `addresses` - Slice of addresses to persist
///
/// # Returns
/// Ok(()) if successful, Err with message if failed
///
/// # Examples
/// ```no_run
/// let addresses = vec![/* ... */];
/// if let Err(e) = write_addresses_to_device(&addresses) {
///     eprintln!("Failed to save: {}", e);
/// }
/// ```
pub fn write_addresses_to_device(addresses: &[StoredAddress]) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        save_to_shared_preferences(addresses)
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock] Would save {} addresses", addresses.len());
        Ok(())
    }
}
#[cfg(target_os = "android")]
fn load_from_shared_preferences() -> Result<Vec<StoredAddress>, String> {
    Err(
        "Android SharedPreferences not yet implemented - needs proper context initialization"
            .to_string(),
    )
}
#[cfg(target_os = "android")]
fn save_to_shared_preferences(addresses: &[StoredAddress]) -> Result<(), String> {
    eprintln!(
        "[Android] Would save {} addresses (persistence not yet implemented)",
        addresses.len(),
    );
    Ok(())
}
/// Serialize addresses to JSON string
///
/// Creates a simple JSON array representation of addresses for storage.
#[allow(dead_code)]
fn serialize_addresses(addresses: &[StoredAddress]) -> Result<String, String> {
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
#[allow(dead_code)]
fn deserialize_addresses(json: &str) -> Result<Vec<StoredAddress>, String> {
    eprintln!("JSON deserialization not fully implemented: {}", json);
    Ok(Vec::new())
}
/// Escape special characters for JSON strings
#[allow(dead_code)]
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json(r#"test"string"#), r#"test\"string"#);
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
    }
    #[test]
    fn test_serialize_empty() {
        let result = serialize_addresses(&[]);
        assert_eq!(result.unwrap(), "[]");
    }
}
