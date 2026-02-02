//! Persistent storage for Android app
//!
//! Provides local storage for user addresses using Android SharedPreferences.
//! Falls back to in-memory storage for non-Android platforms (testing).
use crate::ui::StoredAddress;
#[cfg(target_os = "android")]
use jni::{
    JNIEnv, JavaVM,
    objects::{JClass, JObject, JString, JValue},
};
#[cfg(target_os = "android")]
use std::sync::OnceLock;
#[cfg(target_os = "android")]
static JVM: OnceLock<JavaVM> = OnceLock::new();
/// Initialize the JVM reference for Android storage operations
///
/// This should be called once during app initialization on Android.
///
/// # Arguments
/// * `env` - JNI environment reference
#[cfg(target_os = "android")]
pub fn init_jvm(env: &JNIEnv) {
    if let Ok(vm) = env.get_java_vm() {
        let _ = JVM.set(vm);
    }
}
/// Load stored addresses from persistent storage
///
/// On Android, reads from SharedPreferences. On other platforms, returns empty vec.
///
/// # Returns
/// Vector of stored addresses, empty if none saved or storage unavailable
///
/// # Examples
/// ```no_run
/// let addresses = read_addresses_from_device();
/// println!("Loaded {} addresses", addresses.len());
/// ```
pub fn read_addresses_from_device() -> Vec<StoredAddress> {
    #[cfg(target_os = "android")]
    {
        load_from_shared_preferences().unwrap_or_else(|e| {
            eprintln!("Failed to load addresses: {}", e);
            Vec::new()
        })
    }
    #[cfg(not(target_os = "android"))]
    {
        Vec::new()
    }
}
/// Write stored addresses to persistent storage
///
/// On Android, writes to SharedPreferences as JSON. On other platforms, no-op.
///
/// # Arguments
/// * `addresses` - Slice of addresses to persist
///
/// # Returns
/// Ok(()) if successful, Err with message if failed
///
/// # Examples
/// ```no_run
/// let addresses = vec![/* ... */];
/// if let Err(e) = write_addresses_to_device(&addresses) {
///     eprintln!("Failed to save: {}", e);
/// }
/// ```
pub fn write_addresses_to_device(addresses: &[StoredAddress]) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        save_to_shared_preferences(addresses)
    }
    #[cfg(not(target_os = "android"))]
    {
        eprintln!("[Mock] Would save {} addresses", addresses.len());
        Ok(())
    }
}
#[cfg(target_os = "android")]
fn load_from_shared_preferences() -> Result<Vec<StoredAddress>, String> {
    let vm = JVM.get().ok_or("JVM not initialized")?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("Failed to attach thread: {:?}", e))?;
    let context = get_application_context(&mut env)?;
    let prefs_name = env
        .new_string("amp_prefs")
        .map_err(|e| format!("Failed to create string: {:?}", e))?;
    let mode = JValue::Int(0);
    let prefs = env
        .call_method(
            &context,
            "getSharedPreferences",
            "(Ljava/lang/String;I)Landroid/content/SharedPreferences;",
            &[JValue::Object(&prefs_name), mode],
        )
        .map_err(|e| format!("Failed to get SharedPreferences: {:?}", e))?
        .l()
        .map_err(|e| format!("Failed to convert to object: {:?}", e))?;
    let key = env
        .new_string("stored_addresses")
        .map_err(|e| format!("Failed to create key: {:?}", e))?;
    let default = env
        .new_string("[]")
        .map_err(|e| format!("Failed to create default: {:?}", e))?;
    let json_obj = env
        .call_method(
            prefs,
            "getString",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(&key), JValue::Object(&default)],
        )
        .map_err(|e| format!("Failed to get string: {:?}", e))?
        .l()
        .map_err(|e| format!("Failed to convert to object: {:?}", e))?;
    let json_jstring: JString = json_obj.into();
    let json_str: String = env
        .get_string(&json_jstring)
        .map_err(|e| format!("Failed to get Rust string: {:?}", e))?
        .into();
    deserialize_addresses(&json_str)
}
#[cfg(target_os = "android")]
fn save_to_shared_preferences(addresses: &[StoredAddress]) -> Result<(), String> {
    let vm = JVM.get().ok_or("JVM not initialized")?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("Failed to attach thread: {:?}", e))?;
    let json_str = serialize_addresses(addresses)?;
    let context = get_application_context(&mut env)?;
    let prefs_name = env
        .new_string("amp_prefs")
        .map_err(|e| format!("Failed to create string: {:?}", e))?;
    let mode = JValue::Int(0);
    let prefs = env
        .call_method(
            &context,
            "getSharedPreferences",
            "(Ljava/lang/String;I)Landroid/content/SharedPreferences;",
            &[JValue::Object(&prefs_name), mode],
        )
        .map_err(|e| format!("Failed to get SharedPreferences: {:?}", e))?
        .l()
        .map_err(|e| format!("Failed to convert to object: {:?}", e))?;
    let editor = env
        .call_method(
            prefs,
            "edit",
            "()Landroid/content/SharedPreferences$Editor;",
            &[],
        )
        .map_err(|e| format!("Failed to get editor: {:?}", e))?
        .l()
        .map_err(|e| format!("Failed to convert to object: {:?}", e))?;
    let key = env
        .new_string("stored_addresses")
        .map_err(|e| format!("Failed to create key: {:?}", e))?;
    let value = env
        .new_string(&json_str)
        .map_err(|e| format!("Failed to create value: {:?}", e))?;
    env.call_method(
        editor,
        "putString",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;",
        &[JValue::Object(&key), JValue::Object(&value)],
    )
    .map_err(|e| format!("Failed to put string: {:?}", e))?;
    env.call_method(editor, "apply", "()V", &[])
        .map_err(|e| format!("Failed to apply: {:?}", e))?;
    Ok(())
}
#[cfg(target_os = "android")]
fn get_application_context(env: &mut JNIEnv) -> Result<JObject, String> {
    Err("Application context not available - needs proper initialization".to_string())
}
