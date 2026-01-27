use dioxus::prelude::*;
use crate::countdown::{bucket_for, format_countdown, TimeBucket};
use crate::static_data::StaticAddressEntry;

/// Display an address with countdown timer in appropriate category
#[component]
pub fn CategorizedAddress(
    entry: StaticAddressEntry,
    on_remove: EventHandler<()>,
) -> Element {
    let bucket = bucket_for(entry.dag, &entry.tid);
    let countdown = format_countdown(entry.dag, &entry.tid).unwrap_or_else(|| "...".to_string());

    let (category_name, css_class) = match bucket {
        TimeBucket::Now => ("Nu", "bucket-now"),
        TimeBucket::Within6Hours => ("Om mindre än 6h", "bucket-6h"),
        TimeBucket::Within1Day => ("Inom 24h", "bucket-1d"),
        TimeBucket::Within1Month => ("Inom 1 månad", "bucket-1m"),
        TimeBucket::Invalid => ("Ingen städning här", "bucket-invalid"),
    };

    rsx! {
        div { class: "category-item {css_class}",
            div { class: "address-text",
                "{entry.adress}"
            }
            div { class: "countdown-text",
                "{countdown}"
            }
            button {
                class: "remove-button",
                onclick: move |_| on_remove.call(()),
                "x"
            }
        }
    }
}

/// Panel displaying addresses needing attention within 4 hours
#[component]
pub fn Active(
    addresses: Vec<StaticAddressEntry>,
    on_remove_address: EventHandler<String>,
) -> Element {
    let active_addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| matches!(bucket_for(a.dag, &a.tid), TimeBucket::Now))
        .collect();

    if active_addrs.is_empty() {
        return rsx! {
            div { class: "panel panel-active",
                h3 { "Nu" }
                p { "Inga adresser kräver uppmärksamhet just nu" }
            }
        };
    }

    rsx! {
        div { class: "panel panel-active",
            h3 { "Nu" }
            div { class: "address-list",
                {active_addrs.into_iter().map(|addr| {
                    let key = addr.adress.clone();
                    rsx! {
                        CategorizedAddress {
                            entry: addr,
                            on_remove: move |_| on_remove_address.call(key.clone()),
                        }
                    }
                })}
            }
        }
    }
}

/// Panel displaying addresses within 6 hours
#[component]
pub fn Six(
    addresses: Vec<StaticAddressEntry>,
    on_remove_address: EventHandler<String>,
) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| matches!(bucket_for(a.dag, &a.tid), TimeBucket::Within6Hours))
        .collect();

    if addrs.is_empty() {
        return rsx! {
            div { class: "panel panel-6h",
                h3 { "Om mindre än 6h" }
                p { "Inga adresser" }
            }
        };
    }

    rsx! {
        div { class: "panel panel-6h",
            h3 { "Om mindre än 6h" }
            div { class: "address-list",
                {addrs.into_iter().map(|addr| {
                    let key = addr.adress.clone();
                    rsx! {
                        CategorizedAddress {
                            entry: addr,
                            on_remove: move |_| on_remove_address.call(key.clone()),
                        }
                    }
                })}
            }
        }
    }
}

/// Panel displaying addresses within 24 hours
#[component]
pub fn Day(
    addresses: Vec<StaticAddressEntry>,
    on_remove_address: EventHandler<String>,
) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| matches!(bucket_for(a.dag, &a.tid), TimeBucket::Within1Day))
        .collect();

    if addrs.is_empty() {
        return rsx! {
            div { class: "panel panel-1d",
                h3 { "Inom 24h" }
                p { "Inga adresser" }
            }
        };
    }

    rsx! {
        div { class: "panel panel-1d",
            h3 { "Inom 24h" }
            div { class: "address-list",
                {addrs.into_iter().map(|addr| {
                    let key = addr.adress.clone();
                    rsx! {
                        CategorizedAddress {
                            entry: addr,
                            on_remove: move |_| on_remove_address.call(key.clone()),
                        }
                    }
                })}
            }
        }
    }
}

/// Panel displaying addresses within 1 month
#[component]
pub fn Month(
    addresses: Vec<StaticAddressEntry>,
    on_remove_address: EventHandler<String>,
) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| matches!(bucket_for(a.dag, &a.tid), TimeBucket::Within1Month))
        .collect();

    if addrs.is_empty() {
        return rsx! {
            div { class: "panel panel-1m",
                h3 { "Inom 1 månad" }
                p { "Inga adresser" }
            }
        };
    }

    rsx! {
        div { class: "panel panel-1m",
            h3 { "Inom 1 månad" }
            div { class: "address-list",
                {addrs.into_iter().map(|addr| {
                    let key = addr.adress.clone();
                    rsx! {
                        CategorizedAddress {
                            entry: addr,
                            on_remove: move |_| on_remove_address.call(key.clone()),
                        }
                    }
                })}
            }
        }
    }
}

/// Panel displaying addresses with no valid parking restriction data
#[component]
pub fn NotValid(
    addresses: Vec<StaticAddressEntry>,
    on_remove_address: EventHandler<String>,
) -> Element {
    let addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| matches!(bucket_for(a.dag, &a.tid), TimeBucket::Invalid))
        .collect();

    if addrs.is_empty() {
        return rsx! {
            div { class: "panel panel-invalid",
                h3 { "Ingen städning här" }
                p { "Inga adresser" }
            }
        };
    }

    rsx! {
        div { class: "panel panel-invalid",
            h3 { "Ingen städning här" }
            div { class: "address-list",
                {addrs.into_iter().map(|addr| {
                    let key = addr.adress.clone();
                    rsx! {
                        CategorizedAddress {
                            entry: addr,
                            on_remove: move |_| on_remove_address.call(key.clone()),
                        }
                    }
                })}
            }
        }
    }
}
