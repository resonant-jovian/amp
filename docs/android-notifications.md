# Android Notification System Implementation

This document describes the local notification system for the amp Android parking app.

## Overview

The notification system alerts users when their parked car's address enters critical time windows:
- **1 day before**: Low-priority reminder
- **6 hours before**: High-priority warning
- **Active now**: Urgent alert with heads-up display

## Architecture

```text
UI Components / Periodic Timer
       |
       v
Storage (read addresses)
       |
       v
Transition Detector
   (detect_transitions)
       |
       v  (transitions found)
Notification Module
   (notify_one_day / notify_six_hours / notify_active)
       |
       v  (check settings)
JNI Bridge
       |
       v
Android NotificationManager
```

## Components

### 1. Notification Module (`components/notifications.rs`)

**Purpose**: Send notifications through Android's notification system

**Key Functions**:
- `initialize_notification_channels()` - Create Android 8+ channels on startup
- `notify_one_day(address)` - Send 1-day reminder (low priority, silent)
- `notify_six_hours(address)` - Send 6-hour warning (high priority, sound)
- `notify_active(address)` - Send active alert (urgent, heads-up)

**Features**:
- Respects user settings from `NotificationSettings`
- Platform-agnostic design (Android vs mock)
- Contextual messages using street names and numbers

### 2. Transition Detector (`components/transitions.rs`)

**Purpose**: Track panel changes to prevent duplicate notifications

**Key Functions**:
- `initialize_panel_tracker()` - Initialize state tracking
- `detect_transitions(addresses)` - Find addresses that changed panels
- `clear_panel_state()` - Reset state (testing/debug)

**State Management**:
- Thread-safe HashMap tracking `(address_id → TimeBucket)`
- Only notifies on transitions to more urgent buckets
- Filters out addresses without parking matches

**Transition Rules**:
```rust
// Notify on these transitions:
None → Within1Day       ✓
Within1Day → Within6Hours ✓
Within6Hours → Now        ✓

// No notification:
Within1Day → Within1Day   ✗ (same bucket)
Now → Within6Hours       ✗ (less urgent)
```

## Notification Channels

### Channel 1: `amp_active`
- **Name**: "Active Parking Restrictions"
- **Importance**: `IMPORTANCE_HIGH`
- **Behavior**: Sound + vibration + heads-up notification
- **Use**: Currently active street cleaning
- **Message**: "🚫 Street cleaning NOW! Street cleaning active on {street}..."

### Channel 2: `amp_six_hours`
- **Name**: "6-Hour Parking Warnings"
- **Importance**: `IMPORTANCE_HIGH`
- **Behavior**: Sound + vibration
- **Use**: 6-hour advance warning
- **Message**: "⏰ Street cleaning in 6 hours. Street cleaning starting soon on {street}..."

### Channel 3: `amp_one_day`
- **Name**: "1-Day Parking Reminders"
- **Importance**: `IMPORTANCE_LOW`
- **Behavior**: Silent, notification tray only
- **Use**: 1-day advance reminder
- **Message**: "📅 Street cleaning tomorrow. Street cleaning tomorrow on {street}..."

## Integration

### Step 1: Initialize on App Startup

```rust
use amp_android::components::{
    notifications::initialize_notification_channels,
    transitions::initialize_panel_tracker,
};

// In your main activity or app initialization:
pub fn init_app() {
    initialize_notification_channels();
    initialize_panel_tracker();
}
```

### Step 2: Periodic Transition Checking

```rust
use amp_android::components::{
    storage::read_addresses_from_device,
    transitions::detect_transitions,
    notifications::{notify_one_day, notify_six_hours, notify_active},
    countdown::TimeBucket,
};

// Call this every 60 seconds (or on address data updates)
pub fn check_notifications() {
    let addresses = read_addresses_from_device();
    let transitions = detect_transitions(&addresses);
    
    for (addr, _prev, new_bucket) in transitions {
        match new_bucket {
            TimeBucket::Within1Day => notify_one_day(&addr),
            TimeBucket::Within6Hours => notify_six_hours(&addr),
            TimeBucket::Now => notify_active(&addr),
            _ => {} // No notification for other buckets
        }
    }
}
```

### Step 3: Add to Lifecycle Component

Update `components/lifecycle.rs`:

```rust
use dioxus::prelude::*;
use std::time::Duration;

#[component]
pub fn NotificationChecker() -> Element {
    use_effect(move || {
        // Initialize once
        crate::components::notifications::initialize_notification_channels();
        crate::components::transitions::initialize_panel_tracker();
    });
    
    // Check every 60 seconds
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(60_000).await;
            check_notifications();
        }
    });
    
    rsx! {}
}
```

## Android Platform Setup

### 1. Create NotificationHelper (Kotlin)

File: `android_project/app/src/main/java/com/amp/NotificationHelper.kt`

```kotlin
package com.amp

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat

object NotificationHelper {
    private const val CHANNEL_ACTIVE = "amp_active"
    private const val CHANNEL_SIX_HOURS = "amp_six_hours"
    private const val CHANNEL_ONE_DAY = "amp_one_day"

    @JvmStatic
    fun createNotificationChannels(context: Context) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) 
                as NotificationManager

            // Channel 1: Active (HIGH)
            val activeChannel = NotificationChannel(
                CHANNEL_ACTIVE,
                "Active Parking Restrictions",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Urgent alerts for active street cleaning"
                enableVibration(true)
                enableLights(true)
            }

            // Channel 2: 6 Hours (HIGH)
            val sixHoursChannel = NotificationChannel(
                CHANNEL_SIX_HOURS,
                "6-Hour Parking Warnings",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Warnings 6 hours before street cleaning"
                enableVibration(true)
            }

            // Channel 3: 1 Day (LOW - silent)
            val oneDayChannel = NotificationChannel(
                CHANNEL_ONE_DAY,
                "1-Day Parking Reminders",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Reminders 1 day before street cleaning"
                setSound(null, null)
            }

            manager.createNotificationChannel(activeChannel)
            manager.createNotificationChannel(sixHoursChannel)
            manager.createNotificationChannel(oneDayChannel)
        }
    }

    @JvmStatic
    fun showNotification(
        context: Context,
        channelId: String,
        notificationId: Int,
        title: String,
        body: String
    ) {
        val priority = when (channelId) {
            CHANNEL_ACTIVE -> NotificationCompat.PRIORITY_HIGH
            CHANNEL_SIX_HOURS -> NotificationCompat.PRIORITY_HIGH
            else -> NotificationCompat.PRIORITY_LOW
        }

        val builder = NotificationCompat.Builder(context, channelId)
            .setSmallIcon(R.drawable.ic_notification) // Replace with your icon
            .setContentTitle(title)
            .setContentText(body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(body))
            .setPriority(priority)
            .setAutoCancel(true)

        with(NotificationManagerCompat.from(context)) {
            notify(notificationId, builder.build())
        }
    }
}
```

### 2. JNI Bridge (Rust)

Update `android/src/android_bridge.rs`:

```rust
#[cfg(target_os = "android")]
pub fn initialize_channels_jni() {
    use jni::JavaVM;
    
    // TODO: Get JavaVM and Context from your Dioxus Android setup
    // Call NotificationHelper.createNotificationChannels(context)
    
    eprintln!("[JNI] Initializing notification channels");
}

#[cfg(target_os = "android")]
pub fn send_notification_jni(
    channel_id: &str,
    notification_id: i32,
    title: &str,
    body: &str,
) {
    use jni::JavaVM;
    
    // TODO: Call NotificationHelper.showNotification(...) via JNI
    
    eprintln!(
        "[JNI] Sending notification: channel={}, id={}, title={}",
        channel_id, notification_id, title
    );
}
```

### 3. Update notifications.rs

Replace the JNI TODOs with actual bridge calls:

```rust
#[cfg(target_os = "android")]
pub fn initialize_notification_channels() {
    crate::android_bridge::initialize_channels_jni();
}

#[cfg(target_os = "android")]
fn send_notification(channel_id: &str, title: &str, body: &str, notification_id: usize) {
    crate::android_bridge::send_notification_jni(
        channel_id,
        notification_id as i32,
        title,
        body,
    );
}
```

## User Settings

The system respects user preferences from `NotificationSettings`:

```rust
pub struct NotificationSettings {
    pub stadning_nu: bool,    // Active notifications (default: true)
    pub sex_timmar: bool,     // 6-hour warnings (default: true)
    pub en_dag: bool,         // 1-day reminders (default: false)
}
```

Users can disable specific notification types:
- Settings are loaded from `settings.parquet`
- Each notification function checks the relevant flag
- If disabled, the notification is skipped with a log message

## Testing

### Unit Tests

```bash
cd android
cargo test --lib notifications
cargo test --lib transitions
```

### Integration Testing

```rust
#[test]
fn test_notification_flow() {
    use crate::components::{
        transitions::{initialize_panel_tracker, detect_transitions},
        countdown::TimeBucket,
    };
    
    initialize_panel_tracker();
    
    // Create test address that will trigger notification
    let addr = create_test_address(1, 1, "0800-1200");
    
    let transitions = detect_transitions(&[addr]);
    assert!(!transitions.is_empty());
    
    // Verify notification would be sent
    let (_, _, bucket) = &transitions[0];
    assert!(matches!(bucket, TimeBucket::Within1Day | TimeBucket::Within6Hours | TimeBucket::Now));
}
```

### Manual Testing on Device

1. **Install app** on Android device/emulator
2. **Add address** with upcoming restriction
3. **Wait for transition** or manipulate system time
4. **Verify notification** appears with correct:
   - Channel (check Android notification settings)
   - Sound/vibration behavior
   - Message content

## Troubleshooting

### No Notifications Appearing

1. Check notification permissions granted
2. Verify channels created: `adb shell dumpsys notification`
3. Check logs: `adb logcat | grep Notifications`
4. Verify settings: `stadning_nu`, `sex_timmar`, `en_dag` enabled

### Duplicate Notifications

1. Ensure `detect_transitions` is called (not bypassed)
2. Check panel state not being cleared unexpectedly
3. Verify same address ID used consistently

### Wrong Channel/Priority

1. Confirm `TimeBucket` calculation correct
2. Check Android channel importance settings
3. Verify mapping: Now→active, Within6Hours→six_hours, Within1Day→one_day

## References

- [Android Notifications Guide](https://developer.android.com/develop/ui/views/notifications)
- [Notification Channels](https://developer.android.com/develop/ui/views/notifications/channels)
- [Adaptive Notifications](https://fixyourandroid.com/about/android-adaptive-notifications/)

## Future Enhancements

- [ ] Notification actions ("Dismiss", "View Map")
- [ ] Deep linking to specific address
- [ ] Grouped notifications for multiple addresses
- [ ] Notification history/log
- [ ] Custom notification sounds per channel
- [ ] Rich media (map thumbnail)
