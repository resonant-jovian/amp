//! WebView configuration for Android without INTERNET permission.
//!
//! This module provides JNI-based configuration of the Android WebView to enable
//! DOM storage (localStorage/sessionStorage) which is required by Dioxus but
//! disabled by default when no INTERNET permission is present.
//!
//! # Problem
//!
//! Dioxus uses localStorage for component state management. Android WebView
//! disables DOM storage when apps don't have INTERNET permission (security policy).
//! Without DOM storage, Dioxus cannot mount components → blank screen.
//!
//! # Solution
//!
//! Call [`configure_webview_dom_storage`] after WebView creation to explicitly
//! enable storage APIs without requiring network access.
//!
//! # Architecture
//!
//! ```text
//! Rust (main.rs)
//!   ↓ launch()
//! Dioxus (internal)
//!   ↓ creates WebView
//! WRY (webview)
//!   ↓ Android WebView created
//! JNI Bridge (this module)
//!   ↓ finds WebView instance
//! WebViewConfigurator.kt
//!   ↓ enables DOM storage
//! Android WebView
//!   ✓ localStorage enabled
//! ```
//!
//! # Usage
//!
//! ## Background Thread Configuration (Current)
//!
//! Spawn background thread to configure WebView without blocking UI:
//!
//! ```no_run
//! use amp_android::webview_config;
//!
//! fn main() {
//!     // Spawn configuration thread
//!     std::thread::spawn(|| {
//!         std::thread::sleep(std::time::Duration::from_millis(300));
//!         if let Err(e) = webview_config::configure_webview_dom_storage() {
//!             log::error!("WebView config failed: {}", e);
//!         }
//!     });
//!
//!     // Launch Dioxus (blocking)
//!     dioxus::launch(ui::App);
//! }
//! ```
//!
//! # Implementation Details
//!
//! ## JNI Call Chain
//!
//! 1. Get Android Context from `ndk_context`
//! 2. Convert raw VM pointer to JavaVM
//! 3. Attach to JVM thread
//! 4. Get Activity's ClassLoader (critical for app classes)
//! 5. Load `WebViewConfigurator` class via app ClassLoader
//! 6. Get `Activity` from context
//! 7. Find `WebView` in activity's view hierarchy
//! 8. Call `WebViewConfigurator.configure(webView)`
//!
//! ## ClassLoader Context Fix
//!
//! **Critical:** JNI's `find_class()` uses the system ClassLoader, which can't
//! see app-specific classes. We must use `Activity.getClassLoader().loadClass()`
//! to find classes in the app's APK.
//!
//! ## Error Handling
//!
//! All JNI errors are caught and logged. If configuration fails:
//! - App will show blank screen (same as before)
//! - Logcat will show error details
//! - Workaround: Add INTERNET permission temporarily
//!
//! # Verification
//!
//! Check logcat for configuration success:
//!
//! ```bash
//! adb logcat | grep -E '(amp_WebViewConfig|amp_webview)'
//! ```
//!
//! Expected output:
//! ```text
//! amp_webview: Attempting WebView configuration...
//! amp_WebViewConfig: Configuring WebView for offline Dioxus operation...
//! amp_WebViewConfig:   ✓ DOM storage enabled
//! amp_webview: ✅ WebView configured successfully
//! ```
//!
//! # Troubleshooting
//!
//! ## Configuration Not Called
//!
//! Logcat missing `amp_webview` logs → Check if function is being called
//!
//! ## JNI Errors
//!
//! ```text
//! JNI error: class not found
//! ```
//! → WebViewConfigurator.kt not compiled, check build.sh
//!
//! ```text
//! JNI error: method not found
//! ```
//! → R8 stripped the class, check ProGuard rules
//!
//! ## Still Blank Screen
//!
//! 1. Verify WebView found: `adb logcat | grep 'WebView instance'`
//! 2. Check DOM storage: Chrome DevTools → `chrome://inspect`
//! 3. Test localStorage: `localStorage.setItem('test', 'works')`
//!
//! # References
//!
//! - [Dioxus Issue #1875](https://github.com/DioxusLabs/dioxus/issues/1875)
//! - [Android WebSettings](https://developer.android.com/reference/android/webkit/WebSettings)
//! - [JNI Spec](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/)
//! - [Android ClassLoader](https://developer.android.com/reference/java/lang/ClassLoader)
#[cfg(target_os = "android")]
use jni::{
    JNIEnv, JavaVM,
    objects::{JObject, JValue, JClass},
};
#[cfg(target_os = "android")]
use log::{debug, error, info, warn};
const TAG: &str = "amp_webview";
/// Configure Android WebView to enable DOM storage for offline Dioxus operation.
///
/// This function uses JNI to call the Kotlin `WebViewConfigurator.configure()`
/// method, which enables localStorage/sessionStorage required by Dioxus.
///
/// # Returns
///
/// - `Ok(())` if WebView was configured successfully
/// - `Err(String)` if configuration failed (with error details)
///
/// # Errors
///
/// Possible errors:
/// - JVM not available (not running on Android)
/// - WebViewConfigurator class not found (not compiled into APK)
/// - WebView not found in activity hierarchy
/// - JNI method call failed
///
/// # Example
///
/// ```no_run
/// use amp_android::webview_config;
///
/// if let Err(e) = webview_config::configure_webview_dom_storage() {
///     log::error!("WebView config failed: {}", e);
/// }
/// ```
#[cfg(target_os = "android")]
pub fn configure_webview_dom_storage() -> Result<(), String> {
    info!("[{}] Attempting WebView configuration...", TAG);
    let ctx = ndk_context::android_context();
    let vm_ptr = ctx.vm() as *mut jni::sys::JavaVM;
    let vm = unsafe { JavaVM::from_raw(vm_ptr) }
        .map_err(|e| format!("Failed to create JavaVM from raw pointer: {:?}", e))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("Failed to attach to JVM: {:?}", e))?;
    configure_webview_internal(&mut env)
}
/// Internal JNI implementation of WebView configuration.
///
/// Separated from public API to allow easier testing and mocking.
/// 
/// # ClassLoader Fix
/// 
/// **Critical:** Uses Activity's ClassLoader instead of system ClassLoader.
/// JNI's `find_class()` searches the system ClassLoader, which doesn't have
/// access to app-specific classes in the APK. We must use:
/// ```java
/// activity.getClassLoader().loadClass("se.malmo.skaggbyran.amp.WebViewConfigurator")
/// ```
#[cfg(target_os = "android")]
fn configure_webview_internal(env: &mut JNIEnv) -> Result<(), String> {
    debug!("[{}] Getting Activity context...", TAG);
    let ctx = ndk_context::android_context();
    let activity = unsafe { JObject::from_raw(ctx.context() as *mut _) };
    debug!("[{}] ✓ Activity context obtained", TAG);
    
    // ========== CRITICAL FIX: Use app ClassLoader ==========
    debug!("[{}] Getting Activity's ClassLoader...", TAG);
    let class_loader = env
        .call_method(
            activity,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )
        .map_err(|e| format!("Failed to get ClassLoader: {:?}", e))?
        .l()
        .map_err(|e| format!("ClassLoader not an object: {:?}", e))?;
    debug!("[{}] ✓ ClassLoader obtained from Activity", TAG);
    
    debug!("[{}] Loading WebViewConfigurator via app ClassLoader...", TAG);
    let class_name = env
        .new_string("se.malmo.skaggbyran.amp.WebViewConfigurator")
        .map_err(|e| format!("Failed to create class name string: {:?}", e))?;
    
    let configurator_class_obj = env
        .call_method(
            class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&class_name.into())],
        )
        .map_err(|e| {
            error!("[{}] ❌ Failed to load WebViewConfigurator via ClassLoader: {:?}", TAG, e);
            error!("[{}] This means the class is NOT in the APK or was stripped by R8", TAG);
            error!("[{}] Check: dexdump -l plain app.apk | grep WebViewConfigurator", TAG);
            format!("WebViewConfigurator class not found in app ClassLoader: {:?}", e)
        })?
        .l()
        .map_err(|e| format!("Loaded class not an object: {:?}", e))?;
    
    // Convert JObject to JClass for static method calls
    let configurator_class = JClass::from(configurator_class_obj);
    debug!("[{}] ✓ WebViewConfigurator class loaded via app ClassLoader", TAG);
    // ========== END CLASSLOADER FIX ==========
    
    debug!("[{}] Getting Window from Activity...", TAG);
    let window = env
        .call_method(activity, "getWindow", "()Landroid/view/Window;", &[])
        .map_err(|e| format!("Failed to get Window: {:?}", e))?
        .l()
        .map_err(|e| format!("Window not an object: {:?}", e))?;
    debug!("[{}] ✓ Window obtained", TAG);
    debug!("[{}] Getting DecorView from Window...", TAG);
    let decor_view = env
        .call_method(window, "getDecorView", "()Landroid/view/View;", &[])
        .map_err(|e| format!("Failed to get DecorView: {:?}", e))?
        .l()
        .map_err(|e| format!("DecorView not an object: {:?}", e))?;
    debug!("[{}] ✓ DecorView obtained", TAG);
    debug!("[{}] Searching for WebView in view hierarchy...", TAG);
    let webview = find_webview_in_hierarchy(env, decor_view)?;
    info!("[{}] ✓ WebView instance found", TAG);
    debug!("[{}] Calling WebViewConfigurator.configure()...", TAG);
    env.call_static_method(
        configurator_class,
        "configure",
        "(Landroid/webkit/WebView;)V",
        &[JValue::Object(&webview)],
    )
    .map_err(|e| {
        error!("[{}] ❌ Failed to call configure(): {:?}", TAG, e);
        error!("[{}] Check ProGuard rules - class may be stripped", TAG);
        format!("Failed to call configure(): {:?}", e)
    })?;
    info!("[{}] ✅ WebView configured successfully", TAG);
    info!("[{}] DOM storage should now be enabled", TAG);
    Ok(())
}
/// Find WebView in Android view hierarchy.
///
/// Recursively searches through ViewGroup children to find WebView instance.
///
/// # Strategy
///
/// 1. Check if current view is a WebView
/// 2. If ViewGroup, iterate through children
/// 3. Return first WebView found
///
/// # Returns
///
/// - `Ok(JObject)` if WebView found
/// - `Err(String)` if no WebView in hierarchy
#[cfg(target_os = "android")]
fn find_webview_in_hierarchy<'a>(
    env: &mut JNIEnv<'a>,
    view: JObject<'a>,
) -> Result<JObject<'a>, String> {
    let is_webview = env
        .is_instance_of(&view, "android/webkit/WebView")
        .map_err(|e| format!("Failed to check WebView instance: {:?}", e))?;
    if is_webview {
        debug!("[{}] Found WebView!", TAG);
        return Ok(view);
    }
    let is_viewgroup = env
        .is_instance_of(&view, "android/view/ViewGroup")
        .map_err(|e| format!("Failed to check ViewGroup instance: {:?}", e))?;
    if !is_viewgroup {
        return Err("Not a ViewGroup, cannot traverse".to_string());
    }
    let child_count = env
        .call_method(&view, "getChildCount", "()I", &[])
        .map_err(|e| format!("Failed to get child count: {:?}", e))?
        .i()
        .map_err(|e| format!("Child count not an int: {:?}", e))?;
    debug!(
        "[{}] ViewGroup has {} children, searching...",
        TAG, child_count
    );
    for i in 0..child_count {
        let child = env
            .call_method(
                &view,
                "getChildAt",
                "(I)Landroid/view/View;",
                &[JValue::Int(i)],
            )
            .map_err(|e| format!("Failed to get child {}: {:?}", i, e))?
            .l()
            .map_err(|e| format!("Child {} not an object: {:?}", i, e))?;
        if let Ok(webview) = find_webview_in_hierarchy(env, child) {
            return Ok(webview);
        }
    }
    Err(format!(
        "WebView not found in ViewGroup with {} children",
        child_count
    ))
}
/// Verify if WebView has DOM storage enabled (for debugging).
///
/// This function checks the WebView's current settings to confirm DOM storage
/// was enabled successfully.
///
/// # Returns
///
/// - `Ok(true)` if DOM storage is enabled
/// - `Ok(false)` if DOM storage is disabled
/// - `Err(String)` if verification failed
#[cfg(target_os = "android")]
pub fn verify_dom_storage_enabled() -> Result<bool, String> {
    let ctx = ndk_context::android_context();
    let vm_ptr = ctx.vm() as *mut jni::sys::JavaVM;
    let vm = unsafe { JavaVM::from_raw(vm_ptr) }
        .map_err(|e| format!("Failed to create JavaVM from raw pointer: {:?}", e))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("Failed to attach to JVM: {:?}", e))?;
    let ctx_obj = unsafe { JObject::from_raw(ctx.context() as *mut _) };
    let window = env
        .call_method(ctx_obj, "getWindow", "()Landroid/view/Window;", &[])
        .map_err(|e| format!("Failed to get Window: {:?}", e))?
        .l()
        .map_err(|e| format!("Window not an object: {:?}", e))?;
    let decor_view = env
        .call_method(window, "getDecorView", "()Landroid/view/View;", &[])
        .map_err(|e| format!("Failed to get DecorView: {:?}", e))?
        .l()
        .map_err(|e| format!("DecorView not an object: {:?}", e))?;
    let webview = find_webview_in_hierarchy(&mut env, decor_view)?;
    let settings = env
        .call_method(
            &webview,
            "getSettings",
            "()Landroid/webkit/WebSettings;",
            &[],
        )
        .map_err(|e| format!("Failed to get WebSettings: {:?}", e))?
        .l()
        .map_err(|e| format!("WebSettings not an object: {:?}", e))?;
    let enabled = env
        .call_method(&settings, "getDomStorageEnabled", "()Z", &[])
        .map_err(|e| format!("Failed to get domStorageEnabled: {:?}", e))?
        .z()
        .map_err(|e| format!("domStorageEnabled not a boolean: {:?}", e))?;
    if enabled {
        info!("[{}] ✅ DOM storage is ENABLED", TAG);
    } else {
        warn!("[{}] ⚠️  DOM storage is DISABLED", TAG);
    }
    Ok(enabled)
}
#[cfg(not(target_os = "android"))]
pub fn configure_webview_dom_storage() -> Result<(), String> {
    Ok(())
}
#[cfg(not(target_os = "android"))]
pub fn verify_dom_storage_enabled() -> Result<bool, String> {
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_non_android_stubs() {
        assert!(configure_webview_dom_storage().is_ok());
        assert!(verify_dom_storage_enabled().is_ok());
    }
}
