pub mod addresses;
pub mod panels;
pub mod top_bar;
use crate::matching::{MatchResult, match_address};
use crate::static_data::StaticAddressEntry;
use crate::storage::{read_addresses_from_device, write_addresses_to_device};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
static CSS: Asset = asset!("/assets/style.css");
static ADDRESS_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
/// Represents a locally stored address with validation and activation state
///
/// Each address is assigned a unique ID for tracking and can be toggled active/inactive.
/// Valid addresses have matching entries in the parking restriction database.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredAddress {
    /// Unique stable identifier for this address
    pub id: usize,
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
    pub matched_entry: Option<StaticAddressEntry>,
}
impl StoredAddress {
    /// Create a new stored address and attempt to match against database
    ///
    /// # Arguments
    /// * `street` - Street name
    /// * `street_number` - Street number
    /// * `postal_code` - Postal code
    ///
    /// # Returns
    /// StoredAddress with unique ID, validation result, and matched data (if found)
    pub fn new(street: String, street_number: String, postal_code: String) -> Self {
        let fuzzy_match_result = fuzzy_match_address(&street, &street_number, &postal_code);
        let (valid, matched_entry) = match fuzzy_match_result {
            Some(entry) => (true, Some(entry)),
            None => (false, None),
        };
        let id = ADDRESS_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
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
/// Fuzzy match address against database
///
/// Implements multi-stage matching strategy:
/// 1. Exact match (case-sensitive)
/// 2. Case-insensitive substring matching
///
/// # Arguments
/// * `street` - Street name
/// * `street_number` - Street number
/// * `postal_code` - Postal code
///
/// # Returns
/// Some(StaticAddressEntry) if match found, None otherwise
///
/// # Examples
/// ```no_run
/// // Matches "Storgatan" even with lowercase input
/// let result = fuzzy_match_address("storgatan", "10", "22100");
/// assert!(result.is_some());
/// ```
fn fuzzy_match_address(
    street: &str,
    street_number: &str,
    postal_code: &str,
) -> Option<StaticAddressEntry> {
    match match_address(street, street_number, postal_code) {
        MatchResult::Valid(entry) => return Some(entry),
        MatchResult::Invalid => {}
    }
    let data = crate::matching::get_parking_data();
    let street_lower = street.to_lowercase().trim().to_string();
    let street_number_lower = street_number.to_lowercase().trim().to_string();
    let postal_code_trimmed = postal_code.trim();
    for entry in data.values() {
        let entry_street_lower = entry.gata.to_lowercase();
        let entry_number_lower = entry.gatunummer.to_lowercase();
        let street_match = entry_street_lower.contains(&street_lower)
            || street_lower.contains(&entry_street_lower);
        let number_match = entry_number_lower == street_number_lower;
        let postal_match = entry.postnummer == postal_code_trimmed;
        if street_match && number_match && postal_match {
            return Some(entry.clone());
        }
    }
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
    let mut stored_addresses = use_signal::<Vec<StoredAddress>>(Vec::new);
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
        if !addrs.iter().any(|a| {
            a.street == new_addr.street
                && a.street_number == new_addr.street_number
                && a.postal_code == new_addr.postal_code
        }) {
            info!("Adding new address, total now: {}", addrs.len() + 1);
            addrs.push(new_addr);
            if let Err(e) = write_addresses_to_device(&addrs) {
                error!("Failed to persist addresses after add: {}", e);
            }
        } else {
            warn!("Duplicate address detected, not adding");
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
