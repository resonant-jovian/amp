package se.malmo.skaggbyran.amp

/**
 * JNI bridge to Rust dormant check functions.
 *
 * This singleton loads the native library and exposes the Rust functions
 * for performing background parking restriction checks.
 */
object DormantBridge {

    init {
        System.loadLibrary("dioxusmain")
    }

    /**
     * Perform a dormant check and return notification data as JSON.
     *
     * Calls Rust's dormant_hourly_check() which:
     * 1. Reads stored addresses from parquet
     * 2. Detects parking restriction transitions
     * 3. Returns JSON array of notification objects
     *
     * @param storagePath Absolute path to app's files directory
     * @return JSON string: [{"channel_id":"...","notification_id":N,"title":"...","body":"..."}]
     */
    external fun dormantCheck(storagePath: String): String

    /**
     * Initialize the Rust storage path for dormant operations.
     *
     * Sets the APP_FILES_DIR environment variable in Rust so that
     * storage and settings modules can find their parquet files.
     *
     * @param storagePath Absolute path to app's files directory
     */
    external fun initDormantStorage(storagePath: String)
}
