package se.malmo.skaggbyran.amp

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log

/**
 * BroadcastReceiver that starts DormantService on device boot.
 *
 * Listens for BOOT_COMPLETED intent and launches the foreground service
 * so parking restriction monitoring resumes after a reboot.
 */
class BootReceiver : BroadcastReceiver() {

    companion object {
        private const val TAG = "AmpBoot"
    }

    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED) {
            Log.i(TAG, "Boot completed, starting DormantService")

            val serviceIntent = Intent(context, DormantService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(serviceIntent)
            } else {
                context.startService(serviceIntent)
            }
        }
    }
}
