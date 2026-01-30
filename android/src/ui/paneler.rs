use crate::countdown::{TimeBucket, bucket_for, format_countdown};
use crate::ui::StoredAddress;
use dioxus::prelude::*;

/// Display an address with countdown timer in appropriate category
/// 
/// # Props
/// * `addr` - StoredAddress to display
/// * `index` - Position in list (for keying)
/// * `on_remove` - Event handler for remove button (currently unused)
#[component]
fn AddressItem(
    addr: StoredAddress,
    index: usize,
    on_remove: EventHandler<usize>,
) -> Element {
    let matched = &addr.matched_entry;
    let countdown = matched
        .as_ref()
        .and_then(|e| format_countdown(e.day, &e.time))
        .unwrap_or_else(|| "...".to_string());
    let address_display = format!(
        "{} {}, {}",
        addr.street,
        addr.street_number,
        addr.postal_code,
    );
    rsx! {
        div { class: "address-item",
            div { class: "address-text", "{address_display}" }
            div { class: "countdown-text", "{countdown}" }
        }
    }
}

/// Panel displaying addresses needing attention within 4 hours
/// 
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (will be filtered)
#[component]
pub fn Active(addresses: Vec<StoredAddress>) -> Element {
    let active_addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry.day, &entry.time), TimeBucket::Now)
            } else {
                false
            }
        })
        .collect();
    let active_count = active_addrs.len();
    rsx! {
        div { class: "category-container category-active",
            div { class: "category-title", "Städas nu" }
            div { class: "category-content", id: "categoryActive",
                if active_count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            active_addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}

/// Panel displaying addresses within 6 hours
/// 
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (will be filtered)
#[component]
pub fn Six(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry.day, &entry.time), TimeBucket::Within6Hours)
            } else {
                false
            }
        })
        .collect();
    let count = addrs.len();
    rsx! {
        div { class: "category-container category-6h",
            div { class: "category-title", "Inom 6 timmar" }
            div { class: "category-content", id: "category6h",
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}

/// Panel displaying addresses within 24 hours
/// 
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (will be filtered)
#[component]
pub fn Day(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry.day, &entry.time), TimeBucket::Within1Day)
            } else {
                false
            }
        })
        .collect();
    let count = addrs.len();
    rsx! {
        div { class: "category-container category-24h",
            div { class: "category-title", "Inom 1 dag" }
            div { class: "category-content", id: "category24h",
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}

/// Panel displaying addresses within 1 month
/// 
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (will be filtered)
#[component]
pub fn Month(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry.day, &entry.time), TimeBucket::Within1Month)
            } else {
                false
            }
        })
        .collect();
    let count = addrs.len();
    rsx! {
        div { class: "category-container category-month",
            div { class: "category-title", "Inom 1 månad" }
            div { class: "category-content", id: "categoryMonth",
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}

/// Panel displaying addresses with no valid parking restriction data
/// 
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (will be filtered)
#[component]
pub fn NotValid(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses.into_iter().filter(|a| a.active && !a.valid).collect();
    let count = addrs.len();
    rsx! {
        div { class: "category-container category-invalid",
            div { class: "category-title", "Ingen städning" }
            div { class: "category-content", id: "categoryInvalid",
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
