# Android Kotlin Integration for Notifications

This guide explains how to integrate the Kotlin NotificationHelper class with the Rust notification system.

## Overview

The notification system requires:
1. **Rust side**: JNI bridge functions in `android_bridge.rs` (✅ complete)
2. **Kotlin side**: `NotificationHelper` class (this document)
3. **Android project**: Proper build configuration and permissions

## Where to Place NotificationHelper.kt

When using Dioxus CLI (`dx`) to build Android apps, the Kotlin/Java code should be placed in your Android Studio project structure:

```
your-android-project/
├── app/
│   └── src/
│       └── main/
│           ├── java/
│           │   └── com/
│           │       └── amp/
│           │           └── NotificationHelper.kt  ← Place here
│           ├── AndroidManifest.xml
│           └── res/
│               └── drawable/
│                   └── ic_notification.xml  ← Notification icon
├── build.gradle
└── app/
    └── build.gradle
```

**Note**: The package name `com.amp` should match your `applicationId` in `app/build.gradle`.

## NotificationHelper.kt Source Code

Create `app/src/main/java/com/amp/NotificationHelper.kt` with this content:

```kotlin
package com.amp

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat

/**
 * Helper class for managing Android notifications for the amp parking app
 *
 * Provides static methods callable from Rust via JNI to create notification
 * channels and display notifications.
 */
object NotificationHelper {

    private const val CHANNEL_ACTIVE = "amp_active"
    private const val CHANNEL_SIX_HOURS = "amp_six_hours"
    private const val CHANNEL_ONE_DAY = "amp_one_day"

    @JvmStatic
    fun createNotificationChannels(context: Context) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val manager = context.getSystemService(Context.NOTIFICATION_SERVICE)
                    as NotificationManager

            // Channel 1: Active Now (IMPORTANCE_HIGH)
            val activeChannel = NotificationChannel(
                CHANNEL_ACTIVE,
                "Active Parking Restrictions",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Urgent alerts when street cleaning is currently active"
                enableVibration(true)
                enableLights(true)
                lightColor = android.graphics.Color.RED
                setShowBadge(true)
            }

            // Channel 2: 6 Hours (IMPORTANCE_HIGH)
            val sixHoursChannel = NotificationChannel(
                CHANNEL_SIX_HOURS,
                "6-Hour Parking Warnings",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Warnings when street cleaning starts in 6 hours"
                enableVibration(true)
                setShowBadge(true)
            }

            // Channel 3: 1 Day (IMPORTANCE_LOW)
            val oneDayChannel = NotificationChannel(
                CHANNEL_ONE_DAY,
                "1-Day Parking Reminders",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Reminders one day before street cleaning"
                setSound(null, null)
                enableVibration(false)
                setShowBadge(true)
            }

            manager.createNotificationChannel(activeChannel)
            manager.createNotificationChannel(sixHoursChannel)
            manager.createNotificationChannel(oneDayChannel)

            android.util.Log.d("NotificationHelper", "Notification channels created")
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
            CHANNEL_ONE_DAY -> NotificationCompat.PRIORITY_LOW
            else -> NotificationCompat.PRIORITY_DEFAULT
        }

        val builder = NotificationCompat.Builder(context, channelId)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentTitle(title)
            .setContentText(body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(body))
            .setPriority(priority)
            .setAutoCancel(true)

        when (channelId) {
            CHANNEL_ACTIVE -> builder.setCategory(NotificationCompat.CATEGORY_ALARM)
            else -> builder.setCategory(NotificationCompat.CATEGORY_REMINDER)
        }

        try {
            val notificationManager = NotificationManagerCompat.from(context)
            if (notificationManager.areNotificationsEnabled()) {
                notificationManager.notify(notificationId, builder.build())
                android.util.Log.d(
                    "NotificationHelper",
                    "Notification shown: channel=$channelId, id=$notificationId"
                )
            }
        } catch (e: Exception) {
            android.util.Log.e("NotificationHelper", "Error: ${e.message}")
        }
    }
}
```

## AndroidManifest.xml Configuration

Add these permissions to `app/src/main/AndroidManifest.xml`:

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.amp">

    <!-- Notification permissions -->
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
    <uses-permission android:name="android.permission.VIBRATE" />
    
    <!-- For heads-up notifications (optional) -->
    <uses-permission android:name="android.permission.USE_FULL_SCREEN_INTENT" />

    <application
        android:name=".AmpApplication"
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
        android:theme="@style/AppTheme">
        
        <!-- Your activities here -->
        
    </application>
</manifest>
```

## build.gradle Configuration

Ensure `app/build.gradle` has the required dependencies:

```gradle
android {
    compileSdk 34
    
    defaultConfig {
        applicationId "com.amp"
        minSdk 26  // Android 8.0+ for notification channels
        targetSdk 34
        versionCode 1
        versionName "1.0"
    }
    
    compileOptions {
        sourceCompatibility JavaVersion.VERSION_1_8
        targetCompatibility JavaVersion.VERSION_1_8
    }
    
    kotlinOptions {
        jvmTarget = "1.8"
    }
}

dependencies {
    implementation 'androidx.core:core-ktx:1.12.0'
    implementation 'androidx.appcompat:appcompat:1.6.1'
    
    // Required for notifications
    implementation 'androidx.core:core:1.12.0'
}
```

## Notification Icon

Create `app/src/main/res/drawable/ic_notification.xml`:

```xml
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="24dp"
    android:height="24dp"
    android:viewportWidth="24"
    android:viewportHeight="24"
    android:tint="?attr/colorControlNormal">
    <path
        android:fillColor="@android:color/white"
        android:pathData="M12,2C8.13,2 5,5.13 5,9c0,5.25 7,13 7,13s7,-7.75 7,-13c0,-3.87 -3.13,-7 -7,-7zM12,11.5c-1.38,0 -2.5,-1.12 -2.5,-2.5s1.12,-2.5 2.5,-2.5 2.5,1.12 2.5,2.5 -1.12,2.5 -2.5,2.5z"/>
</vector>
```

## Integrating with Dioxus Build

The Dioxus CLI (`dx`) handles Android builds, but you need to ensure your Android Studio project is properly configured:

### Option 1: Using dx with Existing Android Project

1. Create your Android Studio project with the Kotlin code above
2. Configure `Dioxus.toml` to point to your Android project:

```toml
[android]
path = "path/to/your/android-project"
application_id = "com.amp"
```

3. Build with `dx build --platform android`

### Option 2: Generate Android Project from dx

1. Run `dx create --platform android` to generate base project
2. Add the NotificationHelper.kt to the generated structure
3. Update AndroidManifest.xml and build.gradle as shown above
4. Build with `dx build --platform android`

## Calling from MainActivity

In your MainActivity (typically auto-generated by Dioxus), initialize channels:

```kotlin
package com.amp

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize notification channels
        NotificationHelper.createNotificationChannels(this)
        
        // Rest of your Dioxus setup...
    }
}
```

## Connecting JNI Bridge

The Rust side (`android_bridge.rs`) currently has TODO comments for JNI implementation. To connect:

### 1. Add JNI Dependencies

In your `android/Cargo.toml`:

```toml
[dependencies]
jni = "0.21"
```

### 2. Implement JNI Functions

Update `android_bridge.rs` to actually call the Kotlin methods:

```rust
#[cfg(target_os = "android")]
fn create_notification_channels() -> Result<(), String> {
    use jni::objects::JObject;
    use jni::JavaVM;
    
    // Get JNI environment (this part depends on how Dioxus exposes JNI)
    // Typically available through dioxus-android or similar
    let vm = get_java_vm()?;  // Implementation depends on Dioxus setup
    let env = vm.attach_current_thread()
        .map_err(|e| format!("Failed to attach JNI thread: {}", e))?;
    
    let context = get_android_context()?;  // Get from Dioxus
    
    let helper_class = env.find_class("com/amp/NotificationHelper")
        .map_err(|e| format!("Failed to find NotificationHelper class: {}", e))?;
    
    env.call_static_method(
        helper_class,
        "createNotificationChannels",
        "(Landroid/content/Context;)V",
        &[context.into()]
    ).map_err(|e| format!("Failed to call createNotificationChannels: {}", e))?;
    
    Ok(())
}
```

**Note**: The exact implementation of `get_java_vm()` and `get_android_context()` depends on how Dioxus exposes JNI access. Check Dioxus Android documentation or examples.

## Testing

### 1. Build and Install

```bash
cd android
dx build --platform android --release
dx serve --platform android  # Or install APK manually
```

### 2. Check Logs

```bash
adb logcat | grep -E "(NotificationHelper|Lifecycle|Notifications)"
```

### 3. Verify Channels

```bash
adb shell dumpsys notification | grep amp_
```

You should see:
```
amp_active (IMPORTANCE_HIGH)
amp_six_hours (IMPORTANCE_HIGH)
amp_one_day (IMPORTANCE_LOW)
```

### 4. Manual Test

From Android device:
1. Settings → Apps → amp → Notifications
2. Verify three channels are listed
3. Check channel settings match specifications

## Troubleshooting

### NotificationHelper class not found

```
Error: Failed to find NotificationHelper class
```

**Solution**: Verify package name matches:
- `package com.amp` in NotificationHelper.kt
- `applicationId "com.amp"` in build.gradle
- `env.find_class("com/amp/NotificationHelper")` in Rust

### Notifications not appearing

1. Check permissions granted:
   ```bash
   adb shell dumpsys package com.amp | grep POST_NOTIFICATIONS
   ```

2. Request runtime permission (Android 13+):
   ```kotlin
   if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
       requestPermissions(
           arrayOf(android.Manifest.permission.POST_NOTIFICATIONS),
           1
       )
   }
   ```

### Build errors

If Kotlin compilation fails:
```bash
./gradlew clean
./gradlew build --info
```

## Next Steps

1. **Complete JNI bridge**: Implement actual JNI calls in `android_bridge.rs`
2. **Test on device**: Install and verify notifications appear
3. **Add UI toggle**: Let users control notification preferences
4. **Add deep linking**: Tap notification to open specific address
5. **Add actions**: "Dismiss" or "View Map" buttons on notifications

## References

- [Android Notifications Guide](https://developer.android.com/develop/ui/views/notifications)
- [Dioxus Android Setup](https://dioxuslabs.com/learn/0.5/getting_started/mobile)
- [JNI Rust Bindings](https://docs.rs/jni/latest/jni/)
- [Notification System Docs](./android-notifications.md)
