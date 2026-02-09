package se.malmo.skaggbyran.amp

import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import dev.dioxus.main.MainActivity as DioxusMainActivity

/**
 * Custom MainActivity that extends Dioxus's auto-generated MainActivity.
 * 
 * This class adds WebView DOM storage configuration to fix the blank screen
 * issue when running Dioxus apps without INTERNET permission.
 * 
 * ## Problem
 * Dioxus requires localStorage to mount components, but Android WebView
 * disables DOM storage when apps don't have INTERNET permission.
 * 
 * ## Solution  
 * After WebView creation, call WebViewConfigurator.configure() to enable
 * localStorage/sessionStorage APIs without requiring network access.
 * 
 * ## Usage
 * This file will be copied to the gradle build directory by build.sh
 * and replace the auto-generated MainActivity.
 */
class MainActivity : DioxusMainActivity() {
    
    companion object {
        private const val TAG = "amp_MainActivity"
        private const val WEBVIEW_CONFIG_DELAY_MS = 300L
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        Log.i(TAG, "MainActivity created, scheduling WebView configuration...")
        
        // Schedule WebView configuration after a delay to ensure WebView is created
        // Dioxus creates WebView asynchronously, so we need to wait a bit
        Handler(Looper.getMainLooper()).postDelayed({
            try {
                val webView = findWebViewInHierarchy(window.decorView)
                
                if (webView != null) {
                    Log.i(TAG, "WebView found, applying configuration...")
                    WebViewConfigurator.configure(webView)
                    Log.i(TAG, "✅ WebView configured successfully")
                } else {
                    Log.w(TAG, "⚠️  WebView not found in view hierarchy")
                    Log.w(TAG, "Will retry configuration attempt after another delay...")
                    
                    // Retry once after longer delay
                    Handler(Looper.getMainLooper()).postDelayed({
                        val retryWebView = findWebViewInHierarchy(window.decorView)
                        if (retryWebView != null) {
                            Log.i(TAG, "WebView found on retry, applying configuration...")
                            WebViewConfigurator.configure(retryWebView)
                            Log.i(TAG, "✅ WebView configured successfully (retry)")
                        } else {
                            Log.e(TAG, "❌ WebView still not found after retry")
                            Log.e(TAG, "App may show blank screen without DOM storage")
                        }
                    }, WEBVIEW_CONFIG_DELAY_MS * 2)
                }
            } catch (e: Exception) {
                Log.e(TAG, "❌ Failed to configure WebView: ${e.message}", e)
            }
        }, WEBVIEW_CONFIG_DELAY_MS)
    }
    
    /**
     * Recursively search the view hierarchy for a WebView instance.
     * 
     * @param view The root view to start searching from
     * @return The first WebView found, or null if none exists
     */
    private fun findWebViewInHierarchy(view: View): WebView? {
        if (view is WebView) {
            return view
        }
        
        if (view is ViewGroup) {
            for (i in 0 until view.childCount) {
                val child = view.getChildAt(i)
                val webView = findWebViewInHierarchy(child)
                if (webView != null) {
                    return webView
                }
            }
        }
        
        return null
    }
}
