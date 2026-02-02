pub mod addresses;
pub mod confirm_dialog;
pub mod info_dialog;
pub mod panels;
pub mod settings_dropdown;
pub mod top_bar;

use crate::components::address_utils::normalize_string;
use crate::components::matching::{match_address, MatchResult};
use crate::components::storage::{read_addresses_from_device, write_addresses_to_device};
use amp_core::structs::DB;
use dioxus::prelude::*;
use uuid::Uuid;

static CSS: Asset = asset!("/assets/style.css");

/// Maximum Levenshtein distance for fuzzy matching
/// Lower values = stricter matching
const FUZZY_MATCH_THRESHOLD: usize = 3;

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
#[derive(Clone, Debug, PartialEq)]
pub struct StoredAddress {
    /// Unique stable identifier (UUID v4)
    pub id: usize, // Keep as usize for backward compatibility with existing code
    /// Street name (e.g., "Storgatan")
    pub street: String,
    /// Street number (e.g., "10")
    pub street_number: String,
    /// Postal code (e.g., "22100")
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
    /// Generates a new UUID v4 for the address ID.
    ///
    /// # Arguments
    /// * `street` - Street name
    /// * `street_number` - Street number
    /// * `postal_code` - Postal code
    ///
    /// # Returns
    /// StoredAddress with unique UUID-based ID, validation result, and matched data (if found)
    pub fn new(street: String, street_number: String, postal_code: String) -> Self {
        let fuzzy_match_result = fuzzy_match_address(&street, &street_number, &postal_code);

        let (valid, matched_entry) = match fuzzy_match_result {
            Some(entry) => (true, Some(entry)),
            None => (false, None),
        };

        // Generate UUID and convert to usize for compatibility
        // Note: This is a hash-based approach, collisions are extremely unlikely
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
/// Collision probability is negligible for typical app usage.
fn uuid_to_usize(uuid: &Uuid) -> usize {
    let bytes = uuid.as_bytes();
    usize::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Fuzzy match address against database using Levenshtein distance
///
/// Implements multi-stage matching strategy:
/// 1. Exact match (normalized)
/// 2. Levenshtein distance matching (within threshold)
/// 3. Substring matching (fallback for partial addresses)
///
/// The Levenshtein distance measures the minimum number of single-character
/// edits (insertions, deletions, substitutions) needed to change one string
/// into another. This catches typos and minor variations.
///
/// # Arguments
/// * `street` - Street name
/// * `street_number` - Street number
/// * `postal_code` - Postal code
///
/// # Returns
/// Some(DB) if match found, None otherwise
///
/// # Examples
/// ```no_run
/// // Matches "Storgatan" with typos or case variations
/// let result = fuzzy_match_address("storgtan", "10", "22100"); // Missing 'a'
/// assert!(result.is_some());
///
/// let result = fuzzy_match_address("STORGATAN", "10", "22100"); // Uppercase
/// assert!(result.is_some());
/// ```
fn fuzzy_match_address(street: &str, street_number: &str, postal_code: &str) -> Option<DB> {
    // Try exact match first (fastest)
    match match_address(street, street_number, postal_code) {
        MatchResult::Valid(entry) => return Some(*entry),
        MatchResult::Invalid(_) => {}
    }

    use crate::components::matching::get_parking_data;
    let data = get_parking_data();

    // Normalize inputs
    let street_norm = normalize_string(street);
    let street_number_norm = normalize_string(street_number);
    let postal_code_norm = postal_code.trim().replace(' ', "");

    // Try Levenshtein distance matching
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

        // Calculate Levenshtein distance for street name
        let street_distance = strsim::levenshtein(&street_norm, &entry_street_norm);

        // Exact match on number and postal code, fuzzy on street name
        let street_match = if street_norm == entry_street_norm {
            true // Exact match
        } else if street_distance <= FUZZY_MATCH_THRESHOLD {
            true // Close enough (typos, minor variations)
        } else {
            // Fallback: substring match for partial addresses
            entry_street_norm.contains(&street_norm) || street_norm.contains(&entry_street_norm)
        };

        let number_match = entry_number_norm == street_number_norm;
        let postal_match = entry_postal_norm == postal_code_norm;

        if street_match && number_match && postal_match {
            eprintln!(
                "[FuzzyMatch] Found match: '{}' matches '{}' (distance: {})",
                street, entry_street_norm, street_distance
            );
            return Some(entry.clone());
        }
    }

    eprintln!(
        "[FuzzyMatch] No match found for: {} {} {}",
        street, street_number, postal_code
    );
    None
}

use crate::ui::{
    addresses::Addresses,
    panels::{ActivePanel, InvalidPanel, OneDayPanel, OneMonthPanel, SixHoursPanel},
    top_bar::TopBar,
};

/// Main application component
///
/// Manages a list of stored addresses and provides UI for:
/// - Adding new addresses
/// - Toggling address active state
/// - Removing addresses
/// - Displaying addresses in categorized panels by urgency
/// - Persisting addresses to local storage
#[component]
pub fn App() -> Element {
    let mut stored_addresses = use_signal::<Vec<StoredAddress>>(Vec::new));

    use_effect(move || {
        let loaded = read_addresses_from_device();
        if !loaded.is_empty() {
            info!("Loaded {} addresses from storage", loaded.len());
            stored_addresses.set(loaded);
        } else {
            info!("No saved addresses, adding examples");
            let examples = vec![
                StoredAddress::new(
                    "Storgatan".to_string(),
                    "1".to_string(),
                    "22100".to_string(),
                ),
                StoredAddress::new(
                    "Storgatan".to_string(),
                    "10".to_string(),
                    "22100".to_string(),
                ),
            ];
            if let Err(e) = write_addresses_to_device(&examples) {
                error!("Failed to save example addresses: {}", e);
            }
            stored_addresses.set(examples);
        }
    });

    let handle_add_address = move |args: (String, String, String)| {
        let (street, street_number, postal_code) = args;
        info!(
            "handle_add_address called with street='{}', street_number='{}', postal_code='{}'",
            street, street_number, postal_code
        );

        let new_addr = StoredAddress::new(street, street_number, postal_code);
        let mut addrs = stored_addresses.write();

        // Duplicate detection with normalized comparison
        let is_duplicate = addrs.iter().any(|a| {
            normalize_string(&a.street) == normalize_string(&new_addr.street)
                && normalize_string(&a.street_number) == normalize_string(&new_addr.street_number)
                && a.postal_code.trim().replace(' ', "") == new_addr.postal_code.trim().replace(' ', "")
        });

        if !is_duplicate {
            info!("Adding new address, total now: {}", addrs.len() + 1);
            addrs.push(new_addr);
            if let Err(e) = write_addresses_to_device(&addrs) {
                error!("Failed to persist addresses after add: {}", e);
            }
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
            if let Err(e) = write_addresses_to_device(&addrs) {
                error!("Failed to persist addresses after toggle: {}", e);
            }
        }
    };

    let handle_remove_address = move |id: usize| {
        info!("remove_address called for id {}", id);
        let mut addrs = stored_addresses.write();
        if let Some(pos) = addrs.iter().position(|a| a.id == id) {
            let removed = addrs.remove(pos);
            info!(
                "Removed address: {} {}, {}",
                removed.street, removed.street_number, removed.postal_code
            );
            if let Err(e) = write_addresses_to_device(&addrs) {
                error!("Failed to persist addresses after remove: {}", e);
            }
        }
    };

    rsx! {
        Stylesheet { href: CSS }
        TopBar { on_add_address: handle_add_address }
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
            InvalidPanel { addresses: stored_addresses.read().clone() }
        }
        script {}
    }
}
