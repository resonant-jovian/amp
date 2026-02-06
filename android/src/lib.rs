//! Android library exports
pub mod android_bridge;
#[cfg(target_os = "android")]
pub mod android_utils;
pub mod components;
pub mod ui;
#[cfg(target_os = "android")]
pub use android_utils::{get_android_files_dir, init_android_storage};
pub use components::{
    lifecycle::LifecycleManager,
    settings::{AppSettings, Language, Theme},
    storage,
};
