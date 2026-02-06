//! Local Android notifications for parking restriction warnings
//!
//! Provides notification scheduling when addresses transition between time panels.
//! Uses Android's NotificationManager and channels for proper categorization.
//!
//! # Notification Channels
//! - **amp_active**: High priority, sound + vibration + heads-up (currently active restrictions)
//! - **amp_six_hours**: High priority, sound + vibration (6-hour warnings)
//! - **amp_one_day**: Low priority, silent (1-day reminders)
//!
//! # Examples
//! ```no_run
//! use amp_android::components::notifications::{initialize_notification_channels, notify_active};
//! use amp_android::ui::StoredAddress;
//!
//! // Initialize on app startup
//! initialize_notification_channels();
//!
//! // Send notification when address becomes active
//! let address = StoredAddress {
//!     id: 1,
//!     street: "Storgatan".to_string(),
//!     street_number: "10".to_string(),
//!     postal_code: "22100".to_string(),
//!     valid: true,
//!     active: true,
//!     matched_entry: None,
//! };
//! notify_active(&address);
//! ```

use crate::components::settings::load_settings;
use crate::ui::StoredAddress;

/// Notification channel IDs
const CHANNEL_ACTIVE: &str = "amp_active";
const CHANNEL_SIX_HOURS: &str = "amp_six_hours";
const CHANNEL_ONE_DAY: &str = "amp_one_day";

/// Initialize notification channels on app startup (Android 8+)
///
/// Creates three notification channels with different importance levels:
/// - Active: IMPORTANCE_HIGH with sound, vibration, and heads-up display
/// - Six Hours: IMPORTANCE_HIGH with sound and vibration
/// - One Day: IMPORTANCE_LOW, silent notifications only
///
/// Call this once during app initialization, typically in your main activity
/// or application class. Safe to call multiple times (Android handles duplicates).
///
/// # Platform Support
/// - **Android 8.0+**: Creates proper notification channels
/// - **Android <8.0**: No-op (channels not required)
/// - **Other platforms**: Mock implementation for testing
///
/// # Examples
/// ```no_run
/// use amp_android::components::notifications::initialize_notification_channels;
///
/// // Call during app startup
/// initialize_notification_channels();
/// ```
pub fn initialize_notification_channels() {
    crate::android_bridge::initialize_notification_channels_jni();
}

/// Send notification when address enters "1 day" panel
///
/// Sends a low-priority, silent notification to remind the user about
/// upcoming street cleaning the next day. Respects user settings and
/// only sends if `en_dag` is enabled in NotificationSettings.
///
/// # Arguments
/// * `address` - The address that entered the 1-day warning window
///
/// # Examples
/// ```no_run
/// use amp_android::components::notifications::notify_one_day;
/// use amp_android::ui::StoredAddress;
///
/// let address = StoredAddress {
///     id: 1,
///     street: "Kornettsgatan".to_string(),
///     street_number: "18C".to_string(),
///     postal_code: "21438".to_string(),
///     valid: true,
///     active: false,
///     matched_entry: None,
/// };
/// notify_one_day(&address);
/// ```
pub fn notify_one_day(address: &StoredAddress) {
    let settings = load_settings();
    if !settings.notifications.en_dag {
        eprintln!(
            "[Notifications] Skipping 1-day notification for {} {} (disabled in settings)",
            address.street, address.street_number
        );
        return;
    }

    let title = "📅 Street cleaning tomorrow";
    let body = format!(
        "Street cleaning tomorrow on {}. Plan to move your car from {} {}.",
        address.street, address.street, address.street_number
    );

    send_notification(CHANNEL_ONE_DAY, title, &body, address.id);
}

/// Send notification when address enters "6 hours" panel
///
/// Sends a high-priority notification with sound and vibration to warn
/// the user that street cleaning will begin in approximately 6 hours.
/// Respects user settings and only sends if `sex_timmar` is enabled.
///
/// # Arguments
/// * `address` - The address that entered the 6-hour warning window
///
/// # Examples
/// ```no_run
/// use amp_android::components::notifications::notify_six_hours;
/// use amp_android::ui::StoredAddress;
///
/// let address = StoredAddress {
///     id: 2,
///     street: "Storgatan".to_string(),
///     street_number: "5".to_string(),
///     postal_code: "22100".to_string(),
///     valid: true,
///     active: false,
///     matched_entry: None,
/// };
/// notify_six_hours(&address);
/// ```
pub fn notify_six_hours(address: &StoredAddress) {
    let settings = load_settings();
    if !settings.notifications.sex_timmar {
        eprintln!(
            "[Notifications] Skipping 6-hour notification for {} {} (disabled in settings)",
            address.street, address.street_number
        );
        return;
    }

    let title = "⏰ Street cleaning in 6 hours";
    let body = format!(
        "Street cleaning starting soon on {}. Consider moving your car from {} {}.",
        address.street, address.street, address.street_number
    );

    send_notification(CHANNEL_SIX_HOURS, title, &body, address.id);
}

/// Send notification when address enters "active now" panel
///
/// Sends an urgent, high-priority notification with sound, vibration, and
/// heads-up display to alert the user that street cleaning is currently
/// active. Respects user settings and only sends if `stadning_nu` is enabled.
///
/// # Arguments
/// * `address` - The address that became active (restriction is currently in effect)
///
/// # Examples
/// ```no_run
/// use amp_android::components::notifications::notify_active;
/// use amp_android::ui::StoredAddress;
///
/// let address = StoredAddress {
///     id: 3,
///     street: "Parkgatan".to_string(),
///     street_number: "12".to_string(),
///     postal_code: "21400".to_string(),
///     valid: true,
///     active: true,
///     matched_entry: None,
/// };
/// notify_active(&address);
/// ```
pub fn notify_active(address: &StoredAddress) {
    let settings = load_settings();
    if !settings.notifications.stadning_nu {
        eprintln!(
            "[Notifications] Skipping active notification for {} {} (disabled in settings)",
            address.street, address.street_number
        );
        return;
    }

    let title = "🚫 Street cleaning NOW!";
    let body = format!(
        "Street cleaning active on {}. Your car at {} {} is in an active zone!",
        address.street, address.street, address.street_number
    );

    send_notification(CHANNEL_ACTIVE, title, &body, address.id);
}

/// Internal: Send notification via android_bridge to JNI
///
/// Routes notification requests through the android_bridge module,
/// which handles JNI calls to Android's NotificationManager.
///
/// # Arguments
/// * `channel_id` - One of CHANNEL_ACTIVE, CHANNEL_SIX_HOURS, or CHANNEL_ONE_DAY
/// * `title` - Notification title text
/// * `body` - Notification body text
/// * `notification_id` - Unique ID for this notification (typically address.id)
fn send_notification(channel_id: &str, title: &str, body: &str, notification_id: usize) {
    eprintln!(
        "[Notifications] Sending: channel={}, title={}, id={}",
        channel_id, title, notification_id
    );

    crate::android_bridge::send_notification_jni(
        channel_id,
        notification_id as i32,
        title,
        body,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_constants() {
        assert_eq!(CHANNEL_ACTIVE, "amp_active");
        assert_eq!(CHANNEL_SIX_HOURS, "amp_six_hours");
        assert_eq!(CHANNEL_ONE_DAY, "amp_one_day");
    }

    #[test]
    fn test_initialize_channels_no_panic() {
        // Should not panic on any platform
        initialize_notification_channels();
    }

    #[test]
    fn test_notify_functions_no_panic() {
        let address = StoredAddress {
            id: 1,
            street: "Test Street".to_string(),
            street_number: "10".to_string(),
            postal_code: "12345".to_string(),
            valid: true,
            active: false,
            matched_entry: None,
        };

        // All notification functions should handle mock mode gracefully
        notify_one_day(&address);
        notify_six_hours(&address);
        notify_active(&address);
    }

    #[test]
    fn test_send_notification_internal() {
        // Test internal function doesn't panic
        send_notification(CHANNEL_ACTIVE, "Test", "Body", 999);
    }
}
