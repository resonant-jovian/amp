pub mod adresser;
pub mod paneler;
pub mod topbar;

use crate::countdown::bucket_for;
use crate::matching::match_address;
use crate::static_data::StaticAddressEntry;
use dioxus::prelude::*;
use std::collections::HashMap;

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
            active: true, // New addresses are active by default
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
    // Try exact match first
    match match_address(gata, gatunummer, postnummer) {
        crate::matching::MatchResult::Valid(entry) => return Some(entry),
        crate::matching::MatchResult::Invalid => {}
    }

    // TODO: Implement fuzzy matching
    // For now, return None if exact match fails
    None
}

/// Compute which paneler bucket an address belongs to based on time remaining
fn compute_bucket(stored: &StoredAddress) -> Option<paneler::PanelBucket> {
    let entry = stored.matched_entry.as_ref()?;
    let bucket = bucket_for(entry.dag, &entry.tid);

    let panel_bucket = match bucket {
        crate::countdown::TimeBucket::Now => paneler::PanelBucket::Active,
        crate::countdown::TimeBucket::Within6Hours => paneler::PanelBucket::Six,
        crate::countdown::TimeBucket::Within1Day => paneler::PanelBucket::Day,
        crate::countdown::TimeBucket::Within1Month => paneler::PanelBucket::Month,
        crate::countdown::TimeBucket::Invalid => paneler::PanelBucket::NotValid,
    };

    Some(panel_bucket)
}

use crate::ui::{
    adresser::Adresser,
    paneler::{Active, Day, Month, NotValid, Six},
    topbar::TopBar,
};

#[component]
pub fn App() -> Element {
    let stored_addresses = use_signal::<Vec<StoredAddress>>(Vec::new);
    let mut bucketed =
        use_signal::<HashMap<paneler::PanelBucket, Vec<StoredAddress>>>(HashMap::new);

    // Update buckets whenever addresses change
    use_effect(move || {
        let addrs = stored_addresses.read();
        let mut buckets: HashMap<paneler::PanelBucket, Vec<StoredAddress>> = HashMap::new();

        for addr in addrs.iter() {
            // Only bucket active, valid addresses
            if !addr.active || !addr.valid {
                continue;
            }

            if let Some(bucket) = compute_bucket(addr) {
                buckets.entry(bucket).or_default().push(addr.clone());
            }
        }

        bucketed.set(buckets);
    });

    // Handle adding a new address
    let handle_add_address: Callback<(String, String, String)> = {
        let mut addresses = stored_addresses;
        Callback::new(move |args: (String, String, String)| {
            let (gata, gatunummer, postnummer) = args;
            let new_addr = StoredAddress::new(gata, gatunummer, postnummer);

            // Check if already exists
            let mut addrs = addresses.write();
            if !addrs.iter().any(|a| {
                a.gata == new_addr.gata
                    && a.gatunummer == new_addr.gatunummer
                    && a.postnummer == new_addr.postnummer
            }) {
                addrs.push(new_addr);
            }

            // TODO: Write addresses to Android persistent storage
            // write_addresses_to_device(&addrs);
        })
    };

    // Handle toggling address active state
    let handle_toggle_active: Callback<usize> = {
        let mut addresses = stored_addresses;
        Callback::new(move |index: usize| {
            let mut addrs = addresses.write();
            if let Some(addr) = addrs.get_mut(index) {
                addr.active = !addr.active;
            }
            // TODO: Persist to Android storage
            // write_addresses_to_device(&addrs);
        })
    };

    // Handle removing an address
    let handle_remove_address: Callback<usize> = {
        let mut addresses = stored_addresses;
        Callback::new(move |index: usize| {
            let mut addrs = addresses.write();
            if index < addrs.len() {
                addrs.remove(index);
            }
            // TODO: Persist to Android storage
            // write_addresses_to_device(&addrs);
        })
    };

    // TODO: Read addresses from Android persistent storage on app start
    // use_effect(move || {
    //     let addrs = read_addresses_from_device();
    //     stored_addresses.set(addrs);
    // });

    // TODO: Implement GPS location reading
    // let handle_gps = move |_| {
    //     let location = read_device_gps_location();
    //     // Process location...
    // };

    // TODO: Implement Android notifications
    // let handle_send_notification = move |title: &str, body: &str| {
    //     send_android_notification(title, body);
    // };

    let bucketed_map = bucketed.read();

    rsx! {
        Stylesheet { href: CSS }
        div {
            class: "app-wrapper",
            TopBar {
                on_add_address: handle_add_address,
            },
            div {
                class: "app-container",
                Adresser {
                    stored_addresses: stored_addresses.read().clone(),
                    on_toggle_active: handle_toggle_active,
                    on_remove_address: handle_remove_address,
                }
                div {
                    class: "categories-section",
                    Active {
                        addresses: bucketed_map.get(&paneler::PanelBucket::Active).cloned().unwrap_or_default(),
                    }
                    Six {
                        addresses: bucketed_map.get(&paneler::PanelBucket::Six).cloned().unwrap_or_default(),
                    }
                    Day {
                        addresses: bucketed_map.get(&paneler::PanelBucket::Day).cloned().unwrap_or_default(),
                    }
                    Month {
                        addresses: bucketed_map.get(&paneler::PanelBucket::Month).cloned().unwrap_or_default(),
                    }
                    NotValid {
                        addresses: bucketed_map.get(&paneler::PanelBucket::NotValid).cloned().unwrap_or_default(),
                    }
                }
            }
        }
    }
}
