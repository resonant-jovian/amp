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
//! ios_bridge::initialize_notification_channels();
//! ```
/// Request notification permission from user
///
/// For iOS 13+: Shows system permission dialog via UNUserNotificationCenter
/// For iOS <13: No-op (permission not required)
#[allow(dead_code)]
pub fn request_notification_permission() {
    #[cfg(target_os = "ios")]
    {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        unsafe {
            let center_class = objc2::class!(UNUserNotificationCenter);
            let center: *mut AnyObject = msg_send![center_class, currentNotificationCenter];
            if center.is_null() {
                eprintln!("[iOS Bridge] request_notification_permission: center nil");
                return;
            }
            let options: u64 = 7;
            let _: () = msg_send![
                center, requestAuthorizationWithOptions : options completionHandler :
                std::ptr::null::< AnyObject > ()
            ];
            eprintln!("[iOS Bridge] Notification permission requested (async, handler not wired)",);
        }
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] Notification permission request (no-op)");
    }
}
/// Initialize iOS notification channels
///
/// On iOS, notification categories serve as notification groupings.
/// Uses UNUserNotificationCenter to register notification categories.
///
/// # Platform Behavior
/// - **iOS**: Stub — TODO implement with objc2-user-notifications
/// - **Other platforms**: Mock implementation logs only
pub fn initialize_notification_channels() {
    #[cfg(target_os = "ios")]
    {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use std::ffi::CString;
        unsafe {
            let center_class = objc2::class!(UNUserNotificationCenter);
            let center: *mut AnyObject = msg_send![center_class, currentNotificationCenter];
            if center.is_null() {
                eprintln!("[iOS Bridge] UNUserNotificationCenter unavailable");
                return;
            }
            let cat_class = objc2::class!(UNNotificationCategory);
            let mut categories: Vec<*mut AnyObject> = Vec::new();
            for id_str in &["amp_active", "amp_six_hours", "amp_one_day"] {
                let c_id = CString::new(*id_str).unwrap_or_default();
                let ns_id: *mut AnyObject = msg_send![
                    objc2::class!(NSString), stringWithUTF8String : c_id.as_ptr()
                ];
                let empty_array: *mut AnyObject = msg_send![objc2::class!(NSArray), array];
                let cat: *mut AnyObject = msg_send![
                    cat_class, categoryWithIdentifier : ns_id actions : empty_array
                    intentIdentifiers : empty_array options : 0u64
                ];
                if !cat.is_null() {
                    categories.push(cat);
                }
            }
            let ns_set_class = objc2::class!(NSSet);
            let cats_ptr = categories.as_ptr();
            let cats_count = categories.len() as u64;
            let ns_set: *mut AnyObject = msg_send![
                ns_set_class, setWithObjects : cats_ptr count : cats_count
            ];
            let _: () = msg_send![center, setNotificationCategories : ns_set];
            eprintln!("[iOS Bridge] UNUserNotificationCenter categories registered");
        }
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
pub fn send_notification(channel_id: &str, notification_id: i32, title: &str, body: &str) {
    #[cfg(target_os = "ios")]
    {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use std::ffi::CString;
        unsafe {
            let center_class = objc2::class!(UNUserNotificationCenter);
            let center: *mut AnyObject = msg_send![center_class, currentNotificationCenter];
            if center.is_null() {
                eprintln!("[iOS Bridge] send_notification: center nil");
                return;
            }
            let content_class = objc2::class!(UNMutableNotificationContent);
            let content: *mut AnyObject = msg_send![content_class, new];
            let c_title = CString::new(title).unwrap_or_default();
            let ns_title: *mut AnyObject = msg_send![
                objc2::class!(NSString), stringWithUTF8String : c_title.as_ptr()
            ];
            let _: () = msg_send![content, setTitle : ns_title];
            let c_body = CString::new(body).unwrap_or_default();
            let ns_body: *mut AnyObject = msg_send![
                objc2::class!(NSString), stringWithUTF8String : c_body.as_ptr()
            ];
            let _: () = msg_send![content, setBody : ns_body];
            let c_cat = CString::new(channel_id).unwrap_or_default();
            let ns_cat: *mut AnyObject = msg_send![
                objc2::class!(NSString), stringWithUTF8String : c_cat.as_ptr()
            ];
            let _: () = msg_send![content, setCategoryIdentifier : ns_cat];
            let id_str = format!("{}-{}", channel_id, notification_id);
            let c_id = CString::new(id_str).unwrap_or_default();
            let ns_id: *mut AnyObject = msg_send![
                objc2::class!(NSString), stringWithUTF8String : c_id.as_ptr()
            ];
            let req_class = objc2::class!(UNNotificationRequest);
            let request: *mut AnyObject = msg_send![
                req_class, requestWithIdentifier : ns_id content : content trigger :
                std::ptr::null::< AnyObject > ()
            ];
            let _: () = msg_send![
                center, addNotificationRequest : request withCompletionHandler :
                std::ptr::null::< AnyObject > ()
            ];
            eprintln!(
                "[iOS Bridge] Notification queued: channel={}, id={}",
                channel_id, notification_id,
            );
        }
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
pub fn start_dormant_service() {
    #[cfg(target_os = "ios")]
    {
        eprintln!("[iOS Bridge] start_dormant_service: stub (needs BGTaskScheduler + Info.plist)",);
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
        eprintln!("[iOS Bridge] read_device_gps_location: stub (needs CLLocationManager delegate)",);
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
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use std::ffi::CStr;
        unsafe {
            let device_class = objc2::class!(UIDevice);
            let device: *mut AnyObject = msg_send![device_class, currentDevice];
            if device.is_null() {
                return "Unknown iOS Device".to_string();
            }
            let model: *mut AnyObject = msg_send![device, model];
            if model.is_null() {
                return "Unknown iOS Device".to_string();
            }
            let bytes: *const std::os::raw::c_char = msg_send![model, UTF8String];
            if bytes.is_null() {
                return "Unknown iOS Device".to_string();
            }
            CStr::from_ptr(bytes)
                .to_str()
                .unwrap_or("Unknown iOS Device")
                .to_string()
        }
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
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use std::ffi::CString;
        unsafe {
            let ns_string_class = objc2::class!(NSString);
            let c_url = CString::new(url).unwrap_or_default();
            let ns_url_str: *mut AnyObject = msg_send![
                ns_string_class, stringWithUTF8String : c_url.as_ptr()
            ];
            let nsurl_class = objc2::class!(NSURL);
            let ns_url: *mut AnyObject = msg_send![
                nsurl_class, URLWithString : ns_url_str
            ];
            if ns_url.is_null() {
                eprintln!("[iOS Bridge] open_url: invalid URL: {}", url);
                return;
            }
            let app_class = objc2::class!(UIApplication);
            let shared_app: *mut AnyObject = msg_send![app_class, sharedApplication];
            if !shared_app.is_null() {
                let _: () = msg_send![shared_app, openURL : ns_url];
            }
        }
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
pub fn export_file(source_path: &str, suggested_name: &str) -> Result<(), String> {
    #[cfg(target_os = "ios")]
    {
        eprintln!(
            "[iOS Bridge] export_file: source={}, name={} (stub, needs UIViewController)",
            source_path, suggested_name,
        );
        Err("Export not yet implemented for iOS".to_string())
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!(
            "[Mock iOS Bridge] export_file: source={}, name={}",
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
pub fn import_file() -> Result<Option<String>, String> {
    #[cfg(target_os = "ios")]
    {
        eprintln!("[iOS Bridge] import_file: stub (needs UIViewController)");
        Ok(None)
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] import_file (no-op)");
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
        initialize_notification_channels();
    }
    #[test]
    fn test_send_notification_no_panic() {
        send_notification("amp_active", 1, "Test Title", "Test Body");
    }
}
