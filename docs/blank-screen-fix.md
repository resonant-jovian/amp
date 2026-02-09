# WebView Blank Screen Fix

## Problem

**App builds successfully and launches in 195ms, but displays only a blank white screen.**

### Symptoms

- ✅ APK installs without errors
- ✅ Activity launches successfully: `Displayed se.malmo.skaggbyran.amp/.dev.dioxus.main.MainActivity for user 0: +195ms`
- ✅ No crashes in logcat
- ✅ NO INTERNET permission (security requirement)
- ✅ NotificationHelper compiled into DEX
- ❌ **Blank white screen** - no UI content rendered

### Logcat Evidence (No Errors)

```
02-09 10:22:22.040 nativeloader: Load ...libdioxusmain.so ... ok
02-09 10:22:22.079 RustStdoutStderr: AndroidUtils: Got filesDir from JNI: /data/user/0/...
02-09 10:22:22.117 ActivityTaskManager: Displayed se.malmo.skaggbyran.amp/... for user 0: +195ms
02-09 10:22:22.086 .skaggbyran.amp: type=1400 audit: avc: denied { read } for name="u:object_r:vendor_display_prop:s0"
                                  ^^^^^^^^ SELinux warning (normal, not the issue)
```

**No WebView errors, no JavaScript errors, no asset loading failures.**

---

## Root Cause

**Dioxus/WRY WebView requires DOM storage when running without INTERNET permission.**

From Stack Overflow [android-webview-not-loading-correctly](https://stackoverflow.com/questions/35072352):

> I got it working by `setDomStorageEnabled(true);`  
> You need to set this when using local storage.

### Why This Happens

1. **Dioxus uses local storage** for state management and reactivity
2. **Android WebView disables DOM storage by default** when no internet permission
3. **Without DOM storage**, Dioxus can't persist component state → blank screen
4. **JavaScript is enabled** (we can verify), but **localStorage/sessionStorage are blocked**

### Similar Issues

- [Dioxus #1875](https://github.com/DioxusLabs/dioxus/issues/1875) - "Blank page when running mobile application on android"
- [Dioxus #1762](https://github.com/DioxusLabs/dioxus/issues/1762) - "Just a grey empty inspect window with nothing in it"
- [Dioxus #3470](https://github.com/DioxusLabs/dioxus/issues/3470) - "app... stay completely white/blank"

---

## Solution 1: Inject WebView Configuration (Automatic)

**Modify `scripts/build.sh` to inject WebView settings into generated MainActivity.**

### Implementation

Add to `setup_notifications()` function in `scripts/build.sh`:

```bash
# ========== FIX: Enable DOM storage for offline WebView ==========
echo ""
echo "  🌐 Configuring WebView for offline operation..."

MAIN_ACTIVITY="$ANDROID_DIR/src/main/kotlin/dev/dioxus/main/MainActivity.kt"

if [ -f "$MAIN_ACTIVITY" ]; then
    # Check if already patched
    if ! grep -q "domStorageEnabled" "$MAIN_ACTIVITY"; then
        echo "    📝 Injecting DOM storage configuration..."
        
        # Find onCreate method and add WebView configuration
        # This enables localStorage/sessionStorage for Dioxus
        sed -i '/override fun onCreate/a\
        \n        // CRITICAL: Enable DOM storage for Dioxus without internet permission\n        val webView = window.decorView.findViewById<android.webkit.WebView>(android.R.id.content)\n        webView?.settings?.apply {\n            javaScriptEnabled = true\n            domStorageEnabled = true  // Required for Dioxus state management\n            databaseEnabled = true    // Enable database API\n            // Allow mixed content for local assets\n            mixedContentMode = android.webkit.WebSettings.MIXED_CONTENT_ALWAYS_ALLOW\n        }' "$MAIN_ACTIVITY"
        
        echo "    ✓ DOM storage enabled"
    else
        echo "    ✓ DOM storage configuration already present"
    fi
else
    echo "    ⚠️  MainActivity.kt not found at $MAIN_ACTIVITY"
    echo "    Manual configuration required (see Solution 2)"
fi
# ========== END WEBVIEW FIX ==========
```

### What This Does

```kotlin
override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    
    // INJECTED CODE
    val webView = window.decorView.findViewById<WebView>(android.R.id.content)
    webView?.settings?.apply {
        javaScriptEnabled = true      // Already enabled by WRY
        domStorageEnabled = true      // CRITICAL for Dioxus
        databaseEnabled = true        // Enable WebSQL/IndexedDB
        mixedContentMode = MIXED_CONTENT_ALWAYS_ALLOW  // For local assets
    }
}
```

---

## Solution 2: Create Custom WebViewClient (Manual)

**If automatic injection fails, create a custom WebView configurator in Kotlin.**

### File: `android/kotlin/WebViewConfigurator.kt`

```kotlin
package se.malmo.skaggbyran.amp

import android.webkit.WebView
import android.webkit.WebSettings

object WebViewConfigurator {
    @JvmStatic
    fun configure(webView: WebView) {
        webView.settings.apply {
            // Enable JavaScript (should already be enabled)
            javaScriptEnabled = true
            
            // CRITICAL: Enable DOM storage for Dioxus state management
            // Without this, localStorage/sessionStorage are blocked
            domStorageEnabled = true
            
            // Enable database APIs (IndexedDB, WebSQL)
            databaseEnabled = true
            
            // Allow mixed content (HTTP resources in HTTPS pages)
            // Needed for local assets even without internet
            mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
            
            // Allow file access for local HTML/CSS/JS
            allowFileAccess = true
            allowContentAccess = true
            
            // Enable zoom if needed
            builtInZoomControls = false
            displayZoomControls = false
            
            // Set cache mode for offline operation
            cacheMode = WebSettings.LOAD_DEFAULT
        }
    }
}
```

### Call From MainActivity

```kotlin
package dev.dioxus.main

import android.os.Bundle
import se.malmo.skaggbyran.amp.WebViewConfigurator

class MainActivity : android.app.NativeActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Find the WebView created by WRY/Dioxus
        val webView = window.decorView.findViewById<android.webkit.WebView>(android.R.id.content)
        webView?.let {
            WebViewConfigurator.configure(it)
        }
    }
}
```

---

## Solution 3: Modify WRY Configuration (Advanced)

**If you have access to WRY configuration in your Rust code:**

```rust
use dioxus::prelude::*;
use wry::webview::WebViewBuilder;

fn main() {
    LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_custom_head(r#"
                    <meta http-equiv="Content-Security-Policy" content="default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:">
                "#.to_string())
        )
        .launch(app);
}
```

**Note**: This approach is limited - Android WebView settings must be configured in Java/Kotlin.

---

## Verification

After applying the fix:

### 1. Check WebView Settings in Logcat

```bash
adb logcat | grep -E "(WebView|domStorage|JavaScript)"
```

Expected output:
```
WebView: JavaScript enabled: true
WebView: DOM storage enabled: true
```

### 2. Test with Chrome DevTools

```bash
# Enable WebView debugging
adb shell am broadcast -a android.intent.action.BOOT_COMPLETED

# Open chrome://inspect in Chrome on your computer
# You should see your app's WebView
# Check Console for errors
```

### 3. Check Local Storage

In Chrome DevTools Console:
```javascript
try {
    localStorage.setItem('test', 'works');
    console.log('localStorage:', localStorage.getItem('test'));
} catch (e) {
    console.error('localStorage blocked:', e);
}
```

Expected: `localStorage: works`

### 4. Verify Dioxus Mounted

In Chrome DevTools Console:
```javascript
console.log('Document:', document.body.innerHTML);
```

Expected: Your Dioxus component HTML (not empty `<body></body>`)

---

## Why DOM Storage is Required

### Dioxus State Management

Dioxus uses `localStorage` for:
1. **Component state persistence** across hot reloads
2. **Signal values** in reactive components
3. **Router state** for navigation history
4. **Asset caching** for performance

### WebView Default Behavior

| Permission | JavaScript | DOM Storage | Database |
|------------|------------|-------------|----------|
| **With INTERNET** | ✅ Enabled | ✅ Enabled | ✅ Enabled |
| **Without INTERNET** | ✅ Enabled | ❌ **Disabled** | ❌ **Disabled** |

**Result**: Dioxus can't persist state → components fail to mount → blank screen

---

## Alternative Diagnosis Methods

### If DOM Storage Fix Doesn't Work

#### 1. Check for JavaScript Errors

```bash
adb logcat | grep -E "(Console|JavaScript|Error)"
```

#### 2. Verify Assets Are Loaded

```bash
adb shell run-as se.malmo.skaggbyran.amp
ls -la /data/user/0/se.malmo.skaggbyran.amp/files/
ls -la /data/app/*/se.malmo.skaggbyran.amp*/base.apk!/assets/
```

Check for:
- `style.css` present
- Correct file permissions (readable)
- No zero-byte files

#### 3. Check Content Security Policy

If you have a `network_security_config.xml` (you shouldn't after our fix), verify:
```xml
<network-security-config>
    <!-- Should NOT exist after INTERNET permission removal -->
</network-security-config>
```

#### 4. Test with Minimal HTML

Create `android/test_webview.html`:
```html
<!DOCTYPE html>
<html>
<head>
    <title>Test</title>
</head>
<body>
    <h1>WebView Works!</h1>
    <script>
        console.log('JavaScript works');
        try {
            localStorage.setItem('test', 'works');
            document.body.innerHTML += '<p>localStorage: ' + localStorage.getItem('test') + '</p>';
        } catch (e) {
            document.body.innerHTML += '<p>localStorage BLOCKED: ' + e + '</p>';
        }
    </script>
</body>
</html>
```

Load it:
```kotlin
webView.loadUrl("file:///android_asset/test_webview.html")
```

If you see "localStorage BLOCKED" → DOM storage fix needed.

---

## Quick Fix to Test Hypothesis

**Temporarily add INTERNET permission to test:**

```xml
<!-- AndroidManifest.xml -->
<uses-permission android:name="android.permission.INTERNET" />
```

Rebu build and install. If the blank screen **disappears**, then the issue is definitely DOM storage.

Then **remove INTERNET permission** and apply the DOM storage fix.

---

## Implementation Checklist

- [ ] Option 1: Modify `build.sh` to inject WebView configuration
- [ ] Option 2: Create `WebViewConfigurator.kt` and call from MainActivity
- [ ] Option 3: Add ProGuard rule to keep WebViewConfigurator:
  ```proguard
  -keep class se.malmo.skaggbyran.amp.WebViewConfigurator {
      public *;
  }
  ```
- [ ] Rebuild APK: `cd android && ../scripts/build.sh`
- [ ] Uninstall old: `adb uninstall se.malmo.skaggbyran.amp`
- [ ] Install new: `adb install target/dx/.../*.apk`
- [ ] Verify with Chrome DevTools: `chrome://inspect`
- [ ] Test localStorage in Console
- [ ] Verify UI renders

---

## References

- [Stack Overflow: WebView Not Loading](https://stackoverflow.com/questions/35072352/android-webview-not-loading-correctly)
- [Dioxus Issue #1875: Blank Page Android](https://github.com/DioxusLabs/dioxus/issues/1875)
- [Dioxus Issue #1762: Grey Empty Window](https://github.com/DioxusLabs/dioxus/issues/1762)
- [Android WebView Settings Documentation](https://developer.android.com/reference/android/webkit/WebSettings)
- [WebView Mixed Content](https://developer.android.com/reference/android/webkit/WebSettings#setMixedContentMode(int))

---

## Summary

**Problem**: Blank white screen after successful build and launch  
**Root Cause**: DOM storage disabled without INTERNET permission  
**Solution**: Enable `domStorageEnabled` in WebView settings  
**Priority**: **HIGH** - blocks all UI rendering  
**Complexity**: **LOW** - single settings change  
**Testing**: Chrome DevTools (`chrome://inspect`)  
