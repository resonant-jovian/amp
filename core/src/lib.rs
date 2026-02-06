//! Address-matched parking data management for Swedish cities.
//!
//! `amp-core` provides the foundational data structures and operations for managing
//! parking restrictions, environmental parking rules, and address matching in Swedish
//! municipalities. It handles data persistence using Apache Parquet format and provides
//! efficient correlation between addresses and parking regulations.
//!
//! # Key Concepts
//!
//! ## Data Sources
//!
//! The library works with three primary data types from GeoJSON sources:
//! - **Addresses** ([`AdressClean`]): Street addresses with coordinates and postal codes
//! - **Environmental Parking** ([`MiljoeDataClean`]): Time-restricted parking for street cleaning
//! - **Parking Zones** ([`ParkeringsDataClean`]): Paid parking zones with pricing tiers (taxa)
//!
//! ## Storage Format
//!
//! All data is persisted in Apache Parquet format for efficient storage and querying:
//! - [`LocalData`]: User's saved addresses with matched parking information
//! - [`SettingsData`]: Application preferences (notifications, theme, language)
//! - [`DB`]: Time-aware parking restriction entries with Swedish timezone support
//!
//! ## Time Handling
//!
//! Times are always stored in UTC but interpreted in Swedish timezone ([`SWEDISH_TZ`]):
//! - Automatic handling of summer/winter time transitions
//! - Year validation (2020-2100) to prevent overflow
//! - Support for checking if restrictions are currently active
//!
//! # Examples
//!
//! ## Loading GeoJSON Data
//!
//! ```no_run
//! use amp_core::api::DataLoader;
//!
//! let addresses = DataLoader::load_addresses("data/adresser.json")?;
//! let parking = DataLoader::load_parkering("data/parkeringsavgifter.json")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Creating a Time-Based Restriction
//!
//! ```
//! use amp_core::structs::DB;
//!
//! // Create a parking restriction for January 15, 2024, from 08:00 to 12:00
//! let restriction = DB::from_dag_tid(
//!     Some("21438".to_string()),
//!     "Storgatan 10".to_string(),
//!     Some("Storgatan".to_string()),
//!     Some("10".to_string()),
//!     Some("Street cleaning".to_string()),
//!     15,                    // Day of month
//!     "0800-1200",          // Time range in HHMM-HHMM format
//!     Some("Taxa C".to_string()),
//!     Some(26),
//!     Some("Längsgående 6".to_string()),
//!     2024,                 // Year
//!     1,                    // Month
//! );
//! assert!(restriction.is_some());
//! ```
//!
//! ## Persisting User Data
//!
//! ```no_run
//! use amp_core::parquet::{build_local_parquet, read_local_parquet_from_bytes};
//! use amp_core::structs::LocalData;
//!
//! // Build parquet data in memory
//! let data = vec![/* LocalData entries */];
//! # let data: Vec<LocalData> = vec![];
//! let parquet_bytes = build_local_parquet(data)?;
//!
//! // Later, read it back
//! let loaded_data = read_local_parquet_from_bytes(&parquet_bytes)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Module Organization
//!
//! - [`api`]: GeoJSON data loading from external files
//! - [`parquet`]: Parquet file I/O for all data structures
//! - [`structs`]: Core data structures and time-based logic
//! - [`checksum`]: File integrity verification for data validation
//! - [`benchmark`]: Performance measurement utilities
//! - [`correlation_algorithms`]: Spatial matching algorithms (address ↔ parking data)
//!
//! [`AdressClean`]: structs::AdressClean
//! [`MiljoeDataClean`]: structs::MiljoeDataClean
//! [`ParkeringsDataClean`]: structs::ParkeringsDataClean
//! [`LocalData`]: structs::LocalData
//! [`SettingsData`]: structs::SettingsData
//! [`DB`]: structs::DB
//! [`SWEDISH_TZ`]: structs::SWEDISH_TZ
pub mod api;
pub mod benchmark;
pub mod checksum;
pub mod correlation;
pub mod correlation_algorithms;
#[cfg(test)]
mod correlation_tests;
pub mod error;
pub mod parquet;
pub mod structs;
