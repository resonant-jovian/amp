pub mod adresser;
pub mod paneler;
pub mod topbar;

use crate::static_data::StaticAddressEntry;
use crate::ui::{
    adresser::Adresser,
    paneler::{Active, Day, Month, NotValid, Six},
    topbar::TopBar,
};

use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/style.css");

#[component]
pub fn App() -> Element {
    // Vector of validated addresses from static correlations
    let mut addresses = use_signal::<Vec<StaticAddressEntry>>(Vec::new);

    // Handle adding a valid address
    let handle_add_address = move |entry: StaticAddressEntry| {
        let mut addrs = addresses.write();
        // Check if address already exists to avoid duplicates
        if !addrs.iter().any(|a| a.adress == entry.adress) {
            addrs.push(entry);
        }
    };

    // Handle removing an address by its full address string
    let handle_remove_address = move |adress: String| {
        let mut addrs = addresses.write();
        addrs.retain(|a| a.adress != adress);
    };

    // Initialize with sample data for testing
    use_effect(move || {
        if !addresses.read().is_empty() {
        }
        // TODO: Initialize with sample data or from persistent storage
    });

    let addrs = addresses.read().clone();

    rsx! {
        Stylesheet { href: CSS }
        div {
            class: "app-wrapper",
            TopBar {  },
            div {
                class: "app-container",
                Adresser {
                    on_add_valid_address: handle_add_address,
                }
                div {
                    class: "categories-section",
                    Active {
                        addresses: addrs.clone(),
                        on_remove_address: handle_remove_address,
                    }
                    Six {
                        addresses: addrs.clone(),
                        on_remove_address: handle_remove_address,
                    }
                    Day {
                        addresses: addrs.clone(),
                        on_remove_address: handle_remove_address,
                    }
                    Month {
                        addresses: addrs.clone(),
                        on_remove_address: handle_remove_address,
                    }
                    NotValid {
                        addresses: addrs.clone(),
                        on_remove_address: handle_remove_address,
                    }
                }
            }
        }
    }
}
