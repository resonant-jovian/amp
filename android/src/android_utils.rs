//! Android-specific utility functions
//!
//! Provides access to Android system resources like internal storage directory
use std::path::PathBuf;
#[cfg(target_os = "android")]
/// Get Android internal files directory using JNI
///
/// This function uses Dioxus's dispatch mechanism to call Android's getFilesDir()
/// through JNI, returning the absolute path to the app's internal storage.
///
/// # Returns
/// - `Ok(PathBuf)` - Path to /data/data/<package>/files/
/// - `Err(String)` - Error message if JNI call fails
///
/// # Examples
/// ```no_run
/// let files_dir = get_android_files_dir()?;
/// println!("App storage: {:?}", files_dir);
/// ```
pub fn get_android_files_dir() -> anyhow::Result<PathBuf> {
    use jni::JNIEnv;
    use jni::objects::{JObject, JString};
    let (tx, rx) = std::sync::mpsc::channel();
    fn run(env: &mut JNIEnv<'_>, activity: &JObject<'_>) -> anyhow::Result<PathBuf> {
        let files_dir = env
            .call_method(activity, "getFilesDir", "()Ljava/io/File;", &[])?
            .l()?;
        let files_dir: JString<'_> = env
            .call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?
            .into();
        let files_dir: String = env.get_string(&files_dir)?.into();
        eprintln!("[AndroidUtils] Got filesDir from JNI: {}", files_dir);
        Ok(PathBuf::from(files_dir))
    }
    dioxus::mobile::wry::prelude::dispatch(move |env, activity, _webview| {
        tx.send(run(env, activity)).unwrap()
    });
    rx.recv().unwrap()
}
#[cfg(not(target_os = "android"))]
pub fn get_android_files_dir() -> anyhow::Result<PathBuf> {
    anyhow::bail!("get_android_files_dir only works on Android")
}
/// Initialize Android storage directory as environment variable
///
/// This should be called once at app startup to set APP_FILES_DIR
/// for use by storage and settings modules.
#[cfg(target_os = "android")]
pub fn init_android_storage() -> anyhow::Result<()> {
    let files_dir = get_android_files_dir()?;
    let files_dir_str = files_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert filesDir to string"))?;
    unsafe {
        std::env::set_var("APP_FILES_DIR", files_dir_str);
    }
    eprintln!("[AndroidUtils] Set APP_FILES_DIR={}", files_dir_str);
    Ok(())
}
#[cfg(not(target_os = "android"))]
pub fn init_android_storage() -> anyhow::Result<()> {
    eprintln!("[AndroidUtils] init_android_storage skipped (not Android)");
    Ok(())
}
