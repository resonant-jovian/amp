pub mod adresser;
pub mod paneler;
pub mod topbar;
use crate::matching::match_address;
use crate::static_data::StaticAddressEntry;
use dioxus::prelude::*;
static CSS: Asset = asset!("/assets/style.css");
/// Represents a locally stored address with validation and activation state
#[derive(Clone, Debug, PartialEq)]
pub struct StoredAddress {
    /// Street name (e.g., "Storgatan")
    pub gata: String,
    /// Street number (e.g., "10")
    pub gatunummer: String,
    /// Postal code (e.g., "22100")
    pub postnummer: String,
    /// Whether this address matches the database
    pub valid: bool,
    /// Whether this address should be displayed in panels
    pub active: bool,
    /// The matched database entry (if valid)
    pub matched_entry: Option<StaticAddressEntry>,
}
impl StoredAddress {
    /// Create a new stored address and attempt to match against database
    pub fn new(gata: String, gatunummer: String, postnummer: String) -> Self {
        let fuzzy_match_result = fuzzy_match_address(&gata, &gatunummer, &postnummer);
        let (valid, matched_entry) = match fuzzy_match_result {
            Some(entry) => (true, Some(entry)),
            None => (false, None),
        };
        StoredAddress {
            gata,
            gatunummer,
            postnummer,
            valid,
            active: true,
            matched_entry,
        }
    }
}
/// Fuzzy match address against database with case-insensitive and whitespace-tolerant matching
///
/// # TODO
/// Better fuzzy matching algorithm (Levenshtein distance or similar)
fn fuzzy_match_address(
    gata: &str,
    gatunummer: &str,
    postnummer: &str,
) -> Option<StaticAddressEntry> {
    match match_address(gata, gatunummer, postnummer) {
        crate::matching::MatchResult::Valid(entry) => return Some(entry),
        crate::matching::MatchResult::Invalid => {}
    }
    None
}
use crate::ui::{
    adresser::Adresser, paneler::{Active, Day, Month, NotValid, Six},
    topbar::TopBar,
};
#[component]
pub fn App() -> Element {
    let mut stored_addresses = use_signal::<Vec<StoredAddress>>(Vec::new);
    let handle_add_address = move |args: (String, String, String)| {
        let (gata, gatunummer, postnummer) = args;
        let new_addr = StoredAddress::new(gata, gatunummer, postnummer);
        let mut addrs = stored_addresses.write();
        if !addrs
            .iter()
            .any(|a| {
                a.gata == new_addr.gata && a.gatunummer == new_addr.gatunummer
                    && a.postnummer == new_addr.postnummer
            })
        {
            addrs.push(new_addr);
        }
    };
    let handle_toggle_active = move |index: usize| {
        let mut addrs = stored_addresses.write();
        if let Some(addr) = addrs.get_mut(index) {
            addr.active = !addr.active;
        }
    };
    let handle_remove_address = move |index: usize| {
        let mut addrs = stored_addresses.write();
        if index < addrs.len() {
            addrs.remove(index);
        }
    };
    let addresses = stored_addresses.read().clone();
    /*
    let addresses: Vec<StoredAddress> = vec![
        StoredAddress {
            gata: "Testgatan".to_string(),
            gatunummer: "1A".to_string(),
            postnummer: "123 45".to_string(),
            valid: false,
            active: true,
            matched_entry: None,
        },
    ];


     */
    rsx! {
        Stylesheet { href: CSS }
        TopBar { on_add_address: handle_add_address }
        Adresser {
            stored_addresses: addresses.clone(),
            on_toggle_active: handle_toggle_active,
            on_remove_address: handle_remove_address,
        }
        div { class: "categories-section",
            Active { addresses: addresses.clone() }
            Six { addresses: addresses.clone() }
            Day { addresses: addresses.clone() }
            Month { addresses: addresses.clone() }
            NotValid { addresses: addresses.clone() }
        }
        script {}
    }
}
