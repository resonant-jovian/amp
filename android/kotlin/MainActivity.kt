package dev.dioxus.main

import android.util.Log
import android.webkit.WebView
import se.malmo.skaggbyran.amp.BuildConfig
// REMOVED: import se.malmo.skaggbyran.amp.WebViewConfigurator
// Using reflection instead to avoid class verification failure

/**
 * Custom MainActivity that extends Dioxus's auto-generated WryActivity.
 * 
 * This class overrides `onWebViewCreate()` to configure WebView DOM storage
 * for offline Dioxus operation without INTERNET permission.
 * 
 * ## Architecture
 * 
 * Dioxus generates:
 * - WryActivity (base class with lifecycle + WebView hooks)
 * - RustWebView (custom WebView with domStorageEnabled = true)
 * - MainActivity (extends WryActivity) ← WE REPLACE THIS
 * 
 * ## Hook: onWebViewCreate()
 * 
 * Called by WryActivity.setWebView() after RustWebView is created but
 * before it's added to the view hierarchy. Perfect timing for configuration.
 * 
 * ## Why Additional Configuration?
 * 
 * RustWebView already sets `settings.domStorageEnabled = true`, but:
 * 1. Android may still disable storage when no INTERNET permission
 * 2. WebSettings may need explicit configuration for offline mode
 * 3. DatabasePath may need to be set for persistent storage
 * 
 * WebViewConfigurator ensures ALL storage APIs work correctly.
 * 
 * ## Why Reflection?
 * 
 * Direct import causes Android's class verifier to load WebViewConfigurator
 * at MainActivity startup (before try-catch). If loading fails, app crashes.
 * 
 * Reflection defers loading until runtime, allowing graceful fallback.
 */
typealias BuildConfig = BuildConfig

class MainActivity : WryActivity() {
    
    companion object {
        private const val TAG = "amp_MainActivity"
        private const val CONFIGURATOR_CLASS = "se.malmo.skaggbyran.amp.WebViewConfigurator"
        private const val CONFIGURE_METHOD = "configure"
    }
    
    /**
     * Called when WebView is created by WRY.
     * 
     * This hook is invoked by WryActivity.setWebView() right after the
     * RustWebView instance is created. We use this to apply additional
     * WebView configuration.
     * 
     * @param webView The newly created RustWebView instance
     */
    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        
        Log.i(TAG, "onWebViewCreate called, configuring WebView...")
        
        try {
            // Load WebViewConfigurator class dynamically using reflection
            // This avoids class verification failure at MainActivity load time
            val configuratorClass = Class.forName(CONFIGURATOR_CLASS)
            Log.d(TAG, "✅ WebViewConfigurator class loaded via reflection")
            
            // Get the static configure(WebView) method
            val configureMethod = configuratorClass.getMethod(
                CONFIGURE_METHOD,
                WebView::class.java
            )
            Log.d(TAG, "✅ configure() method found")
            
            // Invoke: WebViewConfigurator.configure(webView)
            configureMethod.invoke(null, webView)
            Log.i(TAG, "✅ WebView configured successfully via reflection")
            
        } catch (e: ClassNotFoundException) {
            Log.e(TAG, "❌ WebViewConfigurator class not found in APK", e)
            Log.e(TAG, "   Class name: $CONFIGURATOR_CLASS")
            Log.e(TAG, "   This should not happen - check build configuration")
            // Non-fatal - RustWebView already has domStorageEnabled=true
            
        } catch (e: NoSuchMethodException) {
            Log.e(TAG, "❌ configure() method not found in WebViewConfigurator", e)
            Log.e(TAG, "   Expected signature: static void configure(WebView)")
            // Non-fatal
            
        } catch (e: Exception) {
            Log.e(TAG, "❌ Failed to configure WebView via reflection: ${e.javaClass.simpleName}", e)
            Log.e(TAG, "   Message: ${e.message}")
            // Non-fatal - app may still work with RustWebView defaults
        }
    }
}
