//! iOS platform bridge for native functionality
//!
//! Provides access to iOS-specific features like GPS location, permissions,
//! notification system, and other system services using objc2.
//!
//! # Platform Support
//! - **iOS**: Stub implementation (full objc2 wiring pending Xcode build)
//! - **Other platforms**: Mock implementation for testing
//!
//! # Examples
//! ```no_run
//! use amp_ios::ios_bridge;
//!
//! // Read GPS location
//! if let Some((lat, lon)) = ios_bridge::read_device_gps_location() {
//!     println!("Location: {}, {}", lat, lon);
//! }
//!
//! // Initialize notifications
//! ios_bridge::initialize_notification_channels_jni();
//! ```
/// Request notification permission from user
///
/// For iOS 13+: Shows system permission dialog via UNUserNotificationCenter
/// For iOS <13: No-op (permission not required)
#[allow(dead_code)]
pub fn request_notification_permission_jni() {
    #[cfg(target_os = "ios")]
    {
        eprintln!("[iOS Bridge] request_notification_permission_jni: stub");
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] Notification permission request (no-op)");
    }
}
/// Initialize iOS notification channels
///
/// On iOS, notification categories replace Android's channels.
/// Uses UNUserNotificationCenter to register notification categories.
///
/// # Platform Behavior
/// - **iOS**: Stub — TODO implement with objc2-user-notifications
/// - **Other platforms**: Mock implementation logs only
pub fn initialize_notification_channels_jni() {
    #[cfg(target_os = "ios")]
    {
        eprintln!("[iOS Bridge] initialize_notification_channels_jni: stub");
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] Notification channels initialized (no-op on non-iOS)",);
    }
}
/// Send a notification via iOS UNUserNotificationCenter
///
/// # Arguments
/// * `channel_id` - One of: "amp_active", "amp_six_hours", "amp_one_day"
/// * `notification_id` - Unique ID for this notification (typically address ID)
/// * `title` - Notification title text
/// * `body` - Notification body/content text
///
/// # Platform Behavior
/// - **iOS**: Stub — TODO implement with objc2-user-notifications
/// - **Other platforms**: Mock implementation logs parameters
pub fn send_notification_jni(channel_id: &str, notification_id: i32, title: &str, body: &str) {
    #[cfg(target_os = "ios")]
    {
        eprintln!(
            "[iOS Bridge] send_notification_jni: channel={}, id={}, title='{}' (stub)",
            channel_id, notification_id, title,
        );
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!(
            "[Mock iOS Bridge] Would send notification: channel='{}', id={}, title='{}', body='{}'",
            channel_id, notification_id, title, body,
        );
    }
}
/// Start the background monitoring service
///
/// On iOS, background processing uses BGTaskScheduler instead of a foreground service.
///
/// # Platform Behavior
/// - **iOS**: Stub — TODO implement with objc2 BGTaskScheduler
/// - **Other platforms**: No-op
#[allow(dead_code)]
pub fn start_dormant_service_jni() {
    #[cfg(target_os = "ios")]
    {
        eprintln!("[iOS Bridge] start_dormant_service_jni: stub");
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] DormantService start (no-op on non-iOS)");
    }
}
/// Read device GPS location
///
/// Attempts to get the current device location using iOS CLLocationManager.
/// Requires location permissions to be granted.
///
/// # Returns
/// - `Some((latitude, longitude))` if location is available
/// - `None` if location unavailable, permissions denied, or on non-iOS platforms
///
/// # Platform Behavior
/// - **iOS**: Stub — TODO implement with objc2 CLLocationManager
/// - **Other platforms**: Returns None (mock for testing)
pub fn read_device_gps_location() -> Option<(f64, f64)> {
    #[cfg(target_os = "ios")]
    {
        eprintln!("[iOS Bridge] read_device_gps_location: stub");
        None
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] GPS location requested - platform not supported");
        None
    }
}
/// Get device model and manufacturer information
///
/// # Returns
/// String describing the device (e.g., "iPhone 15 Pro")
#[allow(dead_code)]
pub fn get_device_info() -> String {
    #[cfg(target_os = "ios")]
    {
        "Unknown iOS Device".to_string()
    }
    #[cfg(not(target_os = "ios"))]
    {
        "Mock Device (Testing)".to_string()
    }
}
/// Open a URL in the device's default browser.
///
/// # Platform Behavior
/// - **iOS**: Uses UIApplication.shared.open(url) via objc2
/// - **Other platforms**: No-op (logs the URL)
pub fn open_url(url: &str) {
    #[cfg(target_os = "ios")]
    {
        eprintln!(
            "[iOS Bridge] open_url: {} (TODO: wire up UIApplication)",
            url
        );
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] open_url: {}", url);
    }
}
/// Export a file to user-chosen location
///
/// On iOS, uses UIDocumentPickerViewController via the share sheet.
///
/// # Arguments
/// * `source_path` - Absolute path to the file to export
/// * `suggested_name` - Suggested file name for the save dialog
///
/// # Returns
/// - `Ok(())` if export succeeded
/// - `Err(message)` if export failed or was cancelled
///
/// # Platform Behavior
/// - **iOS**: Stub — TODO implement with objc2 UIDocumentPickerViewController
/// - **Other platforms**: Mock implementation (always errors)
pub fn export_file_jni(source_path: &str, suggested_name: &str) -> Result<(), String> {
    #[cfg(target_os = "ios")]
    {
        eprintln!(
            "[iOS Bridge] export_file_jni: source={}, name={} (stub)",
            source_path, suggested_name,
        );
        Err("Export not yet implemented for iOS".to_string())
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!(
            "[Mock iOS Bridge] export_file_jni: source={}, name={}",
            source_path, suggested_name,
        );
        Err("Export not supported on this platform".to_string())
    }
}
/// Import a file from user-chosen location
///
/// On iOS, uses UIDocumentPickerViewController.
///
/// # Returns
/// - `Ok(Some(path))` with temp file path if a file was selected
/// - `Ok(None)` if the user cancelled
/// - `Err(message)` if import failed
///
/// # Platform Behavior
/// - **iOS**: Stub — TODO implement with objc2 UIDocumentPickerViewController
/// - **Other platforms**: Mock implementation (always returns None)
pub fn import_file_jni() -> Result<Option<String>, String> {
    #[cfg(target_os = "ios")]
    {
        eprintln!("[iOS Bridge] import_file_jni: stub");
        Ok(None)
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] import_file_jni (no-op)");
        Ok(None)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_gps_location_non_ios() {
        let result = read_device_gps_location();
        assert_eq!(result, None);
    }
    #[test]
    fn test_device_info() {
        let info = get_device_info();
        assert!(!info.is_empty());
    }
    #[test]
    fn test_initialize_channels_no_panic() {
        initialize_notification_channels_jni();
    }
    #[test]
    fn test_send_notification_no_panic() {
        send_notification_jni("amp_active", 1, "Test Title", "Test Body");
    }
}
