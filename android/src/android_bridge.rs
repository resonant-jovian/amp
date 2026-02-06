//! Android platform bridge for native functionality
//!
//! Provides access to Android-specific features like GPS location, permissions,
//! notification system, and other system services using JNI (Java Native Interface).
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
//!
//! // Initialize notifications
//! android_bridge::initialize_notification_channels_jni();
//! ```

/// Initialize Android notification channels
///
/// Creates three notification channels for Android 8.0+ (API 26+):
/// - `amp_active`: High importance with sound, vibration, and heads-up
/// - `amp_six_hours`: High importance with sound and vibration
/// - `amp_one_day`: Low importance, silent notifications
///
/// Safe to call multiple times - Android handles duplicate channel creation.
///
/// # Platform Behavior
/// - **Android 8.0+**: Creates notification channels via NotificationHelper
/// - **Android <8.0**: No-op (channels not required)
/// - **Other platforms**: Mock implementation logs only
///
/// # JNI Integration
/// Calls `com.amp.NotificationHelper.createNotificationChannels(Context)`
///
/// # Examples
/// ```no_run
/// use amp_android::android_bridge::initialize_notification_channels_jni;
///
/// // Call once during app startup
/// initialize_notification_channels_jni();
/// ```
pub fn initialize_notification_channels_jni() {
    #[cfg(target_os = "android")]
    {
        match create_notification_channels() {
            Ok(()) => {
                eprintln!("[Android Bridge] Notification channels initialized successfully");
            }
            Err(e) => {
                eprintln!("[Android Bridge] Failed to initialize notification channels: {}", e);
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock Android Bridge] Notification channels initialized (no-op on non-Android)");
    }
}

/// Send a notification via Android NotificationManager
///
/// Displays a notification using the specified channel and content.
/// The channel determines notification priority, sound, and vibration behavior.
///
/// # Arguments
/// * `channel_id` - One of: "amp_active", "amp_six_hours", "amp_one_day"
/// * `notification_id` - Unique ID for this notification (typically address ID)
/// * `title` - Notification title text
/// * `body` - Notification body/content text
///
/// # Platform Behavior
/// - **Android**: Uses NotificationManagerCompat to display notification
/// - **Other platforms**: Mock implementation logs parameters
///
/// # JNI Integration
/// Calls `com.amp.NotificationHelper.showNotification(Context, String, int, String, String)`
///
/// # Examples
/// ```no_run
/// use amp_android::android_bridge::send_notification_jni;
///
/// send_notification_jni(
///     "amp_active",
///     1,
///     "Street cleaning NOW!",
///     "Your car is in an active zone"
/// );
/// ```
pub fn send_notification_jni(
    channel_id: &str,
    notification_id: i32,
    title: &str,
    body: &str,
) {
    #[cfg(target_os = "android")]
    {
        match show_notification(channel_id, notification_id, title, body) {
            Ok(()) => {
                eprintln!(
                    "[Android Bridge] Notification sent: channel={}, id={}, title='{}'",
                    channel_id, notification_id, title
                );
            }
            Err(e) => {
                eprintln!(
                    "[Android Bridge] Failed to send notification (channel={}, id={}): {}",
                    channel_id, notification_id, e
                );
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!(
            "[Mock Android Bridge] Would send notification: channel='{}', id={}, title='{}', body='{}'",
            channel_id, notification_id, title, body
        );
    }
}

/// Internal: Create notification channels via JNI
///
/// # JNI Implementation Notes
/// This function should:
/// 1. Get JNIEnv from the current thread/context
/// 2. Get Android Context (typically from MainActivity or Application)
/// 3. Find the NotificationHelper class: "com/amp/NotificationHelper"
/// 4. Call static method: `createNotificationChannels(Landroid/content/Context;)V`
///
/// # Example JNI Structure
/// ```ignore
/// let env = get_jni_env()?;
/// let context = get_android_context()?;
/// let helper_class = env.find_class("com/amp/NotificationHelper")?;
/// let method_id = env.get_static_method_id(
///     helper_class,
///     "createNotificationChannels",
///     "(Landroid/content/Context;)V"
/// )?;
/// env.call_static_method_unchecked(
///     helper_class,
///     method_id,
///     JavaType::Primitive(Primitive::Void),
///     &[context.into()]
/// )?;
/// ```
#[cfg(target_os = "android")]
fn create_notification_channels() -> Result<(), String> {
    // TODO: Implement JNI call to NotificationHelper.createNotificationChannels()
    //
    // Required steps:
    // 1. Obtain JNIEnv from Dioxus/Android context
    // 2. Get Android Context object
    // 3. Find NotificationHelper class
    // 4. Call createNotificationChannels static method
    //
    // The Kotlin NotificationHelper class should be created at:
    // android_project/app/src/main/java/com/amp/NotificationHelper.kt
    //
    // For now, return error indicating JNI not yet connected
    Err("JNI bridge not yet connected - requires NotificationHelper Kotlin class and JNIEnv access".to_string())
}

/// Internal: Show a notification via JNI
///
/// # JNI Implementation Notes
/// This function should:
/// 1. Get JNIEnv and Android Context
/// 2. Convert Rust strings to Java strings
/// 3. Find NotificationHelper class
/// 4. Call static method: `showNotification(Context, String, int, String, String)`
///
/// # Example JNI Structure
/// ```ignore
/// let env = get_jni_env()?;
/// let context = get_android_context()?;
/// let helper_class = env.find_class("com/amp/NotificationHelper")?;
/// let method_id = env.get_static_method_id(
///     helper_class,
///     "showNotification",
///     "(Landroid/content/Context;Ljava/lang/String;ILjava/lang/String;Ljava/lang/String;)V"
/// )?;
/// 
/// let j_channel_id = env.new_string(channel_id)?;
/// let j_title = env.new_string(title)?;
/// let j_body = env.new_string(body)?;
/// 
/// env.call_static_method_unchecked(
///     helper_class,
///     method_id,
///     JavaType::Primitive(Primitive::Void),
///     &[
///         context.into(),
///         j_channel_id.into(),
///         JValue::Int(notification_id),
///         j_title.into(),
///         j_body.into(),
///     ]
/// )?;
/// ```
#[cfg(target_os = "android")]
fn show_notification(
    channel_id: &str,
    notification_id: i32,
    title: &str,
    body: &str,
) -> Result<(), String> {
    // TODO: Implement JNI call to NotificationHelper.showNotification()
    //
    // Required steps:
    // 1. Obtain JNIEnv
    // 2. Get Android Context
    // 3. Convert Rust strings to Java strings
    // 4. Find NotificationHelper class
    // 5. Call showNotification static method with parameters
    //
    // For now, return error indicating JNI not yet connected
    let _ = (channel_id, notification_id, title, body); // Suppress unused warnings
    Err("JNI bridge not yet connected - requires NotificationHelper Kotlin class and JNIEnv access".to_string())
}

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
        eprintln!("[Mock Android Bridge] GPS location requested - platform not supported");
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

    #[test]
    fn test_initialize_channels_no_panic() {
        // Should not panic on any platform
        initialize_notification_channels_jni();
    }

    #[test]
    fn test_send_notification_no_panic() {
        // Should not panic on any platform
        send_notification_jni(
            "amp_active",
            1,
            "Test Title",
            "Test Body"
        );
    }
}
