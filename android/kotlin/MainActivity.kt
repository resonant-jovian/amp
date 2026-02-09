package dev.dioxus.main

import android.util.Log
import android.webkit.WebView
import se.malmo.skaggbyran.amp.BuildConfig
import se.malmo.skaggbyran.amp.WebViewConfigurator

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
 */
typealias BuildConfig = BuildConfig

class MainActivity : WryActivity() {
    
    companion object {
        private const val TAG = "amp_MainActivity"
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
            // Call WebViewConfigurator to ensure DOM storage is enabled
            // This supplements RustWebView's domStorageEnabled = true
            WebViewConfigurator.configure(webView)
            Log.i(TAG, "✅ WebView configured successfully")
        } catch (e: Exception) {
            Log.e(TAG, "❌ Failed to configure WebView: ${e.message}", e)
            // Non-fatal - RustWebView already has domStorageEnabled=true
            // App may still work, but localStorage might be limited
        }
    }
}
