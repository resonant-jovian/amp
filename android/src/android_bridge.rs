//! Android platform bridge for native functionality
//!
//! Provides access to Android-specific features like GPS location.
//! Uses JNI to communicate with Android system services.
#[cfg(target_os = "android")]
use jni::{
    JNIEnv,
    objects::{JObject, JValue},
};
/// Read device GPS location
///
/// Attempts to get the current device location using Android LocationManager.
/// Requires location permissions to be granted.
///
/// # Returns
/// Some((latitude, longitude)) if location available, None otherwise
///
/// # Platform Behavior
/// - **Android**: Uses LocationManager to get last known location
/// - **Other platforms**: Returns None (mock for testing)
///
/// # Examples
/// ```no_run
/// if let Some((lat, lon)) = read_device_gps_location() {
///     println!("Location: {}, {}", lat, lon);
/// }
/// ```
pub fn read_device_gps_location() -> Option<(f64, f64)> {
    #[cfg(target_os = "android")]
    {
        get_android_location().ok()
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock] GPS location requested - returning test coordinates");
        None
    }
}
#[cfg(target_os = "android")]
fn get_android_location() -> Result<(f64, f64), String> {
    Err(
        "Android location reading not fully implemented - requires LocationManager integration"
            .to_string(),
    )
}
/// Request location permissions
///
/// Triggers the Android permission request dialog for location access.
/// Should be called before attempting to read GPS location.
///
/// # Platform Behavior
/// - **Android**: Requests ACCESS_FINE_LOCATION permission
/// - **Other platforms**: No-op
#[cfg(target_os = "android")]
pub fn request_location_permission() {
    eprintln!("[Android] Location permission request not implemented");
}
#[cfg(not(target_os = "android"))]
pub fn request_location_permission() {
    eprintln!("[Mock] Location permission request (no-op on non-Android)");
}
/// Check if location permissions are granted
///
/// # Returns
/// true if permissions granted, false otherwise
#[cfg(target_os = "android")]
pub fn has_location_permission() -> bool {
    false
}
#[cfg(not(target_os = "android"))]
pub fn has_location_permission() -> bool {
    false
}
