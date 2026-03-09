//! iOS-specific utility functions
//!
//! Provides access to iOS system resources like internal storage directory
use std::path::PathBuf;
#[cfg(target_os = "ios")]
pub fn get_ios_files_dir() -> anyhow::Result<PathBuf> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use std::ffi::CStr;
    use std::os::raw::c_char;
    unsafe {
        let fm_class = objc2::class!(NSFileManager);
        let fm: *mut AnyObject = msg_send![fm_class, defaultManager];
        anyhow::ensure!(!fm.is_null(), "NSFileManager defaultManager returned nil");
        let urls: *mut AnyObject = msg_send![
            fm, URLsForDirectory : 9u64 inDomains : 1u64
        ];
        anyhow::ensure!(!urls.is_null(), "URLsForDirectory returned nil");
        let url: *mut AnyObject = msg_send![urls, firstObject];
        anyhow::ensure!(!url.is_null(), "No documents directory URL");
        let path_obj: *mut AnyObject = msg_send![url, path];
        anyhow::ensure!(!path_obj.is_null(), "Documents URL has no file path");
        let bytes: *const c_char = msg_send![path_obj, UTF8String];
        anyhow::ensure!(!bytes.is_null(), "UTF8String returned nil");
        let path_str = CStr::from_ptr(bytes)
            .to_str()
            .map_err(|e| anyhow::anyhow!("Path is not valid UTF-8: {}", e))?;
        Ok(PathBuf::from(path_str))
    }
}
#[cfg(not(target_os = "ios"))]
pub fn get_ios_files_dir() -> anyhow::Result<PathBuf> {
    anyhow::bail!("get_ios_files_dir only works on iOS")
}
#[cfg(target_os = "ios")]
pub fn init_ios_storage() -> anyhow::Result<()> {
    let files_dir = get_ios_files_dir()?;
    let files_dir_str = files_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert filesDir to string"))?;
    unsafe {
        std::env::set_var("APP_FILES_DIR", files_dir_str);
    }
    eprintln!("[iOSUtils] APP_FILES_DIR set to: {}", files_dir_str);
    Ok(())
}
#[cfg(not(target_os = "ios"))]
pub fn init_ios_storage() -> anyhow::Result<()> {
    eprintln!("[iOSUtils] init_ios_storage skipped (not ios)");
    Ok(())
}
