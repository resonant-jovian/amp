package se.malmo.skaggbyran.amp

import android.Manifest
import android.app.Activity
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import android.util.Log

/**
 * Helper class for managing Android notifications from Rust/JNI.
 *
 * Provides static methods to:
 * - Request notification permission (Android 13+)
 * - Create notification channels (Android 8.0+ / API 26+)
 * - Display notifications with proper priority and behavior
 *
 * All methods are thread-safe and can be called from Rust via JNI.
 *
 * Channel Configuration:
 * - amp_active: HIGH importance, sound + vibration + heads-up
 * - amp_six_hours: HIGH importance, sound + vibration
 * - amp_one_day: LOW importance, silent
 */
object NotificationHelper {
    private const val TAG = "AmpNotifications"
    private const val PERMISSION_REQUEST_CODE = 1001
    
    // Channel IDs matching Rust constants
    private const val CHANNEL_ACTIVE = "amp_active"
    private const val CHANNEL_SIX_HOURS = "amp_six_hours"
    private const val CHANNEL_ONE_DAY = "amp_one_day"

    /**
     * Request notification permission from user.
     *
     * For Android 13+: Shows system permission dialog for POST_NOTIFICATIONS
     * For Android <13: No-op (permission not required)
     *
     * @param activity The activity to request permission from (must be Activity, not Context)
     *
     * @JvmStatic annotation makes this callable from JNI
     */
    @JvmStatic
    fun requestNotificationPermission(activity: Activity) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            when {
                ContextCompat.checkSelfPermission(
                    activity,
                    Manifest.permission.POST_NOTIFICATIONS
                ) == PackageManager.PERMISSION_GRANTED -> {
                    Log.d(TAG, "Notification permission already granted")
                }
                ActivityCompat.shouldShowRequestPermissionRationale(
                    activity,
                    Manifest.permission.POST_NOTIFICATIONS
                ) -> {
                    // User previously denied, show explanation if needed
                    Log.d(TAG, "User previously denied notification permission")
                    ActivityCompat.requestPermissions(
                        activity,
                        arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                        PERMISSION_REQUEST_CODE
                    )
                }
                else -> {
                    // First time requesting
                    Log.d(TAG, "Requesting notification permission")
                    ActivityCompat.requestPermissions(
                        activity,
                        arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                        PERMISSION_REQUEST_CODE
                    )
                }
            }
        } else {
            Log.d(TAG, "Android < 13, notification permission not required")
        }
    }

    /**
     * Create notification channels for Android 8.0+ (API 26+).
     *
     * Safe to call multiple times - Android handles duplicate channel creation.
     * On Android < 8.0, this is a no-op since channels are not required.
     *
     * @param context Android application or activity context
     *
     * @JvmStatic annotation makes this callable from JNI
     */
    @JvmStatic
    fun createNotificationChannels(context: Context) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Log.d(TAG, "Creating notification channels for Android 8.0+")
            
            val notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) 
                as NotificationManager

            // Channel 1: Active parking restrictions (URGENT)
            val activeChannel = NotificationChannel(
                CHANNEL_ACTIVE,
                "Active Parking Restrictions",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Urgent alerts when street cleaning is currently active at your location"
                enableVibration(true)
                enableLights(true)
                setShowBadge(true)
                // Sound is enabled by default for HIGH importance
            }

            // Channel 2: 6-hour warnings (HIGH PRIORITY)
            val sixHoursChannel = NotificationChannel(
                CHANNEL_SIX_HOURS,
                "6-Hour Parking Warnings",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Warnings 6 hours before street cleaning begins"
                enableVibration(true)
                setShowBadge(true)
                // Sound is enabled by default for HIGH importance
            }

            // Channel 3: 1-day reminders (LOW PRIORITY - silent)
            val oneDayChannel = NotificationChannel(
                CHANNEL_ONE_DAY,
                "1-Day Parking Reminders",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Reminders 1 day before street cleaning"
                setSound(null, null) // Silent notifications
                enableVibration(false)
                setShowBadge(true)
            }

            // Register all channels
            notificationManager.createNotificationChannel(activeChannel)
            notificationManager.createNotificationChannel(sixHoursChannel)
            notificationManager.createNotificationChannel(oneDayChannel)
            
            Log.i(TAG, "Notification channels created successfully")
        } else {
            Log.d(TAG, "Skipping channel creation (Android < 8.0)")
        }
    }

    /**
     * Display a notification using the specified channel.
     *
     * The notification priority/behavior is determined by the channel:
     * - amp_active: Urgent with sound, vibration, heads-up
     * - amp_six_hours: High priority with sound and vibration
     * - amp_one_day: Low priority, silent
     *
     * @param context Android application or activity context
     * @param channelId One of: "amp_active", "amp_six_hours", "amp_one_day"
     * @param notificationId Unique ID for this notification (use address ID)
     * @param title Notification title text
     * @param body Notification body/content text
     *
     * @throws SecurityException if notification permission not granted (Android 13+)
     *
     * @JvmStatic annotation makes this callable from JNI
     */
    @JvmStatic
    fun showNotification(
        context: Context,
        channelId: String,
        notificationId: Int,
        title: String,
        body: String
    ) {
        Log.d(TAG, "Showing notification: channel=$channelId, id=$notificationId, title='$title'")

        try {
            // Determine priority based on channel for pre-Android 8.0 compatibility
            val priority = when (channelId) {
                CHANNEL_ACTIVE -> NotificationCompat.PRIORITY_HIGH
                CHANNEL_SIX_HOURS -> NotificationCompat.PRIORITY_HIGH
                CHANNEL_ONE_DAY -> NotificationCompat.PRIORITY_LOW
                else -> {
                    Log.w(TAG, "Unknown channel ID: $channelId, using default priority")
                    NotificationCompat.PRIORITY_DEFAULT
                }
            }

            // Build notification
            val builder = NotificationCompat.Builder(context, channelId)
                .setSmallIcon(android.R.drawable.ic_dialog_info) // TODO: Replace with app icon
                .setContentTitle(title)
                .setContentText(body)
                .setStyle(NotificationCompat.BigTextStyle().bigText(body))
                .setPriority(priority)
                .setAutoCancel(true) // Dismiss when tapped
                .setCategory(NotificationCompat.CATEGORY_REMINDER)

            // For active notifications, show as heads-up
            if (channelId == CHANNEL_ACTIVE) {
                builder.setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            }

            // Show notification
            with(NotificationManagerCompat.from(context)) {
                // Note: On Android 13+ (API 33+), this requires POST_NOTIFICATIONS permission
                // The app should request this permission at runtime
                notify(notificationId, builder.build())
            }
            
            Log.i(TAG, "Notification displayed successfully: id=$notificationId")
            
        } catch (e: SecurityException) {
            // Android 13+ requires POST_NOTIFICATIONS permission
            Log.e(TAG, "SecurityException: Missing notification permission", e)
            // The app should request android.permission.POST_NOTIFICATIONS at runtime
        } catch (e: Exception) {
            Log.e(TAG, "Failed to show notification: id=$notificationId", e)
        }
    }

    /**
     * Cancel a specific notification by ID.
     *
     * Useful for dismissing notifications when the user resolves the parking situation
     * or when an address is removed from the app.
     *
     * @param context Android application or activity context
     * @param notificationId The ID of the notification to cancel
     */
    @JvmStatic
    fun cancelNotification(context: Context, notificationId: Int) {
        Log.d(TAG, "Canceling notification: id=$notificationId")
        try {
            with(NotificationManagerCompat.from(context)) {
                cancel(notificationId)
            }
            Log.i(TAG, "Notification canceled: id=$notificationId")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to cancel notification: id=$notificationId", e)
        }
    }

    /**
     * Cancel all notifications from this app.
     *
     * Useful for clearing all parking notifications, e.g., when user
     * clears all addresses or disables notifications.
     */
    @JvmStatic
    fun cancelAllNotifications(context: Context) {
        Log.d(TAG, "Canceling all notifications")
        try {
            with(NotificationManagerCompat.from(context)) {
                cancelAll()
            }
            Log.i(TAG, "All notifications canceled")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to cancel all notifications", e)
        }
    }

    /**
     * Check if notification permission is granted (Android 13+ / API 33+).
     *
     * @param context Android application or activity context
     * @return true if permission granted or not required (Android < 13)
     */
    @JvmStatic
    fun hasNotificationPermission(context: Context): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            // Android 13+ requires runtime permission
            context.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) == 
                PackageManager.PERMISSION_GRANTED
        } else {
            // Older Android versions don't require runtime permission
            true
        }
    }
}
