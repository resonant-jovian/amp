package se.malmo.skaggbyran.amp

import android.webkit.WebView
import android.webkit.WebSettings
import android.util.Log

/**
 * WebView configurator for Dioxus/WRY apps running without INTERNET permission.
 *
 * ## Problem
 * Android WebView disables DOM storage (localStorage/sessionStorage) by default
 * when INTERNET permission is not present. Dioxus requires DOM storage for
 * component state management and reactivity.
 *
 * ## Solution
 * This configurator explicitly enables:
 * - DOM storage (localStorage/sessionStorage)
 * - Database APIs (IndexedDB/WebSQL)
 * - Mixed content mode (HTTP resources in HTTPS context)
 * - File access (for bundled assets)
 *
 * ## Usage
 * Call from MainActivity.onCreate():
 * ```kotlin
 * override fun onCreate(savedInstanceState: Bundle?) {
 *     super.onCreate(savedInstanceState)
 *     
 *     val webView = window.decorView.findViewById<WebView>(android.R.id.content)
 *     webView?.let { WebViewConfigurator.configure(it) }
 * }
 * ```
 *
 * ## Security Notes
 * - No INTERNET permission = no network access
 * - DOM storage is isolated to app sandbox (/data/user/0/se.malmo.skaggbyran.amp/)
 * - Mixed content mode only affects local assets (no remote resources can load)
 * - File access restricted to app's private directory
 *
 * @since 1.0.0
 * @see <a href="https://github.com/DioxusLabs/dioxus/issues/1875">Dioxus Issue #1875</a>
 */
object WebViewConfigurator {
    
    private const val TAG = "amp_WebViewConfig"
    
    /**
     * Configure WebView for offline Dioxus operation.
     *
     * @param webView The WebView to configure (typically from WRY/Dioxus)
     */
    @JvmStatic
    fun configure(webView: WebView) {
        Log.i(TAG, "Configuring WebView for offline Dioxus operation...")
        
        webView.settings.apply {
            // ========== CRITICAL: DOM Storage ==========
            // Required for Dioxus state management
            // Without this, localStorage/sessionStorage are blocked → blank screen
            domStorageEnabled = true
            Log.i(TAG, "  ✓ DOM storage enabled")
            
            // ========== JavaScript (should already be enabled by WRY) ==========
            javaScriptEnabled = true
            Log.i(TAG, "  ✓ JavaScript enabled")
            
            // ========== Database APIs ==========
            // Enable IndexedDB, WebSQL for advanced storage
            databaseEnabled = true
            Log.i(TAG, "  ✓ Database APIs enabled")
            
            // ========== Mixed Content ==========
            // Allow HTTP resources in HTTPS pages (for local assets)
            // Safe because no INTERNET permission = no remote loading
            mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
            Log.i(TAG, "  ✓ Mixed content allowed")
            
            // ========== File Access ==========
            // Allow access to bundled assets (HTML/CSS/JS/fonts)
            allowFileAccess = true
            allowContentAccess = true
            Log.i(TAG, "  ✓ File access enabled")
            
            // ========== Cache Mode ==========
            // Use default caching strategy
            cacheMode = WebSettings.LOAD_DEFAULT
            Log.i(TAG, "  ✓ Cache mode set to default")
            
            // ========== Zoom Controls ==========
            // Disable built-in zoom (Dioxus handles responsive layout)
            builtInZoomControls = false
            displayZoomControls = false
            Log.i(TAG, "  ✓ Zoom controls disabled")
            
            // ========== User Agent ==========
            // Keep default user agent (Dioxus doesn't need custom UA)
            // userAgentString = "Custom UA"  // Uncomment if needed
            Log.i(TAG, "  ✓ User agent: $userAgentString")
        }
        
        Log.i(TAG, "✅ WebView configuration complete")
        Log.i(TAG, "   DOM storage: ${webView.settings.domStorageEnabled}")
        Log.i(TAG, "   JavaScript: ${webView.settings.javaScriptEnabled}")
        Log.i(TAG, "   Database: ${webView.settings.databaseEnabled}")
    }
    
    /**
     * Verify WebView configuration for debugging.
     *
     * Logs current WebView settings to help diagnose blank screen issues.
     *
     * @param webView The WebView to inspect
     */
    @JvmStatic
    fun verify(webView: WebView) {
        Log.i(TAG, "=== WebView Configuration Verification ===")
        webView.settings.apply {
            Log.i(TAG, "DOM storage enabled: $domStorageEnabled")
            Log.i(TAG, "JavaScript enabled: $javaScriptEnabled")
            Log.i(TAG, "Database enabled: $databaseEnabled")
            Log.i(TAG, "Mixed content mode: $mixedContentMode")
            Log.i(TAG, "File access allowed: $allowFileAccess")
            Log.i(TAG, "Content access allowed: $allowContentAccess")
            Log.i(TAG, "Cache mode: $cacheMode")
            Log.i(TAG, "User agent: $userAgentString")
        }
        Log.i(TAG, "=========================================")
    }
}
