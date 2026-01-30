pub mod addresses;
pub mod panels;
pub mod top_bar;
use crate::matching::match_address;
use crate::static_data::StaticAddressEntry;
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
/// Fuzzy match address against database with case-insensitive and whitespace-tolerant matching
///
/// # Arguments
/// * `street` - Street name
/// * `street_number` - Street number
/// * `postal_code` - Postal code
///
/// # Returns
/// Some(StaticAddressEntry) if match found, None otherwise
///
/// # TODO
/// Implement better fuzzy matching algorithm (Levenshtein distance or similar)
fn fuzzy_match_address(
    street: &str,
    street_number: &str,
    postal_code: &str,
) -> Option<StaticAddressEntry> {
    match match_address(street, street_number, postal_code) {
        crate::matching::MatchResult::Valid(entry) => return Some(entry),
        crate::matching::MatchResult::Invalid => {}
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
#[component]
pub fn App() -> Element {
    let mut stored_addresses = use_signal::<Vec<StoredAddress>>(Vec::new);
    {
        let mut addrs = stored_addresses.write();
        if addrs.is_empty() {
            addrs.push(StoredAddress::new(
                "Storgatan".to_string(),
                "1".to_string(),
                "22100".to_string(),
            ));
            addrs.push(StoredAddress::new(
                "Storgatan".to_string(),
                "10".to_string(),
                "22100".to_string(),
            ));
            addrs.push(StoredAddress::new(
                "Kyrkogården".to_string(),
                "1".to_string(),
                "22222".to_string(),
            ));
            addrs.push(StoredAddress::new(
                "Klostergatan".to_string(),
                "5".to_string(),
                "22100".to_string(),
            ));
            addrs.push(StoredAddress::new(
                "Fantasigatan".to_string(),
                "999".to_string(),
                "00000".to_string(),
            ));
        }
    }
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
