//! Android platform bridge for native functionality
//!
//! Provides access to Android-specific features like GPS location, permissions,
//! and system services using JNI (Java Native Interface).
//!
//! # Platform Support
//! - **Android**: Full native implementation using JNI
//! - **Other platforms**: Mock implementation for testing
//!
//! # Examples
//! ```no_run
//! use amp_android::android_bridge;
//!
//! // Request location permissions first
//! android_bridge::request_location_permission();
//!
//! // Then read GPS location
//! if let Some((lat, lon)) = android_bridge::read_device_gps_location() {
//!     println!("Location: {}, {}", lat, lon);
//! }
//! ```
#[cfg(target_os = "android")]
use jni::{
    JNIEnv,
    objects::{JObject, JValue},
};
#[cfg(target_os = "android")]
use std::sync::OnceLock;
#[cfg(target_os = "android")]
static JVM: OnceLock<jni::JavaVM> = OnceLock::new();
/// Initialize JVM reference for Android operations
///
/// Must be called during app startup before any Android bridge functions.
/// This stores the JavaVM reference for later use in location services.
///
/// # Arguments
/// * `env` - JNI environment reference from Android
///
/// # Platform
/// Only available on Android targets
#[cfg(target_os = "android")]
pub fn init_jvm(env: &JNIEnv) {
    if let Ok(vm) = env.get_java_vm() {
        let _ = JVM.set(vm);
        eprintln!("[Android Bridge] JVM initialized successfully");
    } else {
        eprintln!("[Android Bridge] Failed to get JavaVM reference");
    }
}
/// Read device GPS location
///
/// Attempts to get the current device location using Android LocationManager.
/// Requires location permissions to be granted via [`request_location_permission`].
///
/// # Returns
/// - `Some((latitude, longitude))` if location is available
/// - `None` if location unavailable, permissions denied, or on non-Android platforms
///
/// # Platform Behavior
/// - **Android**: Uses LocationManager to get last known location
/// - **Other platforms**: Returns None (mock for testing)
///
/// # Security
/// Requires `ACCESS_FINE_LOCATION` or `ACCESS_COARSE_LOCATION` permission
///
/// # Examples
/// ```no_run
/// if let Some((lat, lon)) = read_device_gps_location() {
///     println!("Current position: {}, {}", lat, lon);
/// } else {
///     eprintln!("Location not available");
/// }
/// ```
pub fn read_device_gps_location() -> Option<(f64, f64)> {
    #[cfg(target_os = "android")]
    {
        match get_android_location() {
            Ok(location) => {
                eprintln!("[Android Bridge] Got location: {:?}", location);
                Some(location)
            }
            Err(e) => {
                eprintln!("[Android Bridge] Location error: {}", e);
                None
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock Android Bridge] GPS location requested - platform not supported",);
        None
    }
}
/// Get Android location using JNI and LocationManager
///
/// # Returns
/// Result containing (latitude, longitude) or error message
///
/// # TODO
/// Implement full LocationManager integration:
/// 1. Get LocationManager system service
/// 2. Request last known location from GPS_PROVIDER
/// 3. Fall back to NETWORK_PROVIDER if GPS unavailable
/// 4. Handle location timeout and retries
#[cfg(target_os = "android")]
fn get_android_location() -> Result<(f64, f64), String> {
    Err(
        "Android location reading not fully implemented - requires LocationManager integration"
            .to_string(),
    )
}
/// Request location permissions from user
///
/// Triggers the Android permission request dialog for location access.
/// Should be called before attempting to read GPS location.
///
/// # Platform Behavior
/// - **Android**: Requests `ACCESS_FINE_LOCATION` permission
/// - **Other platforms**: No-op (mock)
///
/// # Permissions Requested
/// - `android.permission.ACCESS_FINE_LOCATION` (GPS location)
/// - Optionally `android.permission.ACCESS_COARSE_LOCATION` (network-based)
///
/// # TODO
/// Implement permission request using ActivityCompat:
/// 1. Check if permission already granted
/// 2. If not, call ActivityCompat.requestPermissions()
/// 3. Handle permission result in callback
#[cfg(target_os = "android")]
pub fn request_location_permission() {
    eprintln!(
        "[Android Bridge] Location permission request not implemented - add JNI call to ActivityCompat.requestPermissions",
    );
}
#[cfg(not(target_os = "android"))]
pub fn request_location_permission() {
    eprintln!("[Mock Android Bridge] Location permission request (no-op on non-Android)",);
}
/// Check if location permissions are granted
///
/// Verifies whether the app has been granted location access permissions.
///
/// # Returns
/// - `true` if permissions granted
/// - `false` if permissions denied or not yet requested
///
/// # Platform
/// Returns `false` on non-Android platforms
///
/// # TODO
/// Implement permission check using ContextCompat.checkSelfPermission
#[cfg(target_os = "android")]
pub fn has_location_permission() -> bool {
    false
}
#[cfg(not(target_os = "android"))]
pub fn has_location_permission() -> bool {
    eprintln!("[Mock Android Bridge] Permission check (always false on non-Android)");
    false
}
/// Get device model and manufacturer information
///
/// # Returns
/// String in format "Manufacturer Model" (e.g., "Samsung Galaxy S21")
///
/// # TODO
/// Implement using android.os.Build
#[allow(dead_code)]
pub fn get_device_info() -> String {
    #[cfg(target_os = "android")]
    {
        "Unknown Device".to_string()
    }
    #[cfg(not(target_os = "android"))]
    {
        "Mock Device (Testing)".to_string()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_gps_location_non_android() {
        let result = read_device_gps_location();
        assert_eq!(result, None);
    }
    #[test]
    fn test_has_permission_non_android() {
        let result = has_location_permission();
        assert!(!result);
    }
    #[test]
    fn test_device_info() {
        let info = get_device_info();
        assert!(!info.is_empty());
    }
}
