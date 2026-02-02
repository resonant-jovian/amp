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
    let mut stored_addresses = use_signal::<Vec<StoredAddress>>(Vec::new());

    use_effect(move || {
        let loaded = read_addresses_from_device();
        if !loaded.is_empty() {
            info!("Loaded {} addresses from storage", loaded.len());
            stored_addresses.set(loaded);
        } else {
            info!("No saved addresses, adding debug test addresses");
            let examples = vec![
                StoredAddress::new("Kornettsgatan".to_string(), "18C".to_string(), "21150".to_string()), // dag 1
                StoredAddress::new("Claesgatan".to_string(), "2B".to_string(), "21426".to_string()), // dag 2
                StoredAddress::new("Östra Kristinelundsvägen".to_string(), "27D".to_string(), "21748".to_string()), // dag 3
                StoredAddress::new("Karlskronaplan".to_string(), "3".to_string(), "21436".to_string()), // dag 4
                StoredAddress::new("Västra Rönneholmsvägen".to_string(), "76C".to_string(), "21741".to_string()), // dag 5
                StoredAddress::new("Vitemöllegatan".to_string(), "11A".to_string(), "21442".to_string()), // dag 6
                StoredAddress::new("Docentgatan".to_string(), "1B".to_string(), "21552".to_string()), // dag 7
                StoredAddress::new("Eriksfältsgatan".to_string(), "98B".to_string(), "21550".to_string()), // dag 8
                StoredAddress::new("Lantmannagatan".to_string(), "50 U1".to_string(), "21448".to_string()), // dag 9
                StoredAddress::new("Pysslinggatan".to_string(), "4".to_string(), "21238".to_string()), // dag 10
                StoredAddress::new("Celsiusgatan".to_string(), "13A U1".to_string(), "21214".to_string()), // dag 11
                StoredAddress::new("Kapellgatan".to_string(), "14 U4".to_string(), "21421".to_string()), // dag 12
                StoredAddress::new("Tegnérgatan".to_string(), "25B".to_string(), "21614".to_string()), // dag 13
                StoredAddress::new("S:t Pauli kyrkogata".to_string(), "13B".to_string(), "21149".to_string()), // dag 14
                StoredAddress::new("Östra Stallmästaregatan".to_string(), "18B".to_string(), "21749".to_string()), // dag 15
                StoredAddress::new("Södervärnsgatan".to_string(), "9B U1".to_string(), "21427".to_string()), // dag 16
                StoredAddress::new("Carl Hillsgatan".to_string(), "10B".to_string(), "21756".to_string()), // dag 17
                StoredAddress::new("Köpenhamnsvägen".to_string(), "46A".to_string(), "21771".to_string()), // dag 18
                StoredAddress::new("Bangatan".to_string(), "13".to_string(), "21426".to_string()), // dag 19
                StoredAddress::new("Smålandsgatan".to_string(), "20A".to_string(), "21430".to_string()), // dag 20
                StoredAddress::new("Tycho Brahegatan".to_string(), "26".to_string(), "21612".to_string()), // dag 21
                StoredAddress::new("Storgatan".to_string(), "43K".to_string(), "21142".to_string()), // dag 22
                StoredAddress::new("Östergårdsgatan".to_string(), "1 U13".to_string(), "21222".to_string()), // dag 23
                StoredAddress::new("Byggmästaregatan".to_string(), "5".to_string(), "21130".to_string()), // dag 24
                StoredAddress::new("Lantmannagatan".to_string(), "11A".to_string(), "21444".to_string()), // dag 25
                StoredAddress::new("Zenithgatan".to_string(), "42C".to_string(), "21214".to_string()), // dag 26
                StoredAddress::new("Bragegatan".to_string(), "37B".to_string(), "21446".to_string()), // dag 27
                StoredAddress::new("Idunsgatan".to_string(), "67B".to_string(), "21446".to_string()), // dag 28
                StoredAddress::new("Värnhemsgatan".to_string(), "2A".to_string(), "21215".to_string()), // dag 29
                StoredAddress::new("Sånekullavägen".to_string(), "36A".to_string(), "21774".to_string()), // dag 30
                StoredAddress::new("Amiralsgatan".to_string(), "83E".to_string(), "21437".to_string()), // ingen städning men parkering
                StoredAddress::new("Docentgatan".to_string(), "3A".to_string(), "21552".to_string()), // städning men ingen parkerings avgift
                StoredAddress::new("Låssasgatan".to_string(), "11A".to_string(), "11111".to_string()), // false street
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
