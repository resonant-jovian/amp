//! Main entry point for ios app
#![allow(non_snake_case)]
mod components;
mod ios_bridge;
#[cfg(target_os = "ios")]
mod ios_utils;
mod ui;
fn main() {
    #[cfg(target_os = "ios")]
    {
        if let Err(e) = ios_utils::init_ios_storage() {
            eprintln!("[Main] Failed to initialize ios storage: {}", e);
            eprintln!("[Main] App will use fallback storage paths");
        }
    }
    #[cfg(target_os = "ios")]
    {
        oslog::OsLogger::new("se.malmo.skaggbyran.amp")
            .level_filter(log::LevelFilter::Debug)
            .init()
            .unwrap_or_else(|e| eprintln!("[Main] Failed to init oslog: {}", e));
    }
    #[cfg(not(target_os = "ios"))]
    {
        env_logger::init();
    }
    log::info!("Starting Amp ios app");
    #[cfg(target_os = "ios")]
    ios_bridge::request_notification_permission();
    #[cfg(target_os = "ios")]
    ios_bridge::start_dormant_service();
    #[cfg(target_os = "ios")]
    {
        use dioxus::mobile::Config;
        dioxus::LaunchBuilder::new()
            .with_cfg(
                Config::new().with_custom_head(r#"<style>body { margin: 0; }</style>"#.to_string()),
            )
            .launch(ui::App);
    }
    #[cfg(not(target_os = "ios"))]
    {
        dioxus::launch(ui::App);
    }
}
