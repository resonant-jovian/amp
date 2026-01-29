use crate::countdown::{bucket_for, format_countdown, TimeBucket};
use crate::ui::StoredAddress;
use dioxus::prelude::*;

/// Represents the five panel buckets for categorizing addresses
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanelBucket {
    Active,
    Six,
    Day,
    Month,
    NotValid,
}

/// Display an address with countdown timer in appropriate category
#[component]
fn AddressItem(addr: StoredAddress, index: usize, on_remove: EventHandler<usize>) -> Element {
    let matched = &addr.matched_entry;
    let countdown = matched
        .as_ref()
        .and_then(|e| format_countdown(e.dag, &e.tid))
        .unwrap_or_else(|| "...".to_string());

    let address_display = format!("{} {}, {}", addr.gata, addr.gatunummer, addr.postnummer);

    rsx! {
        div { class: "address-item",
            div { class: "address-info",
                div { class: "address-text", "{address_display}" }
                div { class: "countdown-text", "{countdown}" }
            }
            button {
                class: "remove-button",
                onclick: move |_| on_remove.call(index),
                "×"
            }
        }
    }
}

/// Panel displaying addresses needing attention within 4 hours
#[component]
pub fn Active(addresses: Vec<StoredAddress>) -> Element {
    let active_addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry.dag, &entry.tid), TimeBucket::Now)
            } else {
                false
            }
        })
        .collect();

    let active_count = active_addrs.len();

    rsx! {
        div { class: "category-container category-active",
            div { class: "category-title", "Städas nu" }
            div { class: "category-content",
                if active_count == 0 {
                    rsx! {
                        div { class: "empty-state",
                            "Inga adresser kräver omedelbar uppmärksamhet"
                        }
                    }
                } else {
                    rsx! {
                        div { class: "address-list",
                            {active_addrs.into_iter().enumerate().map(|(i, addr)| {
                                rsx! {
                                    AddressItem {
                                        addr: addr.clone(),
                                        index: i,
                                        on_remove: move |_| { /* TODO: wire to parent */ },
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}

/// Panel displaying addresses within 6 hours
#[component]
pub fn Six(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry.dag, &entry.tid), TimeBucket::Within6Hours)
            } else {
                false
            }
        })
        .collect();

    let count = addrs.len();

    rsx! {
        div { class: "category-container category-6h",
            div { class: "category-title", "Inom 6 timmar" }
            div { class: "category-content",
                if count == 0 {
                    rsx! {
                        div { class: "empty-state",
                            "Inga adresser"
                        }
                    }
                } else {
                    rsx! {
                        div { class: "address-list",
                            {addrs.into_iter().enumerate().map(|(i, addr)| {
                                rsx! {
                                    AddressItem {
                                        addr: addr.clone(),
                                        index: i,
                                        on_remove: move |_| { /* TODO: wire to parent */ },
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}

/// Panel displaying addresses within 24 hours
#[component]
pub fn Day(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry.dag, &entry.tid), TimeBucket::Within1Day)
            } else {
                false
            }
        })
        .collect();

    let count = addrs.len();

    rsx! {
        div { class: "category-container category-24h",
            div { class: "category-title", "Inom 1 dag" }
            div { class: "category-content",
                if count == 0 {
                    rsx! {
                        div { class: "empty-state",
                            "Inga adresser"
                        }
                    }
                } else {
                    rsx! {
                        div { class: "address-list",
                            {addrs.into_iter().enumerate().map(|(i, addr)| {
                                rsx! {
                                    AddressItem {
                                        addr: addr.clone(),
                                        index: i,
                                        on_remove: move |_| { /* TODO: wire to parent */ },
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}

/// Panel displaying addresses within 1 month
#[component]
pub fn Month(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry.dag, &entry.tid), TimeBucket::Within1Month)
            } else {
                false
            }
        })
        .collect();

    let count = addrs.len();

    rsx! {
        div { class: "category-container category-month",
            div { class: "category-title", "Inom 1 månad" }
            div { class: "category-content",
                if count == 0 {
                    rsx! {
                        div { class: "empty-state",
                            "Inga adresser"
                        }
                    }
                } else {
                    rsx! {
                        div { class: "address-list",
                            {addrs.into_iter().enumerate().map(|(i, addr)| {
                                rsx! {
                                    AddressItem {
                                        addr: addr.clone(),
                                        index: i,
                                        on_remove: move |_| { /* TODO: wire to parent */ },
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}

/// Panel displaying addresses with no valid parking restriction data
#[component]
pub fn NotValid(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| !a.valid || !a.active)
        .collect();

    let count = addrs.len();

    rsx! {
        div { class: "category-container category-invalid",
            div { class: "category-title", "Ingen städning" }
            div { class: "category-content",
                if count == 0 {
                    rsx! {
                        div { class: "empty-state",
                            "Inga adresser"
                        }
                    }
                } else {
                    rsx! {
                        div { class: "address-list",
                            {addrs.into_iter().enumerate().map(|(i, addr)| {
                                rsx! {
                                    AddressItem {
                                        addr: addr.clone(),
                                        index: i,
                                        on_remove: move |_| { /* TODO: wire to parent */ },
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}
