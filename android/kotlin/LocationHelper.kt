package se.malmo.skaggbyran.amp

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.pm.PackageManager
import android.location.LocationManager
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import android.util.Log

/**
 * Helper class for reading GPS location from Rust/JNI.
 *
 * Provides static methods to:
 * - Check and request ACCESS_FINE_LOCATION / ACCESS_COARSE_LOCATION permissions
 * - Read last known location from GPS, network, or passive provider
 *
 * All methods are thread-safe and callable from Rust via JNI.
 * Called from android_bridge.rs get_android_location().
 */
object LocationHelper {
    private const val TAG = "AmpLocation"
    private const val LOCATION_REQUEST_CODE = 1002

    /**
     * Check whether the app has fine or coarse location permission.
     *
     * @param context Android context
     * @return true if at least coarse location is granted
     */
    @JvmStatic
    fun hasPermission(context: Context): Boolean {
        return ContextCompat.checkSelfPermission(
            context,
            Manifest.permission.ACCESS_FINE_LOCATION
        ) == PackageManager.PERMISSION_GRANTED ||
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.ACCESS_COARSE_LOCATION
            ) == PackageManager.PERMISSION_GRANTED
    }

    /**
     * Request location permission from user.
     *
     * Shows system permission dialog for ACCESS_FINE_LOCATION and
     * ACCESS_COARSE_LOCATION. The caller must be an Activity.
     *
     * @param activity The activity to request permission from
     */
    @JvmStatic
    fun requestLocationPermission(activity: Activity) {
        ActivityCompat.requestPermissions(
            activity,
            arrayOf(
                Manifest.permission.ACCESS_FINE_LOCATION,
                Manifest.permission.ACCESS_COARSE_LOCATION
            ),
            LOCATION_REQUEST_CODE
        )
        Log.d(TAG, "Location permission requested")
    }

    /**
     * Get the last known device location.
     *
     * Returns a "latitude,longitude" string or empty string if unavailable.
     * Tries GPS_PROVIDER first (most accurate), then NETWORK_PROVIDER,
     * then PASSIVE_PROVIDER as fallback.
     *
     * If permission is not yet granted, requests it (when context is an Activity)
     * and returns empty string — the user must tap the GPS button again after
     * granting the permission.
     *
     * @param context Android context (Activity or Application)
     * @return "lat,lon" decimal string, or "" if no location is available
     */
    @JvmStatic
    fun getLocation(context: Context): String {
        if (!hasPermission(context)) {
            Log.w(TAG, "Location permission not granted — requesting")
            if (context is Activity) {
                requestLocationPermission(context)
            }
            return ""
        }

        val locationManager = context.getSystemService(Context.LOCATION_SERVICE) as? LocationManager
            ?: run {
                Log.e(TAG, "LocationManager unavailable")
                return ""
            }

        val providers = listOf(
            LocationManager.GPS_PROVIDER,
            LocationManager.NETWORK_PROVIDER,
            LocationManager.PASSIVE_PROVIDER
        )

        for (provider in providers) {
            try {
                if (!locationManager.isProviderEnabled(provider)) continue

                @Suppress("MissingPermission")
                val location = locationManager.getLastKnownLocation(provider)
                if (location != null) {
                    val result = "${location.latitude},${location.longitude}"
                    Log.d(TAG, "Got location from $provider: $result")
                    return result
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error getting location from $provider: ${e.message}")
            }
        }

        Log.w(TAG, "No location available from any provider")
        return ""
    }
}
