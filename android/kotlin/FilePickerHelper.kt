package se.malmo.skaggbyran.amp

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.util.Log
import java.io.File
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/**
 * SAF (Storage Access Framework) helper for exporting and importing files.
 *
 * Uses startActivityForResult + CountDownLatch for synchronous JNI bridge calls.
 * This approach is deprecated but works on all API levels (min SDK 21) and avoids
 * the complexity of registerForActivityResult which requires registration in
 * onCreate before STARTED state.
 *
 * Called from android_bridge.rs export_file_jni() and import_file_jni().
 */
object FilePickerHelper {
    private const val TAG = "AmpFilePicker"
    const val REQUEST_CODE_EXPORT = 2001
    const val REQUEST_CODE_IMPORT = 2002
    private const val TIMEOUT_SECONDS = 120L

    private var exportLatch: CountDownLatch? = null
    private var importLatch: CountDownLatch? = null
    private var resultUri: Uri? = null
    private var resultCode: Int = Activity.RESULT_CANCELED

    /**
     * Export a file to a user-chosen location via SAF.
     *
     * Launches ACTION_CREATE_DOCUMENT, blocks until user picks location,
     * then copies file bytes to the chosen URI.
     *
     * @param context Activity context (must be an Activity for startActivityForResult)
     * @param sourcePath Absolute path to the file to export
     * @param suggestedName Suggested file name for the save dialog
     * @param mimeType MIME type for the file (use "application/octet-stream" for parquet)
     * @return "ok" on success, "error:<message>" on failure
     */
    @JvmStatic
    fun exportFile(context: Activity, sourcePath: String, suggestedName: String, mimeType: String): String {
        Log.d(TAG, "exportFile: source=$sourcePath, name=$suggestedName, mime=$mimeType")

        val sourceFile = File(sourcePath)
        if (!sourceFile.exists()) {
            val msg = "Source file not found: $sourcePath"
            Log.e(TAG, msg)
            return "error:$msg"
        }

        exportLatch = CountDownLatch(1)
        resultUri = null
        resultCode = Activity.RESULT_CANCELED

        try {
            val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = mimeType
                putExtra(Intent.EXTRA_TITLE, suggestedName)
            }

            context.runOnUiThread {
                try {
                    @Suppress("DEPRECATION")
                    context.startActivityForResult(intent, REQUEST_CODE_EXPORT)
                    Log.d(TAG, "Export SAF picker launched")
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to launch export picker: ${e.message}")
                    exportLatch?.countDown()
                }
            }

            // Block until user picks location or cancels
            val completed = exportLatch?.await(TIMEOUT_SECONDS, TimeUnit.SECONDS) ?: false
            if (!completed) {
                return "error:Timeout waiting for file picker"
            }

            if (resultCode != Activity.RESULT_OK || resultUri == null) {
                Log.d(TAG, "Export cancelled by user")
                return "error:cancelled"
            }

            // Write source file bytes to chosen URI
            val uri = resultUri!!
            context.contentResolver.openOutputStream(uri)?.use { outputStream ->
                sourceFile.inputStream().use { inputStream ->
                    inputStream.copyTo(outputStream)
                }
            } ?: return "error:Could not open output stream"

            Log.d(TAG, "Export successful to $uri")
            return "ok"

        } catch (e: Exception) {
            val msg = "Export failed: ${e.message}"
            Log.e(TAG, msg, e)
            return "error:$msg"
        }
    }

    /**
     * Import a file from a user-chosen location via SAF.
     *
     * Launches ACTION_OPEN_DOCUMENT, blocks until user picks file,
     * then copies it to a temp file in the cache directory.
     *
     * @param context Activity context
     * @param mimeType MIME type filter (use star/star for parquet since no standard MIME)
     * @return Temp file path on success, "" if cancelled, "error:<message>" on failure
     */
    @JvmStatic
    fun importFile(context: Activity, mimeType: String): String {
        Log.d(TAG, "importFile: mime=$mimeType")

        importLatch = CountDownLatch(1)
        resultUri = null
        resultCode = Activity.RESULT_CANCELED

        try {
            val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = mimeType
            }

            context.runOnUiThread {
                try {
                    @Suppress("DEPRECATION")
                    context.startActivityForResult(intent, REQUEST_CODE_IMPORT)
                    Log.d(TAG, "Import SAF picker launched")
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to launch import picker: ${e.message}")
                    importLatch?.countDown()
                }
            }

            // Block until user picks file or cancels
            val completed = importLatch?.await(TIMEOUT_SECONDS, TimeUnit.SECONDS) ?: false
            if (!completed) {
                return "error:Timeout waiting for file picker"
            }

            if (resultCode != Activity.RESULT_OK || resultUri == null) {
                Log.d(TAG, "Import cancelled by user")
                return ""
            }

            // Copy chosen file to temp in cache dir
            val uri = resultUri!!
            val tempFile = File(context.cacheDir, "import_temp_${System.currentTimeMillis()}.parquet")

            context.contentResolver.openInputStream(uri)?.use { inputStream ->
                tempFile.outputStream().use { outputStream ->
                    inputStream.copyTo(outputStream)
                }
            } ?: return "error:Could not open input stream"

            Log.d(TAG, "Import successful, temp file: ${tempFile.absolutePath}")
            return tempFile.absolutePath

        } catch (e: Exception) {
            val msg = "Import failed: ${e.message}"
            Log.e(TAG, msg, e)
            return "error:$msg"
        }
    }

    /**
     * Handle activity result from SAF file picker.
     *
     * Must be called from MainActivity.onActivityResult() for our request codes.
     *
     * @param requestCode The request code from onActivityResult
     * @param resultCode The result code (RESULT_OK or RESULT_CANCELED)
     * @param data The intent data containing the chosen URI
     * @return true if this result was handled (our request code), false otherwise
     */
    @JvmStatic
    fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?): Boolean {
        Log.d(TAG, "onActivityResult: requestCode=$requestCode, resultCode=$resultCode")

        return when (requestCode) {
            REQUEST_CODE_EXPORT -> {
                this.resultCode = resultCode
                this.resultUri = data?.data
                exportLatch?.countDown()
                true
            }
            REQUEST_CODE_IMPORT -> {
                this.resultCode = resultCode
                this.resultUri = data?.data
                importLatch?.countDown()
                true
            }
            else -> false
        }
    }
}
