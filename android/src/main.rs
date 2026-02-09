//! Main entry point for Android app
#![allow(non_snake_case)]
use dioxus::prelude::*;
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
    {
        log::info!("[Main] Spawning WebView configuration thread...");
        std::thread::spawn(|| {
            log::debug!("[Main] Waiting 300ms for WebView creation...");
            std::thread::sleep(std::time::Duration::from_millis(300));
            log::info!("[Main] Configuring WebView DOM storage...");
            match webview_config::configure_webview_dom_storage() {
                Ok(()) => {
                    log::info!("[Main] ✅ WebView configuration successful!");
                    log::info!("[Main] DOM storage enabled - Dioxus should render");
                    match webview_config::verify_dom_storage_enabled() {
                        Ok(true) => {
                            log::info!("[Main] ✅ Verification: DOM storage is enabled")
                        }
                        Ok(false) => {
                            log::warn!("[Main] ⚠️  Verification: DOM storage still disabled")
                        }
                        Err(e) => log::warn!("[Main] ⚠️  Verification failed: {}", e),
                    }
                }
                Err(e) => {
                    log::error!("[Main] ❌ WebView configuration FAILED: {}", e);
                    log::error!("[Main] App will show blank screen without DOM storage");
                    log::error!("[Main] Workaround: Add INTERNET permission temporarily");
                }
            }
        });
        log::info!("[Main] WebView configuration thread spawned");
    }
    log::info!("[Main] Launching Dioxus...");
    launch(ui::App);
}
