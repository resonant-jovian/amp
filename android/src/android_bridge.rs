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
//! // Read GPS location
//! if let Some((lat, lon)) = android_bridge::read_device_gps_location() {
//!     println!("Location: {}, {}", lat, lon);
//! }
//! ```
/// Read device GPS location
///
/// Attempts to get the current device location using Android LocationManager.
/// Requires location permissions to be granted.
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
    fn test_device_info() {
        let info = get_device_info();
        assert!(!info.is_empty());
    }
}
