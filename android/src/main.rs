//! Main entry point for Android app
#![allow(non_snake_case)]
mod android_bridge;
#[cfg(target_os = "android")]
mod android_utils;
mod components;
mod ui;
#[cfg(target_os = "android")]
mod webview_config;
fn main() {
    #[cfg(target_os = "android")]
    {
        if let Err(e) = android_utils::init_android_storage() {
            eprintln!("[Main] Failed to initialize Android storage: {}", e);
            eprintln!("[Main] App will use fallback storage paths");
        }
    }
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("amp"),
        );
    }
    #[cfg(not(target_os = "android"))]
    {
        env_logger::init();
    }
    log::info!("Starting Amp Android app");
    #[cfg(target_os = "android")]
    android_bridge::request_notification_permission_jni();
    #[cfg(target_os = "android")]
    {
        use dioxus::mobile::Config;
        dioxus::LaunchBuilder::new()
            .with_cfg(
                Config::new().with_custom_head(r#"<style>body { margin: 0; }</style>"#.to_string()),
            )
            .launch(ui::App);
    }
    #[cfg(not(target_os = "android"))]
    {
        dioxus::launch(ui::App);
    }
}
