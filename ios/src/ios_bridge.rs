//! iOS platform bridge for native functionality
//!
//! Provides access to iOS-specific features like GPS location, permissions,
//! notification system, file import/export, and background processing using objc2.
//!
//! # Platform Support
//! - **iOS**: Native implementation using objc2 raw msg_send!
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

// ============================================================
// iOS-only types, helpers, and FFI declarations
// ============================================================

/// CLLocationCoordinate2D matching the CoreLocation C struct layout
#[cfg(target_os = "ios")]
#[repr(C)]
#[derive(Copy, Clone)]
struct CLLocationCoordinate2D {
    latitude: f64,
    longitude: f64,
}

#[cfg(target_os = "ios")]
unsafe impl objc2::encode::Encode for CLLocationCoordinate2D {
    const ENCODING: objc2::encode::Encoding = objc2::encode::Encoding::Struct(
        "CLLocationCoordinate2D",
        &[
            objc2::encode::Encoding::Double,
            objc2::encode::Encoding::Double,
        ],
    );
}

#[cfg(target_os = "ios")]
unsafe impl objc2::encode::RefEncode for CLLocationCoordinate2D {
    const ENCODING_REF: objc2::encode::Encoding =
        objc2::encode::Encoding::Pointer(&<Self as objc2::encode::Encode>::ENCODING);
}

/// Create an NSString from a Rust &str
#[cfg(target_os = "ios")]
unsafe fn make_nsstring(s: &str) -> *mut objc2::runtime::AnyObject {
    use objc2::msg_send;
    let c_str = std::ffi::CString::new(s).unwrap_or_default();
    msg_send![objc2::class!(NSString), stringWithUTF8String: c_str.as_ptr()]
}

/// Get the root UIViewController from the key window (iOS 13+ scene API with fallback)
#[cfg(target_os = "ios")]
unsafe fn get_root_view_controller() -> *mut objc2::runtime::AnyObject {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let app: *mut AnyObject = msg_send![objc2::class!(UIApplication), sharedApplication];
    if app.is_null() {
        return std::ptr::null_mut();
    }

    // iOS 13+: connectedScenes → UIWindowScene → windows → keyWindow
    let scenes: *mut AnyObject = msg_send![app, connectedScenes];
    if !scenes.is_null() {
        let all_objects: *mut AnyObject = msg_send![scenes, allObjects];
        if !all_objects.is_null() {
            let count: u64 = msg_send![all_objects, count];
            for i in 0..count {
                let scene: *mut AnyObject = msg_send![all_objects, objectAtIndex: i];
                if scene.is_null() {
                    continue;
                }
                let windows: *mut AnyObject = msg_send![scene, windows];
                if windows.is_null() {
                    continue;
                }
                let win_count: u64 = msg_send![windows, count];
                for j in 0..win_count {
                    let window: *mut AnyObject = msg_send![windows, objectAtIndex: j];
                    if window.is_null() {
                        continue;
                    }
                    let is_key: bool = msg_send![window, isKeyWindow];
                    if is_key {
                        let vc: *mut AnyObject = msg_send![window, rootViewController];
                        if !vc.is_null() {
                            return vc;
                        }
                    }
                }
            }
        }
    }

    // Fallback: deprecated keyWindow API (still works on all iOS versions)
    let key_window: *mut AnyObject = msg_send![app, keyWindow];
    if key_window.is_null() {
        return std::ptr::null_mut();
    }
    msg_send![key_window, rootViewController]
}

/// Schedule a background app refresh via BGTaskScheduler
#[cfg(target_os = "ios")]
#[allow(dead_code)]
unsafe fn schedule_bg_refresh() {
    unsafe {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;

        let scheduler: *mut AnyObject = msg_send![objc2::class!(BGTaskScheduler), sharedScheduler];
        if scheduler.is_null() {
            return;
        }

        let request_class = objc2::class!(BGAppRefreshTaskRequest);
        let task_id = make_nsstring("se.malmo.skaggbyran.amp.dormant");
        let request: *mut AnyObject = msg_send![request_class, alloc];
        let request: *mut AnyObject = msg_send![request, initWithIdentifier: task_id];

        // Schedule earliest 1 hour from now
        let date: *mut AnyObject =
            msg_send![objc2::class!(NSDate), dateWithTimeIntervalSinceNow: 3600.0f64];
        let _: () = msg_send![request, setEarliestBeginDate: date];

        let mut error: *mut AnyObject = std::ptr::null_mut();
        let success: bool = msg_send![scheduler, submitTaskRequest: request, error: &mut error];
        if success {
            eprintln!("[iOS Bridge] BGAppRefreshTaskRequest scheduled for ~1 hour");
        } else if !error.is_null() {
            let desc: *mut AnyObject = msg_send![error, localizedDescription];
            if !desc.is_null() {
                let bytes: *const std::os::raw::c_char = msg_send![desc, UTF8String];
                if !bytes.is_null() {
                    let msg = std::ffi::CStr::from_ptr(bytes)
                        .to_str()
                        .unwrap_or("unknown");
                    eprintln!("[iOS Bridge] BGTaskScheduler error: {}", msg);
                }
            }
        }
    }
}

// ============================================================
// Public API
// ============================================================

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
                center, requestAuthorizationWithOptions: options, completionHandler:
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
/// - **iOS**: Registers categories via UNUserNotificationCenter
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
                    cat_class, categoryWithIdentifier: ns_id, actions: empty_array,
                    intentIdentifiers: empty_array, options: 0u64
                ];
                if !cat.is_null() {
                    categories.push(cat);
                }
            }
            let ns_set_class = objc2::class!(NSSet);
            let cats_ptr = categories.as_ptr();
            let cats_count = categories.len() as u64;
            let ns_set: *mut AnyObject = msg_send![
                ns_set_class, setWithObjects: cats_ptr, count: cats_count
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
/// - **iOS**: Sends via UNUserNotificationCenter
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
                req_class, requestWithIdentifier: ns_id, content: content, trigger:
                std::ptr::null::< AnyObject > ()
            ];
            let _: () = msg_send![
                center, addNotificationRequest: request, withCompletionHandler:
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
/// Registers a BGTaskScheduler app refresh task for periodic dormant checks,
/// and starts a foreground timer thread as fallback. The BGTaskScheduler task
/// runs `dormant_hourly_check()` and sends resulting notifications.
///
/// # Platform Behavior
/// - **iOS**: BGTaskScheduler registration + foreground timer thread
/// - **Other platforms**: No-op
///
/// # Note
/// BGTaskScheduler requires `BGTaskSchedulerPermittedIdentifiers` in Info.plist
/// with the identifier `se.malmo.skaggbyran.amp.dormant`.
#[allow(dead_code)]
pub fn start_dormant_service() {
    #[cfg(target_os = "ios")]
    {
        // TODO: BGTaskScheduler requires registering a handler (block2) before
        // submitting task requests. Since block2 has encoding issues with *mut AnyObject,
        // we skip BGTaskScheduler for now and rely on the foreground timer thread.
        // When block2 encoding is resolved, add handler registration + schedule_bg_refresh().

        // Foreground timer thread: runs dormant checks hourly while app is active.
        // When iOS suspends the app, this thread freezes and resumes on foreground return.
        std::thread::spawn(|| {
            eprintln!("[iOS Bridge] Dormant timer thread started");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
                let storage_path = std::env::var("APP_FILES_DIR").unwrap_or_default();
                if storage_path.is_empty() {
                    continue;
                }
                eprintln!("[iOS Bridge] Running dormant hourly check (foreground timer)");
                let notifications = crate::components::dormant::dormant_hourly_check(&storage_path);
                for notif in &notifications {
                    send_notification(
                        &notif.channel_id,
                        notif.notification_id,
                        &notif.title,
                        &notif.body,
                    );
                }
            }
        });

        eprintln!("[iOS Bridge] Dormant service started");
    }
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[Mock iOS Bridge] DormantService start (no-op on non-iOS)");
    }
}

/// Read device GPS location via CLLocationManager
///
/// Uses CLLocationManager's `location` property to get the last known location.
/// This is a synchronous read of cached location data, similar to Android's
/// `getLastKnownLocation()`. Requires `NSLocationWhenInUseUsageDescription`
/// in Info.plist.
///
/// # Returns
/// - `Some((latitude, longitude))` if location is available
/// - `None` if location unavailable, permissions denied, or on non-iOS platforms
pub fn read_device_gps_location() -> Option<(f64, f64)> {
    #[cfg(target_os = "ios")]
    {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;

        unsafe {
            let manager: *mut AnyObject = msg_send![objc2::class!(CLLocationManager), new];
            if manager.is_null() {
                eprintln!("[iOS Bridge] CLLocationManager: failed to create");
                return None;
            }

            // Request when-in-use authorization (no-op if already granted/denied)
            let _: () = msg_send![manager, requestWhenInUseAuthorization];

            // Get last known location (may be nil if no location fix yet)
            let location: *mut AnyObject = msg_send![manager, location];
            if location.is_null() {
                eprintln!("[iOS Bridge] CLLocationManager: no location available");
                return None;
            }

            let coord: CLLocationCoordinate2D = msg_send![location, coordinate];

            // Validate: (0,0) is in the Gulf of Guinea, not a valid Malmö location
            if coord.latitude == 0.0 && coord.longitude == 0.0 {
                eprintln!("[iOS Bridge] CLLocationManager: location is (0,0), likely invalid");
                return None;
            }

            eprintln!(
                "[iOS Bridge] GPS: lat={}, lon={}",
                coord.latitude, coord.longitude
            );
            Some((coord.latitude, coord.longitude))
        }
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

/// Export a file to user-chosen location via iOS share sheet (UIActivityViewController)
///
/// Presents the iOS system share sheet with the source file, allowing the user
/// to save to Files, AirDrop, email, etc. The file is copied to a temp location
/// with the suggested name before sharing.
///
/// # Arguments
/// * `source_path` - Absolute path to the file to export
/// * `suggested_name` - Suggested file name for the exported file
///
/// # Returns
/// - `Ok(())` if the share sheet was presented successfully
/// - `Err(message)` if presentation failed
pub fn export_file(source_path: &str, suggested_name: &str) -> Result<(), String> {
    #[cfg(target_os = "ios")]
    {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;

        // Verify source file exists
        if !std::path::Path::new(source_path).exists() {
            return Err(format!("Source file not found: {}", source_path));
        }

        // Copy to temp dir with suggested name so the share sheet shows the correct filename
        let temp_dir = std::env::temp_dir();
        let temp_path = format!("{}/{}", temp_dir.display(), suggested_name);
        std::fs::copy(source_path, &temp_path)
            .map_err(|e| format!("Failed to prepare file for export: {}", e))?;

        unsafe {
            let ns_path = make_nsstring(&temp_path);
            let file_url: *mut AnyObject =
                msg_send![objc2::class!(NSURL), fileURLWithPath: ns_path];
            if file_url.is_null() {
                return Err("Failed to create file URL".to_string());
            }

            let items: *mut AnyObject =
                msg_send![objc2::class!(NSArray), arrayWithObject: file_url];

            let avc_class = objc2::class!(UIActivityViewController);
            let avc: *mut AnyObject = msg_send![avc_class, alloc];
            let avc: *mut AnyObject = msg_send![
                avc,
                initWithActivityItems: items,
                applicationActivities: std::ptr::null::<AnyObject>()
            ];
            if avc.is_null() {
                return Err("Failed to create UIActivityViewController".to_string());
            }

            let vc = get_root_view_controller();
            if vc.is_null() {
                return Err("No root view controller available".to_string());
            }

            let _: () = msg_send![
                vc,
                presentViewController: avc,
                animated: true,
                completion: std::ptr::null::<AnyObject>()
            ];

            eprintln!(
                "[iOS Bridge] Share sheet presented for: {} ({})",
                source_path, suggested_name
            );
            Ok(())
        }
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

/// Import a file from the app's Documents/import directory
///
/// Looks for a `.parquet` file in the Documents/import/ directory. Users can
/// place files there via the iOS Files app (the app's Documents folder is
/// visible in Files when `UIFileSharingEnabled` is set in Info.plist).
///
/// # Returns
/// - `Ok(Some(path))` with temp file path if a .parquet file was found
/// - `Ok(None)` if no importable file was found
/// - `Err(message)` if import failed
pub fn import_file() -> Result<Option<String>, String> {
    #[cfg(target_os = "ios")]
    {
        let files_dir = std::env::var("APP_FILES_DIR").unwrap_or_default();
        if files_dir.is_empty() {
            return Err("APP_FILES_DIR not set".to_string());
        }

        let import_dir = format!("{}/import", files_dir);

        // Create import directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&import_dir) {
            return Err(format!("Failed to create import directory: {}", e));
        }

        // Find the first .parquet file in the import directory
        let entries = std::fs::read_dir(&import_dir)
            .map_err(|e| format!("Failed to read import directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("parquet") {
                let path_str = path
                    .to_str()
                    .ok_or_else(|| "Invalid path encoding".to_string())?
                    .to_string();

                // Copy to temp location (caller will delete after processing)
                let temp_dir = std::env::temp_dir();
                let millis = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                let temp_path = format!("{}/import_temp_{}.parquet", temp_dir.display(), millis);

                std::fs::copy(&path_str, &temp_path)
                    .map_err(|e| format!("Failed to copy import file: {}", e))?;

                // Remove the original so it's not re-imported next time
                let _ = std::fs::remove_file(&path_str);

                eprintln!("[iOS Bridge] Imported file from: {}", path_str);
                return Ok(Some(temp_path));
            }
        }

        Err(format!(
            "No .parquet file found. Place a file in the 'import' folder via the Files app (On My iPhone > amp > import), then try again. Path: {}",
            import_dir
        ))
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

    #[test]
    fn test_export_file_missing_source() {
        let result = export_file("/nonexistent/path.parquet", "test.parquet");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_file_non_ios() {
        let result = import_file();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_start_dormant_service_no_panic() {
        start_dormant_service();
    }
}
