//! Android-specific implementation for the Amp parking app.
//!
//! This crate provides the Android UI layer built with Dioxus, along with
//! platform-specific utilities for storage, lifecycle management, and native
//! Android integration through JNI.
//!
//! # Architecture
//!
//! The Android app follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │   Dioxus UI Components (ui/)        │  ← User interface
//! ├─────────────────────────────────────┤
//! │   Business Logic (components/)      │  ← App logic, state management
//! ├─────────────────────────────────────┤
//! │   Android Bridge (android_bridge)   │  ← JNI integration
//! ├─────────────────────────────────────┤
//! │   Core Library (amp_core)           │  ← Parking correlation engine
//! └─────────────────────────────────────┘
//! ```
//!
//! # Main Modules
//!
//! ## UI Layer (`ui`)
//!
//! Dioxus components for the user interface:
//! - **Home screen**: Address search and parking status display
//! - **Settings**: Theme, language, notification preferences
//! - **Debug**: Developer tools and data inspection
//!
//! ## Business Logic (`components`)
//!
//! Core app functionality:
//! - [`storage`]: Persistent data management with Parquet files
//! - [`lifecycle::LifecycleManager`]: Android activity lifecycle handling
//! - [`settings`]: App configuration (theme, language, notifications)
//! - [`matching`]: Address-to-parking correlation logic
//! - [`validity`]: Parking restriction time validity checking
//! - [`countdown`]: Real-time countdown to parking expiry
//!
//! ## Platform Integration
//!
//! - [`android_bridge`]: JNI functions for Kotlin/Java interop
//! - [`android_utils`]: Android-specific file system access
//!
//! # Quick Start
//!
//! ## Initializing the App
//!
//! ```no_run
//! use amp_android::{
//!     LifecycleManager,
//!     storage::StorageManager,
//!     AppSettings,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize lifecycle management
//! let lifecycle = LifecycleManager::new();
//!
//! // Load app settings
//! let settings = AppSettings::load().await?;
//!
//! // Initialize storage system
//! let storage = StorageManager::new("/path/to/app/files").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Configuring WebView (Android)
//!
//! ```no_run
//! use amp_android::webview_config;
//!
//! # #[cfg(target_os = "android")]
//! # {
//! // Enable DOM storage for offline Dioxus operation
//! if let Err(e) = webview_config::configure_webview_dom_storage() {
//!     log::error!("WebView config failed: {}", e);
//! }
//! # }
//! ```
//!
//! ## Loading Parking Data
//!
//! ```no_run
//! use amp_android::storage::StorageManager;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = StorageManager::new("/data").await?;
//!
//! // Load parking zones from embedded Parquet files
//! let parking_data = storage.load_parkering_data().await?;
//! let env_data = storage.load_miljoe_data().await?;
//!
//! println!("Loaded {} parking zones", parking_data.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Searching for Address Restrictions
//!
//! ```no_run
//! use amp_android::components::matching::find_matching_parkering;
//! use amp_core::structs::AdressClean;
//!
//! # async fn example(address: AdressClean) -> Result<(), Box<dyn std::error::Error>> {
//! # let parking_data = vec![];
//! // Find parking restrictions for an address
//! let result = find_matching_parkering(&address, &parking_data).await;
//!
//! match result {
//!     Some(restriction) => println!("Found restriction: {:?}", restriction),
//!     None => println!("No parking restrictions at this address"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Android Integration
//!
//! ## JNI Bridge
//!
//! The [`android_bridge`] module exposes Rust functions to Kotlin/Java:
//!
//! ```kotlin
//! // Kotlin example
//! class MainActivity : ComponentActivity() {
//!     external fun rustInitialize(): Boolean
//!     external fun rustSearchAddress(address: String): String
//!
//!     companion object {
//!         init {
//!             System.loadLibrary("amp_android")
//!         }
//!     }
//! }
//! ```
//!
//! ## File System Access
//!
//! Android apps have restricted file system access. Use [`android_utils`] to get
//! the correct paths:
//!
//! ```no_run
//! # #[cfg(target_os = "android")]
//! # {
//! use amp_android::get_android_files_dir;
//!
//! // Get app-specific storage directory
//! let files_dir = get_android_files_dir();
//! println!("Storing data in: {}", files_dir);
//! # }
//! ```
//!
//! ## WebView Configuration
//!
//! For offline operation without INTERNET permission, explicitly enable DOM storage:
//!
//! ```no_run
//! # #[cfg(target_os = "android")]
//! # {
//! use amp_android::webview_config;
//!
//! // Called after Dioxus initializes WebView
//! if let Err(e) = webview_config::configure_webview_dom_storage() {
//!     log::error!("Failed to enable DOM storage: {}", e);
//! }
//! # }
//! ```
//!
//! # Data Storage
//!
//! The app stores data in Apache Parquet format for efficiency:
//! - **Addresses**: 20,000+ Malmö addresses with coordinates
//! - **Parking zones**: ~2,000 parking restriction line segments
//! - **Settings**: User preferences (JSON)
//! - **Cache**: Computed correlation results
//!
//! Parquet files are embedded in the APK and copied to app storage on first run.
//!
//! # Features
//!
//! ## Core Features
//! - Real-time address search with fuzzy matching
//! - Parking restriction lookup and validation
//! - Time-based validity checking (considers Swedish holidays)
//! - Countdown timer to parking expiry
//! - Multi-language support (Swedish, English)
//! - Dark/light theme support
//!
//! ## Platform Features
//! - Android lifecycle-aware background processing
//! - Notification support for parking expiry warnings
//! - Offline-first design (all data embedded)
//! - Fast startup (<1s on modern devices)
//! - WebView DOM storage for Dioxus state management
//!
//! # Performance
//!
//! - **Cold start**: ~0.5-1.0s (Parquet loading + index building)
//! - **Address search**: ~10-50ms for fuzzy matching
//! - **Correlation**: ~0.01-0.05ms with R-tree spatial index
//! - **Memory**: ~15-30 MB total (including UI)
//!
//! # Build Requirements
//!
//! - Rust 2024 edition
//! - Android NDK 26+
//! - Cargo NDK for cross-compilation
//! - Dioxus CLI for mobile builds
//!
//! # See Also
//!
//! - [`amp_core`]: Core parking correlation engine
//! - [`storage`]: Data persistence and Parquet handling
//! - [`LifecycleManager`]: Android lifecycle management
//! - [`AppSettings`]: Configuration management
pub mod android_bridge;
#[cfg(target_os = "android")]
pub mod android_utils;
pub mod components;
pub mod ui;
#[cfg(target_os = "android")]
#[cfg(target_os = "android")]
pub use android_utils::{get_android_files_dir, init_android_storage};
pub use components::{
    lifecycle::LifecycleManager,
    settings::{AppSettings, Language, Theme},
    storage,
};
