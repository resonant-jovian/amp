//! Main entry point for Android app
#![allow(non_snake_case)]
use dioxus::prelude::*;
mod android_bridge;
#[cfg(target_os = "android")]
mod android_utils;
mod components;
mod ui;
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
    launch(ui::App);
}
