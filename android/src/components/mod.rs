//! Business logic components for the Android parking app.
//!
//! This module contains the core functionality of the app, separated from
//! the UI layer for better testability and maintainability.
//!
//! # Module Organization
//!
//! ## Data Management
//! - [`storage`]: Persistent data storage using Parquet files
//! - [`static_data`]: Embedded parking restriction database
//! - [`settings`]: User preferences and app configuration
//!
//! ## Address Processing
//! - [`matching`]: Address validation and parking lookup
//! - [`address_utils`]: Address string manipulation utilities
//! - [`validity`]: Date-dependent restriction validity checking
//!
//! ## Time Management
//! - [`countdown`]: Real-time countdown to parking expiry
//!
//! ## Android Integration
//! - [`lifecycle`]: Activity lifecycle and background tasks
//! - [`notification`]: Push notification handling
//! - [`geo`]: GPS location services
//!
//! ## Development Tools
//! - [`debug`]: Debug UI and data inspection
//! - [`file`]: File system utilities
//!
//! # Quick Start
//!
//! Most apps will use these key components:
//!
//! ```no_run
//! use amp_android::components::{
//!     storage,
//!     matching::{match_address, MatchResult},
//!     validity::check_and_update_validity,
//!     lifecycle::LifecycleManager,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize lifecycle
//! let mut lifecycle = LifecycleManager::new();
//! lifecycle.start();
//!
//! // Load saved addresses
//! let mut addresses = storage::read_addresses_from_device();
//!
//! // Match new address
//! match match_address("Storgatan", "10", "22100") {
//!     MatchResult::Valid(entry) => println!("Found: {}", entry.adress),
//!     MatchResult::Invalid => println!("Not found"),
//! }
//!
//! // Check validity (e.g., for Feb 30)
//! if check_and_update_validity(&mut addresses) {
//!     storage::write_addresses_to_device(&addresses)?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! Components follow a layered architecture:
//!
//! ```text
//! UI Layer (Dioxus components)
//!     ↓
//! Business Logic (this module)
//!     ↓
//! Core Library (amp_core)
//! ```
//!
//! This separation allows:
//! - Unit testing without UI
//! - Code reuse across platforms
//! - Clear dependency boundaries
pub mod address_utils;
pub mod countdown;
pub mod debug;
pub mod file;
pub mod geo;
pub mod lifecycle;
pub mod matching;
pub mod notification;
pub mod settings;
pub mod static_data;
pub mod storage;
pub mod validity;
