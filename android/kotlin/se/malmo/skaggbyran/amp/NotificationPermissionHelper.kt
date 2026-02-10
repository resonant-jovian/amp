package se.malmo.skaggbyran.amp

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

/**
 * Helper class for requesting notification permissions on Android 13+
 *
 * Android 13 (API 33) requires runtime permission for POST_NOTIFICATIONS.
 * This class handles the permission request flow.
 */
object NotificationPermissionHelper {
    private const val TAG = "NotificationPermission"
    private const val PERMISSION_REQUEST_CODE = 1001

    /**
     * Check if notification permission is granted
     *
     * @param activity The activity context
     * @return true if permission granted or not needed (Android < 13)
     */
    fun hasNotificationPermission(activity: Activity): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            ContextCompat.checkSelfPermission(
                activity,
                Manifest.permission.POST_NOTIFICATIONS
            ) == PackageManager.PERMISSION_GRANTED
        } else {
            // Android 12 and below don't need runtime permission
            true
        }
    }

    /**
     * Request notification permission if needed
     *
     * For Android 13+: Shows system permission dialog
     * For Android 12 and below: No-op (permission not required)
     *
     * @param activity The activity to request permission from
     */
    fun requestNotificationPermission(activity: Activity) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            when {
                hasNotificationPermission(activity) -> {
                    Log.d(TAG, "Notification permission already granted")
                }
                ActivityCompat.shouldShowRequestPermissionRationale(
                    activity,
                    Manifest.permission.POST_NOTIFICATIONS
                ) -> {
                    // User previously denied permission
                    // You could show an explanation dialog here before requesting again
                    Log.d(TAG, "User previously denied notification permission")
                    ActivityCompat.requestPermissions(
                        activity,
                        arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                        PERMISSION_REQUEST_CODE
                    )
                }
                else -> {
                    // First time requesting permission
                    Log.d(TAG, "Requesting notification permission")
                    ActivityCompat.requestPermissions(
                        activity,
                        arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                        PERMISSION_REQUEST_CODE
                    )
                }
            }
        } else {
            Log.d(TAG, "Android version < 13, notification permission not required")
        }
    }

    /**
     * Handle permission result callback
     *
     * Call this from Activity.onRequestPermissionsResult()
     *
     * @param requestCode The request code from onRequestPermissionsResult
     * @param grantResults The grant results array
     * @return true if notification permission was granted
     */
    fun handlePermissionResult(
        requestCode: Int,
        grantResults: IntArray
    ): Boolean {
        if (requestCode == PERMISSION_REQUEST_CODE) {
            val granted = grantResults.isNotEmpty() &&
                    grantResults[0] == PackageManager.PERMISSION_GRANTED
            
            if (granted) {
                Log.i(TAG, "✅ Notification permission GRANTED")
            } else {
                Log.w(TAG, "❌ Notification permission DENIED")
            }
            
            return granted
        }
        return false
    }
}
