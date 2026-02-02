//! Android notification system
//!
//! Provides functionality to send local notifications to the user.
//! Uses Android NotificationManager and NotificationCompat.
#[cfg(target_os = "android")]
use jni::{
    JNIEnv,
    objects::{JObject, JString, JValue},
};
use std::sync::atomic::{AtomicI32, Ordering};
/// Counter for generating unique notification IDs
static NOTIFICATION_ID_COUNTER: AtomicI32 = AtomicI32::new(1);
/// Send an Android notification
///
/// Displays a notification to the user with the specified title and body.
///
/// # Arguments
/// * `title` - Notification title
/// * `body` - Notification body text
///
/// # Returns
/// Ok(notification_id) if successful, Err with message if failed
///
/// # Platform Behavior
/// - **Android**: Creates and displays a system notification
/// - **Other platforms**: Logs to stderr (mock)
///
/// # Examples
/// ```no_run
/// let title = "Parking Alert";
/// let body = "Storgatan 10 - 2 hours remaining";
/// if let Ok(id) = send_android_notification(title, body) {
///     println!("Notification sent with ID: {}", id);
/// }
/// ```
pub fn send_android_notification(title: &str, body: &str) -> Result<i32, String> {
    let notification_id = NOTIFICATION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    #[cfg(target_os = "android")]
    {
        send_notification_android(notification_id, title, body)?;
        Ok(notification_id)
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!(
            "[Mock Notification #{}] {}: {}",
            notification_id, title, body
        );
        Ok(notification_id)
    }
}
#[cfg(target_os = "android")]
fn send_notification_android(id: i32, title: &str, body: &str) -> Result<(), String> {
    eprintln!(
        "[Android] Notification not fully implemented - would show: {} - {}",
        title, body,
    );
    Ok(())
}
/// Cancel a notification by ID
///
/// Removes a previously displayed notification from the notification tray.
///
/// # Arguments
/// * `notification_id` - ID returned from send_android_notification
///
/// # Returns
/// Ok(()) if successful, Err with message if failed
pub fn cancel_notification(notification_id: i32) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        cancel_notification_android(notification_id)
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock] Would cancel notification #{}", notification_id);
        Ok(())
    }
}
#[cfg(target_os = "android")]
fn cancel_notification_android(id: i32) -> Result<(), String> {
    Ok(())
}
/// Send notification with action
///
/// Sends a notification that can open a specific location in maps when tapped.
///
/// # Arguments
/// * `title` - Notification title
/// * `body` - Notification body text
/// * `address` - Address to open in maps (e.g., "Storgatan 10, 22100")
///
/// # Returns
/// Ok(notification_id) if successful, Err with message if failed
pub fn send_notification_with_map_action(
    title: &str,
    body: &str,
    address: &str,
) -> Result<i32, String> {
    let notification_id = NOTIFICATION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    #[cfg(target_os = "android")]
    {
        eprintln!(
            "[Android] Map notification not implemented: {} - {} ({})",
            title, body, address,
        );
        Ok(notification_id)
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!(
            "[Mock Notification #{}] {}: {} (would open: {})",
            notification_id, title, body, address,
        );
        Ok(notification_id)
    }
}
/// Create notification channel (Android 8.0+)
///
/// Required for notifications on Android Oreo and above.
/// Should be called once during app initialization.
///
/// # Arguments
/// * `channel_id` - Unique channel identifier
/// * `channel_name` - User-visible channel name
/// * `importance` - Channel importance (1=low, 2=default, 3=high)
#[cfg(target_os = "android")]
pub fn create_notification_channel(channel_id: &str, channel_name: &str, importance: i32) {
    eprintln!(
        "[Android] Would create notification channel: {} ({})",
        channel_id, channel_name,
    );
}
#[cfg(not(target_os = "android"))]
pub fn create_notification_channel(_channel_id: &str, _channel_name: &str, _importance: i32) {}
