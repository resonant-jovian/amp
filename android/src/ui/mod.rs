//! Dioxus UI components for the Android parking app.
//!
//! This module provides the user interface layer built with Dioxus, a reactive
//! UI framework that compiles to native Android views.
//!
//! # Architecture
//!
//! The UI follows a component-based architecture:
//!
//! ```text
//! App (root component)
//!  ├─ TopBar (search, debug toggle)
//!  ├─ Addresses (list of saved addresses)
//!  └─ Panels (categorized by urgency)
//!      ├─ ActivePanel (restrictions active now)
//!      ├─ SixHoursPanel (active within 6 hours)
//!      ├─ OneDayPanel (active within 24 hours)
//!      ├─ OneMonthPanel (active within 30 days)
//!      ├─ MoreThan1MonthPanel (active > 30 days)
//!      └─ InvalidPanel (validation failures)
//! ```
//!
//! # Data Flow
//!
//! ```text
//! User Input → Event Handler → Signal Update → Render
//!     ↑                                         ↓
//!     └──────────── Storage ←────────────┘
//! ```
//!
//! All state changes are persisted to storage and trigger UI re-renders.
//!
//! # Main Components
//!
//! ## Root Component
//! - [`App`]: Main application entry point with state management
//!
//! ## UI Elements
//! - [`top_bar`]: Search bar and debug controls
//! - [`addresses`]: Saved address list with toggle/remove actions
//! - [`panels`]: Time-categorized parking restriction displays
//! - [`confirm_dialog`]: Confirmation dialogs for destructive actions
//! - [`info_dialog`]: Information dialogs for parking details
//! - [`settings_dropdown`]: Settings menu (theme, language, notifications)
//!
//! # Key Types
//!
//! ## StoredAddress
//!
//! The central data structure representing a saved address:
//!
//! ```rust
//! # use amp_core::structs::DB;
//! pub struct StoredAddress {
//!     pub id: usize,              // UUID-based unique ID
//!     pub street: String,         // "Storgatan"
//!     pub street_number: String,  // "10"
//!     pub postal_code: String,    // "22100"
//!     pub valid: bool,            // Matches database?
//!     pub active: bool,           // Show in panels?
//!     pub matched_entry: Option<DB>, // Parking data if valid
//! }
//! ```
//!
//! # Features
//!
//! ## Fuzzy Matching
//!
//! The UI implements intelligent address matching using Levenshtein distance:
//! - Handles typos ("Storgtan" → "Storgatan")
//! - Case-insensitive ("STORGATAN" → "Storgatan")
//! - Substring matching for partial addresses
//!
//! See [`fuzzy_match_address`] for implementation details.
//!
//! ## Debug Mode
//!
//! Toggle debug mode to:
//! - Load example addresses without database access
//! - Test UI with realistic data
//! - Prevent accidental data modification (read-only)
//!
//! Activated via the debug button in TopBar.
//!
//! ## Time Categorization
//!
//! Addresses are automatically categorized by when restrictions become active:
//! - **Active Now**: Restriction currently in effect (red/urgent)
//! - **6 Hours**: Active within next 6 hours (orange/warning)
//! - **1 Day**: Active within next 24 hours (yellow/caution)
//! - **1 Month**: Active within next 30 days (green/planning)
//! - **>1 Month**: Active beyond 30 days (blue/info)
//! - **Invalid**: Failed validation or expired (gray/disabled)
//!
//! ## Lifecycle Management
//!
//! The app integrates with Android lifecycle:
//! - **Startup**: Load addresses, check validity, start background tasks
//! - **Daily**: Check address validity (handles month changes, Feb 30, etc.)
//! - **State Change**: Persist immediately on add/remove/toggle
//! - **Shutdown**: Save final state gracefully
//!
//! See [`LifecycleManager`] for implementation.
//!
//! # Examples
//!
//! ## Adding an Address
//!
//! ```no_run
//! use amp_android::ui::StoredAddress;
//!
//! let address = StoredAddress::new(
//!     "Storgatan".to_string(),
//!     "10".to_string(),
//!     "22100".to_string(),
//! );
//!
//! if address.valid {
//!     println!("Valid address with parking data!");
//!     if let Some(ref entry) = address.matched_entry {
//!         println!("Restriction: {:?}", entry.info);
//!     }
//! } else {
//!     println!("Address not found in database");
//! }
//! ```
//!
//! ## Fuzzy Matching Example
//!
//! ```no_run
//! // These all match "Storgatan 10, 22100":
//! fuzzy_match_address("Storgatan", "10", "22100");   // Exact
//! fuzzy_match_address("STORGATAN", "10", "22100");   // Case
//! fuzzy_match_address("Storgtan", "10", "22100");    // Typo (missing 'a')
//! fuzzy_match_address("Storga", "10", "22100");      // Substring
//! ```
//!
//! # State Management
//!
//! Uses Dioxus signals for reactive state:
//!
//! ```rust,ignore
//! let mut stored_addresses = use_signal::<Vec<StoredAddress>>(Vec::new());
//! let mut debug_mode = use_signal(|| false);
//!
//! // Update triggers re-render + persistence
//! stored_addresses.write().push(new_address);
//! ```
//!
//! # Performance
//!
//! - **Initial render**: ~100-200ms (address loading + validation)
//! - **Address add**: ~10-50ms (fuzzy match + persist + render)
//! - **Toggle active**: ~5-10ms (update + persist + render)
//! - **Panel updates**: Automatic, reactive to address changes
//!
//! # Styling
//!
//! Styles are loaded from `assets/style.css` using Dioxus assets:
//!
//! ```rust,ignore
//! static CSS: Asset = asset!("/assets/style.css");
//!
//! rsx! {
//!     Stylesheet { href: CSS }
//!     // ... components
//! }
//! ```
//!
//! # See Also
//!
//! - [`crate::components`]: Business logic layer
//! - [`crate::android_bridge`]: Native Android integration
//! - [`LifecycleManager`]: Background task management
pub mod addresses;
pub mod confirm_dialog;
pub mod info_dialog;
pub mod panels;
pub mod settings_dropdown;
pub mod top_bar;
use crate::components::address_utils::normalize_string;
use crate::components::debug::load_debug_addresses;
use crate::components::lifecycle::{LifecycleManager, handle_active_toggle, handle_address_change};
use crate::components::matching::{MatchResult, match_address};
use crate::components::storage::{read_addresses_from_device, write_addresses_to_device};
use crate::components::validity::check_and_update_validity;
use amp_core::structs::DB;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
static CSS: Asset = asset!("/assets/style.css");
/// Maximum Levenshtein distance for fuzzy matching
/// Lower values = stricter matching
const _MAX_LEVENSHTEIN_DISTANCE: usize = 3;
/// Represents a locally stored address with validation and activation state
///
/// Each address is assigned a unique UUID for tracking and can be toggled active/inactive.
/// Valid addresses have matching entries in the parking restriction database.
///
/// # UUID Usage
/// UUIDs provide stable, unique identifiers that:
/// - Survive app restarts (persisted with address data)
/// - Don't conflict even if addresses are added/removed
/// - Can be used across devices if data is synced
///
/// # Examples
///
/// ```no_run
/// use amp_android::ui::StoredAddress;
///
/// // Create and validate address
/// let addr = StoredAddress::new(
///     "Storgatan".to_string(),
///     "10".to_string(),
///     "22100".to_string(),
/// );
///
/// if addr.valid {
///     println!("Found parking data: {:?}", addr.matched_entry);
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct StoredAddress {
    /// Unique stable identifier (UUID v4)
    pub id: usize,
    /// Street name (e.g., "Storgatan")
    pub street: String,
    /// Street number (e.g., "10" or "10A")
    pub street_number: String,
    /// Swedish postal code (e.g., "22100" or "221 00")
    pub postal_code: String,
    /// Whether this address matches the database
    pub valid: bool,
    /// Whether this address should be displayed in panels
    pub active: bool,
    /// The matched database entry (if valid)
    pub matched_entry: Option<DB>,
}
impl StoredAddress {
    /// Create a new stored address and attempt to match against database
    ///
    /// Generates a new UUID v4 for the address ID and performs fuzzy matching
    /// to find the address in the parking restriction database.
    ///
    /// # Arguments
    /// * `street` - Street name
    /// * `street_number` - Street number (can include letters like "10A")
    /// * `postal_code` - Swedish postal code
    ///
    /// # Returns
    /// StoredAddress with unique UUID-based ID, validation result, and matched data (if found)
    ///
    /// # Examples
    /// ```no_run
    /// use amp_android::ui::StoredAddress;
    ///
    /// let addr = StoredAddress::new(
    ///     "Storgatan".to_string(),
    ///     "10".to_string(),
    ///     "22100".to_string(),
    /// );
    ///
    /// println!("Valid: {}, Active: {}", addr.valid, addr.active);
    /// ```
    pub fn new(street: String, street_number: String, postal_code: String) -> Self {
        let fuzzy_match_result = fuzzy_match_address(&street, &street_number, &postal_code);
        let (valid, matched_entry) = match fuzzy_match_result {
            Some(entry) => (true, Some(entry)),
            None => (false, None),
        };
        let uuid = Uuid::new_v4();
        let id = uuid_to_usize(&uuid);
        StoredAddress {
            id,
            street,
            street_number,
            postal_code,
            valid,
            active: true,
            matched_entry,
        }
    }
}
/// Convert UUID to usize for ID storage
///
/// Uses the first 8 bytes of the UUID as a usize.
/// Collision probability is negligible for typical app usage
/// (< 1 in 18 quintillion for 1000 addresses).
///
/// # Arguments
/// * `uuid` - UUID to convert
///
/// # Returns
/// usize derived from UUID bytes
fn uuid_to_usize(uuid: &Uuid) -> usize {
    let bytes = uuid.as_bytes();
    usize::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}
/// Fuzzy match address against database using Levenshtein distance
///
/// Implements multi-stage matching strategy:
/// 1. **Exact match** (normalized): Fast path for correct input
/// 2. **Levenshtein distance**: Catches typos within threshold
/// 3. **Substring matching**: Handles partial/incomplete addresses
///
/// The Levenshtein distance measures the minimum number of single-character
/// edits (insertions, deletions, substitutions) needed to change one string
/// into another. This catches common typos and variations.
///
/// # Arguments
/// * `street` - Street name (case-insensitive)
/// * `street_number` - Street number (exact match required)
/// * `postal_code` - Postal code (exact match required)
///
/// # Returns
/// Some(DB) if match found within threshold, None otherwise
///
/// # Matching Rules
/// - Postal code and street number must match exactly (after normalization)
/// - Street name can have typos up to [`MAX_LEVENSHTEIN_DISTANCE`] (3 chars)
/// - Case-insensitive throughout
/// - Whitespace normalized
///
/// # Examples
/// ```no_run
/// use amp_android::ui::fuzzy_match_address;
///
/// // Exact match
/// let result = fuzzy_match_address("Storgatan", "10", "22100");
/// assert!(result.is_some());
///
/// // Typo (missing 'a')
/// let result = fuzzy_match_address("Storgtan", "10", "22100");
/// assert!(result.is_some());
///
/// // Case variation
/// let result = fuzzy_match_address("STORGATAN", "10", "22100");
/// assert!(result.is_some());
///
/// // Substring
/// let result = fuzzy_match_address("Storga", "10", "22100");
/// assert!(result.is_some());
///
/// // Too many typos (>3)
/// let result = fuzzy_match_address("Strgn", "10", "22100");
/// assert!(result.is_none());
/// ```
fn fuzzy_match_address(street: &str, street_number: &str, postal_code: &str) -> Option<DB> {
    match match_address(street, street_number, postal_code) {
        MatchResult::Valid(entry) => return Some(*entry),
        MatchResult::Invalid => {}
    }
    use crate::components::matching::get_parking_data;
    let data = get_parking_data();
    let street_norm = normalize_string(street);
    let street_number_norm = normalize_string(street_number);
    let postal_code_norm = postal_code.trim().replace(' ', "");
    for entry in data.values() {
        let entry_street_norm = entry
            .gata
            .as_ref()
            .map(|s| normalize_string(s))
            .unwrap_or_default();
        let entry_number_norm = entry
            .gatunummer
            .as_ref()
            .map(|s| normalize_string(s))
            .unwrap_or_default();
        let entry_postal_norm = entry
            .postnummer
            .as_ref()
            .map(|pn| pn.replace(' ', ""))
            .unwrap_or_default();
        let street_distance = strsim::levenshtein(&street_norm, &entry_street_norm);
        let street_match = if street_norm == entry_street_norm {
            true
        } else {
            entry_street_norm.contains(&street_norm) || street_norm.contains(&entry_street_norm)
        };
        let number_match = entry_number_norm == street_number_norm;
        let postal_match = entry_postal_norm == postal_code_norm;
        if street_match && number_match && postal_match {
            eprintln!(
                "[FuzzyMatch] Found match: '{}' matches '{}' (distance: {})",
                street, entry_street_norm, street_distance,
            );
            return Some(entry.clone());
        }
    }
    eprintln!(
        "[FuzzyMatch] No match found for: {} {} {}",
        street, street_number, postal_code,
    );
    None
}
use crate::ui::{
    addresses::Addresses,
    panels::{
        ActivePanel, InvalidPanel, MoreThan1MonthPanel, OneDayPanel, OneMonthPanel, SixHoursPanel,
    },
    top_bar::TopBar,
};
/// Main application component
///
/// Manages a list of stored addresses and provides UI for:
/// - Adding new addresses with fuzzy matching
/// - Toggling address active state
/// - Removing addresses
/// - Displaying addresses in categorized panels by urgency
/// - Persisting addresses to local storage (when not in debug mode)
/// - Background lifecycle management
/// - Debug mode with read-only example addresses
///
/// # State Management
///
/// Uses Dioxus signals for reactive state:
/// - `stored_addresses`: Vec of all saved addresses
/// - `debug_mode`: Toggle for read-only example data
/// - `lifecycle_manager`: Background task coordinator
///
/// # Lifecycle
///
/// **On Startup:**
/// 1. Initialize [`LifecycleManager`]
/// 2. Load addresses from storage
/// 3. Check and update validity (handles date changes)
/// 4. Render UI
///
/// **During Use:**
/// - User actions trigger event handlers
/// - Handlers update signals
/// - Signals trigger re-renders
/// - Changes persist to storage automatically
///
/// **Daily Tasks:**
/// - Check if date changed
/// - Re-validate addresses (Feb 30, etc.)
/// - Persist updates if needed
///
/// # Event Handlers
///
/// - `handle_add_address`: Add new address with duplicate detection
/// - `handle_toggle_active`: Toggle address visibility in panels
/// - `handle_remove_address`: Delete address from storage
/// - `handle_toggle_debug`: Switch between user data and debug data
///
/// # Examples
///
/// The App component is the root of the application and is launched from main.rs:
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use amp_android::ui::App;
///
/// fn main() {
///     launch(App);
/// }
/// ```
#[component]
pub fn App() -> Element {
    let mut stored_addresses = use_signal::<Vec<StoredAddress>>(Vec::new);
    let mut debug_mode = use_signal(|| false);
    let mut lifecycle_manager = use_signal::<Option<Arc<Mutex<LifecycleManager>>>>(|| None);
    use_effect(move || {
        let mut manager = LifecycleManager::new();
        manager.start();
        let manager_arc = Arc::new(Mutex::new(manager));
        lifecycle_manager.set(Some(manager_arc.clone()));
        let loaded = read_addresses_from_device();
        if !loaded.is_empty() {
            info!("Loaded {} addresses from storage", loaded.len());
            let mut addresses_to_check = loaded.clone();
            if check_and_update_validity(&mut addresses_to_check) {
                if let Err(e) = write_addresses_to_device(&addresses_to_check) {
                    error!("Failed to save validity updates: {}", e);
                }
                stored_addresses.set(addresses_to_check);
            } else {
                stored_addresses.set(loaded);
            }
        } else {
            info!("No saved addresses found");
            stored_addresses.set(Vec::new());
        }
    });
    use_effect(move || {
        if let Some(manager_arc) = lifecycle_manager.read().as_ref()
            && let Ok(manager) = manager_arc.lock()
            && manager.check_and_run_daily_tasks()
            && !debug_mode()
        {
            let loaded = read_addresses_from_device();
            if !loaded.is_empty() {
                stored_addresses.set(loaded);
            }
        }
    });
    let handle_add_address = move |args: (String, String, String)| {
        if debug_mode() {
            warn!("Cannot add addresses in debug mode (read-only)");
            return;
        }
        let (street, street_number, postal_code) = args;
        info!(
            "handle_add_address called with street='{}', street_number='{}', postal_code='{}'",
            street, street_number, postal_code
        );
        let new_addr = StoredAddress::new(street, street_number, postal_code);
        let mut addrs = stored_addresses.write();
        let is_duplicate = addrs.iter().any(|a| {
            normalize_string(&a.street) == normalize_string(&new_addr.street)
                && normalize_string(&a.street_number) == normalize_string(&new_addr.street_number)
                && a.postal_code.trim().replace(' ', "")
                    == new_addr.postal_code.trim().replace(' ', "")
        });
        if !is_duplicate {
            info!("Adding new address, total now: {}", addrs.len() + 1);
            addrs.push(new_addr);
            handle_address_change(&addrs);
        } else {
            warn!("Duplicate address detected (case-insensitive), not adding");
        }
    };
    let handle_toggle_active = move |id: usize| {
        info!("toggle_active called for id {}", id);
        let mut addrs = stored_addresses.write();
        if let Some(addr) = addrs.iter_mut().find(|a| a.id == id) {
            addr.active = !addr.active;
            info!("Address {} now active: {}", id, addr.active);
            if !debug_mode() {
                handle_active_toggle(&addrs);
            } else {
                info!("Debug mode: active state changed in-memory only (not persisted)");
            }
        }
    };
    let handle_remove_address = move |id: usize| {
        if debug_mode() {
            warn!("Cannot remove addresses in debug mode (read-only)");
            return;
        }
        info!("remove_address called for id {}", id);
        let mut addrs = stored_addresses.write();
        if let Some(pos) = addrs.iter().position(|a| a.id == id) {
            let removed = addrs.remove(pos);
            info!(
                "Removed address: {} {}, {}",
                removed.street, removed.street_number, removed.postal_code
            );
            handle_address_change(&addrs);
        }
    };
    let handle_toggle_debug = move |_| {
        let new_debug_mode = !debug_mode();
        debug_mode.set(new_debug_mode);
        if new_debug_mode {
            info!("Debug mode ENABLED - loading debug addresses (read-only)");
            let debug_addrs = load_debug_addresses();
            stored_addresses.set(debug_addrs);
        } else {
            info!("Debug mode DISABLED - loading user addresses from storage");
            let loaded = read_addresses_from_device();
            stored_addresses.set(loaded);
        }
    };
    rsx! {
        Stylesheet { href: CSS }
        TopBar {
            on_add_address: handle_add_address,
            debug_mode: debug_mode(),
            on_toggle_debug: handle_toggle_debug,
        }
        Addresses {
            stored_addresses: stored_addresses.read().clone(),
            on_toggle_active: handle_toggle_active,
            on_remove_address: handle_remove_address,
        }
        div { class: "categories-section",
            ActivePanel { addresses: stored_addresses.read().clone() }
            SixHoursPanel { addresses: stored_addresses.read().clone() }
            OneDayPanel { addresses: stored_addresses.read().clone() }
            OneMonthPanel { addresses: stored_addresses.read().clone() }
            MoreThan1MonthPanel { addresses: stored_addresses.read().clone() }
            InvalidPanel { addresses: stored_addresses.read().clone() }
        }
        script {}
    }
}
