# MainActivity Injection Required

## ⚠️ MANUAL STEP NEEDED

**The build script now copies and compiles `WebViewConfigurator.kt`, but it cannot be called automatically.**

### Problem

Dioxus/WRY generates `MainActivity.kt` automatically, and we cannot modify it via `build.sh` because:
1. It's generated **after** our script runs
2. Dioxus uses a Rust-compiled MainActivity (not Kotlin)
3. The WebView is created internally by WRY and not exposed

### Current Status

✅ **WebViewConfigurator.kt copied** to build output  
✅ **ProGuard rule added** to prevent R8 stripping  
✅ **sourceSets registered** for Kotlin compilation  
❌ **Not called from MainActivity** - requires manual injection OR Dioxus patch

---

## Option 1: Manual Patching (Temporary)

### After Build, Patch MainActivity

```bash
cd ~/Documents/amp
./scripts/build.sh

# Find MainActivity location
MAIN_ACTIVITY=$(find target/dx/amp/release/android/app/app/src/main -name "MainActivity.kt" 2>/dev/null)

if [ -f "$MAIN_ACTIVITY" ]; then
    echo "Found: $MAIN_ACTIVITY"
    
    # Check if already patched
    if grep -q "WebViewConfigurator" "$MAIN_ACTIVITY"; then
        echo "Already patched!"
    else
        # Inject WebViewConfigurator call
        sed -i '/override fun onCreate/a\
        \n        // Fix blank screen - enable DOM storage\n        try {\n            val webView = window.decorView.findViewById<android.webkit.WebView>(android.R.id.content)\n            webView?.let { se.malmo.skaggbyran.amp.WebViewConfigurator.configure(it) }\n        } catch (e: Exception) {\n            android.util.Log.e("amp_MainActivity", "Failed to configure WebView", e)\n        }' "$MAIN_ACTIVITY"
        
        echo "✓ MainActivity patched - WebViewConfigurator will be called"
        
        # Rebuild APK
        cd target/dx/amp/release/android/app
        ./gradlew assembleRelease
    fi
else
    echo "⚠️  MainActivity.kt not found (might be NativeActivity instead)"
fi
```

### If NativeActivity (No onCreate Override)

Dioxus uses `NativeActivity` for Rust integration, which doesn't have a Kotlin `onCreate` we can easily patch.

**Workaround**: Create custom `Application` class:

```kotlin
// android/kotlin/AmpApplication.kt
package se.malmo.skaggbyran.amp

import android.app.Application
import android.webkit.WebView
import android.util.Log

class AmpApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        
        // Enable WebView debugging
        WebView.setWebContentsDebuggingEnabled(true)
        
        Log.i("amp_Application", "Application initialized - WebView debugging enabled")
    }
}
```

Then modify `AndroidManifest.xml`:
```xml
<application
    android:name=".AmpApplication"
    ...
>
```

**BUT THIS DOESN'T HELP** - we need access to the specific WebView instance.

---

## Option 2: Dioxus Patch (Recommended)

### Fork dioxus-mobile or wry

The proper fix is to patch Dioxus/WRY to generate MainActivity with WebView configuration hooks.

### Approach A: Modify WRY's Android WebView Creation

In `wry/src/webview/android/mod.rs` (or similar), after WebView creation:

```rust
// After creating WebView, configure settings
pub fn configure_webview(webview: &WebView) {
    let settings = webview.get_settings();
    settings.set_javascript_enabled(true);
    settings.set_dom_storage_enabled(true);  // <-- ADD THIS
    settings.set_database_enabled(true);
    settings.set_allow_file_access(true);
}
```

### Approach B: Add JNI Hook in Rust

Call Java configuration from Rust after WebView is created:

```rust
use jni::objects::JObject;
use jni::JNIEnv;

pub fn configure_webview_from_rust(env: &mut JNIEnv, webview: JObject) -> Result<(), jni::errors::Error> {
    // Get WebViewConfigurator class
    let class = env.find_class("se/malmo/skaggbyran/amp/WebViewConfigurator")?;
    
    // Call static configure method
    env.call_static_method(
        class,
        "configure",
        "(Landroid/webkit/WebView;)V",
        &[webview.into()]
    )?;
    
    Ok(())
}
```

This would require modifying `amp_android/src/lib.rs` or wherever the WebView is initialized.

---

## Option 3: Runtime Injection (Hack)

### Use JNI from Rust to Configure WebView

After Dioxus initializes the WebView, call configuration from Rust:

```rust
// amp_android/src/webview_config.rs
use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use jni::sys::jint;

pub fn enable_dom_storage(env: &mut JNIEnv, activity: JObject) -> Result<(), Box<dyn std::error::Error>> {
    // Get the WebView from activity's DecorView
    let window = env.call_method(activity, "getWindow", "()Landroid/view/Window;", &[])?;
    let decor_view = env.call_method(window.l()?, "getDecorView", "()Landroid/view/View;", &[])?;
    
    // Find WebView by ID (android.R.id.content)
    let content_id = env.get_static_field(
        "android/R$id",
        "content",
        "I"
    )?.i()?;
    
    let webview = env.call_method(
        decor_view.l()?,
        "findViewById",
        "(I)Landroid/view/View;",
        &[JValue::Int(content_id)]
    )?;
    
    if let Ok(webview_obj) = webview.l() {
        // Get WebSettings
        let settings = env.call_method(
            webview_obj,
            "getSettings",
            "()Landroid/webkit/WebSettings;",
            &[]
        )?;
        
        // Enable DOM storage
        env.call_method(
            settings.l()?,
            "setDomStorageEnabled",
            "(Z)V",
            &[JValue::Bool(1)]
        )?;
        
        log::info!("✅ DOM storage enabled from Rust!");
    }
    
    Ok(())
}
```

Then call it from `amp_android/src/lib.rs` after platform initialization:

```rust
#[no_mangle]
pub extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    saved_state: *const u8,
    saved_state_size: usize,
) {
    // Existing Dioxus initialization...
    
    // AFTER WebView is created, configure it
    let ctx = ndk_context::android_context();
    let env = ctx.vm().attach_current_thread().unwrap();
    
    if let Ok(activity_obj) = JObject::from_raw(activity as *mut _) {
        if let Err(e) = webview_config::enable_dom_storage(&mut env, activity_obj) {
            log::error!("Failed to configure WebView: {}", e);
        }
    }
}
```

---

## Recommended Path Forward

### Short Term (Today)

**Test with INTERNET permission temporarily** to verify other functionality works:

```xml
<!-- AndroidManifest.xml - TEMPORARY -->
<uses-permission android:name="android.permission.INTERNET" />
```

This will enable DOM storage automatically, allowing you to verify:
- UI renders correctly
- NotificationHelper works
- App icons display
- No other issues

Then remove INTERNET permission once WebView config is working.

### Medium Term (This Week)

Implement **Option 3: Runtime Injection from Rust**:
1. Add JNI helper in `amp_android/src/webview_config.rs`
2. Call it after WRY initializes WebView
3. Verify with Chrome DevTools: `chrome://inspect`
4. Test localStorage in Console

### Long Term (Next Release)

Submit PR to Dioxus/WRY to add WebView configuration hooks:
- `dioxus::mobile::Config::with_webview_config(fn)`
- Default configuration enables DOM storage for offline apps
- Makes this a one-liner in user code

---

## Verification Steps

### 1. Check if WebView Configuration is Applied

```bash
# Monitor logcat for WebViewConfigurator logs
adb logcat | grep "amp_WebViewConfig"

# Expected output:
# amp_WebViewConfig: Configuring WebView for offline Dioxus operation...
# amp_WebViewConfig:   ✓ DOM storage enabled
# amp_WebViewConfig:   ✓ JavaScript enabled
# amp_WebViewConfig: ✅ WebView configuration complete
```

If you see these logs → WebViewConfigurator is being called ✅

### 2. Test DOM Storage with Chrome DevTools

```bash
# Enable WebView debugging (if not already in AmpApplication)
adb shell "setprop debug.webview.enable_crash_reporter 1"

# Open Chrome on your computer
# Navigate to: chrome://inspect
# Find your app's WebView
# Click "inspect"
```

In the Console:
```javascript
try {
    localStorage.setItem('test', 'works');
    console.log('✅ localStorage works:', localStorage.getItem('test'));
} catch (e) {
    console.error('❌ localStorage blocked:', e);
}
```

**Expected**: `✅ localStorage works: works`  
**If blocked**: WebViewConfigurator not called yet

### 3. Check DOM Rendering

In Chrome DevTools Elements tab:
```javascript
console.log('Document body:', document.body.innerHTML);
```

**Expected**: Your Dioxus component HTML  
**If empty**: DOM storage still blocked, Dioxus can't mount

---

## Current Build Output

With the latest `build.sh`:

✅ **WebViewConfigurator.kt** → copied to `src/main/kotlin/se/malmo/skaggbyran/amp/`  
✅ **ProGuard rule** → added to `proguard-rules.pro`  
✅ **sourceSets** → registered in `build.gradle.kts`  
✅ **Compiled to DEX** → verified with `dexdump`  
❌ **Called from code** → NOT YET (needs MainActivity patch or Rust JNI)

---

## Summary

**Problem**: Blank screen because DOM storage disabled  
**Root Cause**: No INTERNET permission → Android disables WebView storage  
**Solution Built**: WebViewConfigurator.kt that enables DOM storage  
**Missing Step**: Call it from MainActivity or Rust initialization  

**Next Actions**:
1. ✅ Test with INTERNET permission temporarily
2. ⏳ Implement Rust JNI configuration call
3. ⏳ Verify with Chrome DevTools
4. ⏳ Submit PR to Dioxus for permanent fix

---

## Files Modified

- ✅ `android/kotlin/WebViewConfigurator.kt` - Created
- ✅ `scripts/build.sh` - Copy WebViewConfigurator, add ProGuard rule
- ✅ `docs/blank-screen-fix.md` - Comprehensive documentation
- ✅ `docs/MainActivity-injection-required.md` - This file
- ⏳ `amp_android/src/webview_config.rs` - TODO: JNI configuration
- ⏳ `amp_android/src/lib.rs` - TODO: Call WebView config after init
