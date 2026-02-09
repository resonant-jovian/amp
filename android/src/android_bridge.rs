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
#[cfg(target_os = "android")]
use jni::{
    JavaVM,
    objects::{JClass, JObject, JValue},
    sys::jint,
};
#[cfg(target_os = "android")]
use ndk_context;
#[cfg(target_os = "android")]
use once_cell::sync::OnceCell;
#[cfg(target_os = "android")]
static JAVA_VM: OnceCell<JavaVM> = OnceCell::new();
/// Request notification permission from user
///
/// For Android 13+: Shows system permission dialog
/// For Android <13: No-op (permission not required)
pub fn request_notification_permission_jni() {
    #[cfg(target_os = "android")]
    {
        match request_notification_permission() {
            Ok(()) => eprintln!("[Android Bridge] Notification permission requested"),
            Err(e) => eprintln!("[Android Bridge] Permission request failed: {}", e),
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock Android Bridge] Notification permission request (no-op)");
    }
}
#[cfg(target_os = "android")]
fn request_notification_permission() -> Result<(), String> {
    let mut env = get_jni_env()?;
    let context = get_android_context()?;
    let class_loader = env
        .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .map_err(|e| format!("ClassLoader error: {:?}", e))?
        .l()
        .map_err(|e| format!("Not object: {:?}", e))?;
    let j_class_name = env
        .new_string("se.malmo.skaggbyran.amp.NotificationHelper")
        .map_err(|e| format!("String error: {:?}", e))?;
    let class_obj = env
        .call_method(
            class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&j_class_name.into())],
        )
        .map_err(|e| format!("Load class error: {:?}", e))?
        .l()
        .map_err(|e| format!("Not object: {:?}", e))?;
    let helper_class = JClass::from(class_obj);
    env.call_static_method(
        helper_class,
        "requestNotificationPermission",
        "(Landroid/app/Activity;)V",
        &[JValue::Object(&context)],
    )
    .map_err(|e| format!("Call failed: {:?}", e))?;
    Ok(())
}
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
/// Calls `se.malmo.skaggbyran.amp.NotificationHelper.createNotificationChannels(Context)`
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
                eprintln!("[Android Bridge] Notification channels initialized successfully",);
            }
            Err(e) => {
                eprintln!(
                    "[Android Bridge] Failed to initialize notification channels: {}",
                    e,
                );
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock Android Bridge] Notification channels initialized (no-op on non-Android)",);
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
/// Calls `se.malmo.skaggbyran.amp.NotificationHelper.showNotification(Context, String, int, String, String)`
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
pub fn send_notification_jni(channel_id: &str, notification_id: i32, title: &str, body: &str) {
    #[cfg(target_os = "android")]
    {
        match show_notification(channel_id, notification_id, title, body) {
            Ok(()) => {
                eprintln!(
                    "[Android Bridge] Notification sent: channel={}, id={}, title='{}'",
                    channel_id, notification_id, title,
                );
            }
            Err(e) => {
                eprintln!(
                    "[Android Bridge] Failed to send notification (channel={}, id={}): {}",
                    channel_id, notification_id, e,
                );
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!(
            "[Mock Android Bridge] Would send notification: channel='{}', id={}, title='{}', body='{}'",
            channel_id, notification_id, title, body,
        );
    }
}
/// Get or initialize the JavaVM
///
/// Returns a reference to the JavaVM stored in static memory.
/// Initializes it on first call using ndk_context.
///
/// # Returns
/// Reference to JavaVM with 'static lifetime
#[cfg(target_os = "android")]
fn get_java_vm() -> Result<&'static JavaVM, String> {
    JAVA_VM.get_or_try_init(|| {
        let ctx = ndk_context::android_context();
        let vm_ptr = ctx.vm() as *mut jni::sys::JavaVM;
        unsafe { JavaVM::from_raw(vm_ptr) }
            .map_err(|e| format!("Failed to create JavaVM from ndk_context: {:?}", e))
    })
}
/// Get JNIEnv for the current thread
///
/// Uses ndk_context to get the JavaVM and attaches the current thread.
/// This is the proper way to get JNIEnv in Dioxus Android apps.
///
/// # Returns
/// Result containing JNIEnv or error message
///
/// # Implementation Notes
/// - Uses `ndk_context::android_context()` to get native activity context
/// - Attaches current thread to JavaVM
/// - Thread-safe and works from any Rust thread
/// - Returns an AttachGuard which must be kept alive
#[cfg(target_os = "android")]
fn get_jni_env() -> Result<jni::AttachGuard<'static>, String> {
    let vm = get_java_vm()?;
    let env = vm
        .attach_current_thread()
        .map_err(|e| format!("Failed to attach thread to JavaVM: {:?}", e))?;
    Ok(env)
}
/// Get Android Context object
///
/// Retrieves the Android Context (Activity or Application) for use in JNI calls.
/// Uses ndk_context to get the native activity context.
///
/// # Returns
/// Result containing JObject Context or error message
///
/// # Implementation Notes
/// - Uses ndk_context to get native activity
/// - Returns the activity as a Context (Activity extends Context)
/// - Valid for the lifetime of the activity
#[cfg(target_os = "android")]
fn get_android_context() -> Result<JObject<'static>, String> {
    let ctx = ndk_context::android_context();
    let activity_ptr = ctx.context() as *mut jni::sys::_jobject;
    let activity = unsafe { JObject::from_raw(activity_ptr) };
    if activity.is_null() {
        return Err("Android context is null".to_string());
    }
    Ok(activity)
}
/// Create notification channels via JNI
///
/// Calls NotificationHelper.createNotificationChannels(Context) using JNI.
///
/// # Returns
/// Result indicating success or error message
///
/// # JNI Call Structure
/// - Class: `se/malmo/skaggbyran/amp/NotificationHelper`
/// - Method: `createNotificationChannels`
/// - Signature: `(Landroid/content/Context;)V`
/// - Static method call with Context parameter
#[cfg(target_os = "android")]
fn create_notification_channels() -> Result<(), String> {
    let mut env = get_jni_env()?;
    let context = get_android_context()?;
    let class_loader = env
        .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .map_err(|e| format!("Failed to get ClassLoader: {:?}", e))?
        .l()
        .map_err(|e| format!("ClassLoader not an object: {:?}", e))?;
    let j_class_name = env
        .new_string("se.malmo.skaggbyran.amp.NotificationHelper")
        .map_err(|e| format!("Failed to create class name string: {:?}", e))?;
    let class_obj = env
        .call_method(
            class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&j_class_name.into())],
        )
        .map_err(|e| format!("Failed to load NotificationHelper: {:?}", e))?
        .l()
        .map_err(|e| format!("Loaded class not an object: {:?}", e))?;
    let helper_class = JClass::from(class_obj);
    env.call_static_method(
        helper_class,
        "createNotificationChannels",
        "(Landroid/content/Context;)V",
        &[JValue::Object(&context)],
    )
    .map_err(|e| format!("Failed to call createNotificationChannels: {:?}", e))?;
    Ok(())
}
/// Show a notification via JNI
///
/// Calls NotificationHelper.showNotification(...) using JNI with proper
/// string conversion and parameter handling.
///
/// # Arguments
/// * `channel_id` - Notification channel ID
/// * `notification_id` - Unique notification ID
/// * `title` - Notification title
/// * `body` - Notification body text
///
/// # Returns
/// Result indicating success or error message
///
/// # JNI Call Structure
/// - Class: `se/malmo/skaggbyran/amp/NotificationHelper`
/// - Method: `showNotification`
/// - Signature: `(Landroid/content/Context;Ljava/lang/String;ILjava/lang/String;Ljava/lang/String;)V`
/// - Parameters: Context, channelId, notificationId, title, body
#[cfg(target_os = "android")]
fn show_notification(
    channel_id: &str,
    notification_id: i32,
    title: &str,
    body: &str,
) -> Result<(), String> {
    let mut env = get_jni_env()?;
    let context = get_android_context()?;
    let j_channel_id = env
        .new_string(channel_id)
        .map_err(|e| format!("Failed to create Java string for channel_id: {:?}", e))?;
    let j_title = env
        .new_string(title)
        .map_err(|e| format!("Failed to create Java string for title: {:?}", e))?;
    let j_body = env
        .new_string(body)
        .map_err(|e| format!("Failed to create Java string for body: {:?}", e))?;
    let class_loader = env
        .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .map_err(|e| format!("Failed to get ClassLoader: {:?}", e))?
        .l()
        .map_err(|e| format!("ClassLoader not an object: {:?}", e))?;
    let j_class_name = env
        .new_string("se.malmo.skaggbyran.amp.NotificationHelper")
        .map_err(|e| format!("Failed to create class name string: {:?}", e))?;
    let class_obj = env
        .call_method(
            class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&j_class_name.into())],
        )
        .map_err(|e| format!("Failed to load NotificationHelper: {:?}", e))?
        .l()
        .map_err(|e| format!("Loaded class not an object: {:?}", e))?;
    let helper_class = JClass::from(class_obj);
    env.call_static_method(
        helper_class,
        "showNotification",
        "(Landroid/content/Context;Ljava/lang/String;ILjava/lang/String;Ljava/lang/String;)V",
        &[
            JValue::Object(&context),
            JValue::Object(&j_channel_id),
            JValue::Int(notification_id as jint),
            JValue::Object(&j_title),
            JValue::Object(&j_body),
        ],
    )
    .map_err(|e| format!("Failed to call showNotification: {:?}", e))?;
    Ok(())
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
    #[test]
    fn test_initialize_channels_no_panic() {
        initialize_notification_channels_jni();
    }
    #[test]
    fn test_send_notification_no_panic() {
        send_notification_jni("amp_active", 1, "Test Title", "Test Body");
    }
    #[test]
    fn test_request_permission_no_panic() {
        request_notification_permission_jni();
    }
}
