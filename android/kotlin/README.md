# Kotlin Integration for Amp Android

This directory contains Kotlin helper classes that bridge Rust code to Android platform APIs via JNI.

## Overview

The Rust codebase uses JNI to call these Kotlin helper classes for Android-specific functionality:
- **NotificationHelper.kt**: Android notification system integration
- Additional helpers for GPS, permissions, etc. (future)

## Project Structure

```
android/
├── src/
│   ├── android_bridge.rs       # Rust JNI bridge
│   └── components/
│       └── notifications.rs    # Notification logic
└── kotlin/
    ├── NotificationHelper.kt   # Android notification API
    └── README.md              # This file
```

## Integration with Android Project

### Option 1: Dioxus Android Project

If using Dioxus CLI (`dx serve --platform android`), add the Kotlin files to your Android project:

```bash
# Assuming your Dioxus project generates an Android project at:
# ./android_project/ or similar

# Copy Kotlin files
cp android/kotlin/*.kt android_project/app/src/main/java/com/amp/
```

### Option 2: Manual Android Project Setup

1. Create an Android project (or use existing)
2. Add Kotlin support if not already enabled
3. Create package directory: `app/src/main/java/com/amp/`
4. Copy `NotificationHelper.kt` to that directory

## Kotlin File Setup

### NotificationHelper.kt

**Location**: `app/src/main/java/com/amp/NotificationHelper.kt`

**Dependencies**: Add to `app/build.gradle.kts`:

```kotlin
dependencies {
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.appcompat:appcompat:1.6.1")
    // Add other dependencies as needed
}
```

**Package Name**: Ensure package matches your app:
```kotlin
package com.amp  // or com.yourcompany.amp
```

## JNI Bridge Configuration

### Current Status

The Rust side (`android_bridge.rs`) has placeholder functions that need JNI implementation:

```rust
// In android/src/android_bridge.rs

#[cfg(target_os = "android")]
fn create_notification_channels() -> Result<(), String> {
    // TODO: Implement JNI call
    Err("JNI bridge not yet connected".to_string())
}

#[cfg(target_os = "android")]
fn show_notification(...) -> Result<(), String> {
    // TODO: Implement JNI call
    Err("JNI bridge not yet connected".to_string())
}
```

### Implementing the JNI Bridge

#### Step 1: Add JNI Dependency

Add to `android/Cargo.toml`:

```toml
[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"
```

#### Step 2: Obtain JNIEnv and Context

With Dioxus, you'll need to access the Android context. This typically requires:

```rust
// Example structure (adapt to your Dioxus setup)
use jni::JNIEnv;
use jni::objects::{JObject, JClass};

// This depends on how Dioxus exposes Android context
// You may need to store it globally or pass it through
fn get_jni_env() -> Result<JNIEnv<'static>, String> {
    // Obtain from Dioxus Android initialization
    // This is framework-specific
    todo!("Implement based on Dioxus Android setup")
}

fn get_android_context() -> Result<JObject<'static>, String> {
    // Obtain application context from Dioxus
    todo!("Implement based on Dioxus Android setup")
}
```

#### Step 3: Implement JNI Calls

Replace the TODO functions in `android_bridge.rs`:

```rust
#[cfg(target_os = "android")]
fn create_notification_channels() -> Result<(), String> {
    let env = get_jni_env()?;
    let context = get_android_context()?;
    
    // Find the NotificationHelper class
    let helper_class = env.find_class("com/amp/NotificationHelper")
        .map_err(|e| format!("Failed to find NotificationHelper class: {:?}", e))?;
    
    // Get the static method ID
    let method_id = env.get_static_method_id(
        helper_class,
        "createNotificationChannels",
        "(Landroid/content/Context;)V"
    ).map_err(|e| format!("Failed to get method ID: {:?}", e))?;
    
    // Call the method
    env.call_static_method_unchecked(
        helper_class,
        method_id,
        jni::signature::JavaType::Primitive(jni::signature::Primitive::Void),
        &[context.into()]
    ).map_err(|e| format!("Failed to call createNotificationChannels: {:?}", e))?;
    
    Ok(())
}

#[cfg(target_os = "android")]
fn show_notification(
    channel_id: &str,
    notification_id: i32,
    title: &str,
    body: &str,
) -> Result<(), String> {
    let env = get_jni_env()?;
    let context = get_android_context()?;
    
    // Convert Rust strings to Java strings
    let j_channel_id = env.new_string(channel_id)
        .map_err(|e| format!("Failed to create Java string: {:?}", e))?;
    let j_title = env.new_string(title)
        .map_err(|e| format!("Failed to create Java string: {:?}", e))?;
    let j_body = env.new_string(body)
        .map_err(|e| format!("Failed to create Java string: {:?}", e))?;
    
    // Find the class and method
    let helper_class = env.find_class("com/amp/NotificationHelper")
        .map_err(|e| format!("Failed to find class: {:?}", e))?;
    
    let method_id = env.get_static_method_id(
        helper_class,
        "showNotification",
        "(Landroid/content/Context;Ljava/lang/String;ILjava/lang/String;Ljava/lang/String;)V"
    ).map_err(|e| format!("Failed to get method ID: {:?}", e))?;
    
    // Call the method
    use jni::objects::JValue;
    env.call_static_method_unchecked(
        helper_class,
        method_id,
        jni::signature::JavaType::Primitive(jni::signature::Primitive::Void),
        &[
            context.into(),
            j_channel_id.into(),
            JValue::Int(notification_id),
            j_title.into(),
            j_body.into(),
        ]
    ).map_err(|e| format!("Failed to call showNotification: {:?}", e))?;
    
    Ok(())
}
```

## Android Manifest Permissions

Add required permissions to `AndroidManifest.xml`:

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.amp">

    <!-- Notification permission for Android 13+ (API 33+) -->
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />

    <!-- For foreground service (if using background notifications) -->
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_DATA_SYNC" />

    <application
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
        android:theme="@style/AppTheme">
        
        <!-- Your activities here -->
        
    </application>
</manifest>
```

## Runtime Permission Request

For Android 13+ (API 33+), request notification permission at runtime:

```kotlin
// In your MainActivity or Application class
import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

class MainActivity : AppCompatActivity() {
    private val NOTIFICATION_PERMISSION_CODE = 100

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Request notification permission on Android 13+
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            if (ContextCompat.checkSelfPermission(
                    this,
                    Manifest.permission.POST_NOTIFICATIONS
                ) != PackageManager.PERMISSION_GRANTED
            ) {
                ActivityCompat.requestPermissions(
                    this,
                    arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                    NOTIFICATION_PERMISSION_CODE
                )
            }
        }
        
        // Initialize notification channels
        NotificationHelper.createNotificationChannels(this)
    }
}
```

## Testing

### Unit Testing Kotlin

Create tests in `app/src/test/java/com/amp/`:

```kotlin
import android.content.Context
import org.junit.Test
import org.junit.runner.RunWith
import org.mockito.Mock
import org.mockito.junit.MockitoJUnitRunner

@RunWith(MockitoJUnitRunner::class)
class NotificationHelperTest {
    @Mock
    private lateinit var context: Context

    @Test
    fun testCreateChannels() {
        // Test channel creation doesn't crash
        NotificationHelper.createNotificationChannels(context)
    }
}
```

### Integration Testing

Test on a real device or emulator:

```bash
# Build and install
./gradlew installDebug

# Check logs for notification messages
adb logcat | grep AmpNotifications

# Verify channels created
adb shell dumpsys notification | grep amp_
```

### Testing from Rust

Run the integration tests:

```bash
cd android
cargo test --test notification_integration_test
```

## Debugging

### Common Issues

1. **ClassNotFoundException**: Verify package name matches in both Kotlin and JNI
   - Kotlin: `package com.amp`
   - JNI: `com/amp/NotificationHelper`

2. **NoSuchMethodError**: Check method signature matches
   - JNI signature: `"(Landroid/content/Context;)V"`
   - Must exactly match Kotlin method parameters

3. **SecurityException**: Request notification permission on Android 13+
   - Add `POST_NOTIFICATIONS` to manifest
   - Request at runtime before sending notifications

4. **Context null or invalid**: Ensure proper context retrieval
   - Use Application context for long-lived operations
   - Activity context for UI-related operations

### Debug Logging

Enable debug logs:

```bash
# Android logs
adb logcat -s AmpNotifications:D AndroidRuntime:E

# Rust logs (if using env_logger)
RUST_LOG=debug cargo run
```

## Next Steps

1. Implement `get_jni_env()` and `get_android_context()` based on your Dioxus setup
2. Replace TODO comments in `android_bridge.rs` with actual JNI calls
3. Test on an Android device/emulator
4. Request notification permissions in your main activity
5. Integrate lifecycle manager with Android WorkManager for background tasks

## Resources

- [Android Notification Guide](https://developer.android.com/develop/ui/views/notifications)
- [JNI in Rust](https://docs.rs/jni/latest/jni/)
- [Dioxus Mobile](https://dioxuslabs.com/learn/0.5/getting_started/mobile)
- [Kotlin for Android](https://kotlinlang.org/docs/android-overview.html)

## See Also

- [Notification System Documentation](../../docs/android-notifications.md)
- [Android README](../README.md)
- [Main Project README](../../README.md)
