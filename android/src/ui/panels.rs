use crate::components::countdown::{TimeBucket, bucket_for, format_countdown};
use crate::ui::StoredAddress;
use dioxus::prelude::*;
use tokio::time::Duration;
/// Display an address with countdown timer in appropriate category
///
/// # Props
/// * `addr` - StoredAddress to display
/// * `index` - Position in list (for keying)
/// * `on_remove` - Event handler for remove button (currently unused)
#[component]
fn AddressItem(addr: StoredAddress, index: usize, on_remove: EventHandler<usize>) -> Element {
    let mut countdown = use_signal(|| "...".to_string());
    let addr_clone = addr.clone();
    use_future(move || {
        let addr_for_future = addr_clone.clone();
        async move {
            let bucket = addr_for_future
                .matched_entry
                .as_ref()
                .map(bucket_for)
                .unwrap_or(TimeBucket::Invalid);
            if let Some(matched) = &addr_for_future.matched_entry {
                countdown.set(format_countdown(matched).unwrap_or_else(|| "...".to_string()));
            }
            if bucket == TimeBucket::Invalid {
                return;
            }
            let update_interval = match bucket {
                TimeBucket::Now | TimeBucket::Within6Hours | TimeBucket::Within1Day => {
                    Duration::from_secs(1)
                }
                _ => Duration::from_secs(60),
            };
            loop {
                tokio::time::sleep(update_interval).await;
                let new_countdown = addr_for_future
                    .matched_entry
                    .as_ref()
                    .and_then(format_countdown)
                    .unwrap_or_else(|| "...".to_string());
                countdown.set(new_countdown);
            }
        }
    });
    let address_display = format!(
        "{} {}, {}",
        addr.street, addr.street_number, addr.postal_code,
    );
    rsx! {
        div { class: "address-item",
            div { class: "address-text", "{address_display}" }
            div { class: "countdown-text", "{countdown()}" }
        }
    }
}
/// Panel displaying addresses needing attention within 4 hours
///
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (will be filtered)
#[component]
pub fn ActivePanel(addresses: Vec<StoredAddress>) -> Element {
    let active_addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::Now)
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
pub fn SixHoursPanel(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::Within6Hours)
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
pub fn OneDayPanel(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::Within1Day)
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
pub fn OneMonthPanel(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::Within1Month)
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
pub fn InvalidPanel(addresses: Vec<StoredAddress>) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.active && !a.valid)
        .collect();
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
