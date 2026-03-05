//! iOS-specific utility functions
//!
//! Provides access to iOS system resources like internal storage directory
use std::path::PathBuf;
#[cfg(target_os = "ios")]
/// Get iOS internal files directory
///
/// Returns the app's Documents directory path, suitable for persistent storage.
///
/// # Returns
/// - `Ok(PathBuf)` - Path to the app's Documents directory
/// - `Err(anyhow::Error)` - Error if not yet implemented or path unavailable
///
/// # Examples
/// ```no_run
/// let files_dir = get_ios_files_dir()?;
/// println!("App storage: {:?}", files_dir);
/// ```
pub fn get_ios_files_dir() -> anyhow::Result<PathBuf> {
    anyhow::bail!("get_ios_files_dir not yet implemented for iOS")
}
#[cfg(not(target_os = "ios"))]
pub fn get_ios_files_dir() -> anyhow::Result<PathBuf> {
    anyhow::bail!("get_ios_files_dir only works on ios")
}
/// Initialize iOS storage directory as environment variable
///
/// This should be called once at app startup to set APP_FILES_DIR
/// for use by storage and settings modules.
#[cfg(target_os = "ios")]
pub fn init_ios_storage() -> anyhow::Result<()> {
    let files_dir = get_ios_files_dir()?;
    let files_dir_str = files_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert filesDir to string"))?;
    unsafe {
        std::env::set_var("APP_FILES_DIR", files_dir_str);
    }
    Ok(())
}
#[cfg(not(target_os = "ios"))]
pub fn init_ios_storage() -> anyhow::Result<()> {
    eprintln!("[iOSUtils] init_ios_storage skipped (not ios)");
    Ok(())
}
