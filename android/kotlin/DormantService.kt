package se.malmo.skaggbyran.amp

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.util.Log
import org.json.JSONArray
import java.util.Calendar

/**
 * Foreground service that performs hourly dormant checks for parking restriction transitions.
 *
 * When the user closes the app, this service continues running in the background.
 * Every hour (aligned to :00), it calls Rust via JNI to check if any stored addresses
 * have transitioned to a more urgent parking restriction bucket, and sends notifications.
 *
 * Lifecycle:
 * - Started by MainActivity on app launch (idempotent)
 * - Started by BootReceiver on device boot
 * - Runs indefinitely via START_STICKY + foreground notification
 * - Only killed by force stop; restarts on next boot or app open
 */
class DormantService : Service() {

    companion object {
        private const val TAG = "AmpDormant"
        private const val DORMANT_CHANNEL_ID = "amp_dormant"
        private const val DORMANT_NOTIFICATION_ID = 9999
        private const val FUZZY_MINUTES = 5

        @Volatile
        private var isRunning = false

        fun isServiceRunning(): Boolean = isRunning
    }

    private val handler = Handler(Looper.getMainLooper())

    private val hourlyRunnable = object : Runnable {
        override fun run() {
            performDormantCheck()
            scheduleNextCheck()
        }
    }

    override fun onCreate() {
        super.onCreate()
        Log.i(TAG, "DormantService created")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (isRunning) {
            Log.i(TAG, "DormantService already running, ignoring start")
            return START_STICKY
        }

        Log.i(TAG, "DormantService starting foreground")
        isRunning = true

        // Create the dormant notification channel
        createDormantChannel()

        // Ensure parking notification channels exist too
        NotificationHelper.createNotificationChannels(this)

        // Start as foreground service with persistent notification
        val notification = buildPersistentNotification()
        startForeground(DORMANT_NOTIFICATION_ID, notification)

        // Initialize Rust storage path
        initRustStorage()

        // Schedule first check
        scheduleNextCheck()

        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        Log.i(TAG, "DormantService destroyed")
        isRunning = false
        handler.removeCallbacks(hourlyRunnable)
    }

    /**
     * Create the dedicated notification channel for the persistent service notification.
     * This is separate from the parking notification channels.
     */
    private fun createDormantChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                DORMANT_CHANNEL_ID,
                "Bakgrundsövervakning",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Visar att AMP övervakar din parkering i bakgrunden"
                setSound(null, null)
                enableVibration(false)
                setShowBadge(false)
            }

            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
            Log.d(TAG, "Dormant notification channel created")
        }
    }

    /**
     * Build the persistent foreground notification.
     */
    private fun buildPersistentNotification(): Notification {
        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, DORMANT_CHANNEL_ID)
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
        }

        return builder
            .setContentTitle("AMP övervakar din parkering")
            .setContentText("Du får notiser vid parkeringsrestriktioner")
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setOngoing(true)
            .build()
    }

    /**
     * Initialize Rust storage path via JNI.
     */
    private fun initRustStorage() {
        try {
            val storagePath = filesDir.absolutePath
            Log.d(TAG, "Initializing Rust storage: $storagePath")
            DormantBridge.initDormantStorage(storagePath)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to init Rust storage", e)
        }
    }

    /**
     * Schedule the next hourly check aligned to the hour mark.
     *
     * Calculates milliseconds until the next :00 and posts a delayed runnable.
     * If we're within FUZZY_MINUTES of the hour, runs immediately and schedules for next hour.
     */
    private fun scheduleNextCheck() {
        val now = System.currentTimeMillis()
        val cal = Calendar.getInstance().apply {
            add(Calendar.HOUR_OF_DAY, 1)
            set(Calendar.MINUTE, 0)
            set(Calendar.SECOND, 0)
            set(Calendar.MILLISECOND, 0)
        }

        var delayMs = cal.timeInMillis - now

        // If we're within FUZZY_MINUTES of the next hour, check now and schedule for after
        val minutesUntilHour = delayMs / 60_000
        if (minutesUntilHour <= FUZZY_MINUTES) {
            Log.d(TAG, "Within $FUZZY_MINUTES min of hour mark, running check now")
            performDormantCheck()
            // Schedule for the next hour after that
            cal.add(Calendar.HOUR_OF_DAY, 1)
            delayMs = cal.timeInMillis - now
        }

        Log.i(TAG, "Next dormant check in ${delayMs / 60_000} minutes")
        handler.postDelayed(hourlyRunnable, delayMs)
    }

    /**
     * Perform the actual dormant check by calling Rust via JNI.
     *
     * 1. Calls DormantBridge.dormantCheck(storagePath) → JSON string
     * 2. Parses JSON array of notification objects
     * 3. Sends each notification via NotificationHelper
     */
    private fun performDormantCheck() {
        Log.i(TAG, "Performing dormant check...")

        try {
            val storagePath = filesDir.absolutePath
            val json = DormantBridge.dormantCheck(storagePath)

            Log.d(TAG, "Dormant check result: $json")

            val notifications = JSONArray(json)
            Log.i(TAG, "Got ${notifications.length()} notifications from Rust")

            for (i in 0 until notifications.length()) {
                val notif = notifications.getJSONObject(i)
                val channelId = notif.getString("channel_id")
                val notificationId = notif.getInt("notification_id")
                val title = notif.getString("title")
                val body = notif.getString("body")

                Log.d(TAG, "Sending notification: channel=$channelId, id=$notificationId, title=$title")
                NotificationHelper.showNotification(this, channelId, notificationId, title, body)
            }
        } catch (e: Exception) {
            Log.e(TAG, "Dormant check failed", e)
        }
    }
}
