use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use dioxus_primitives::switch::SwitchThumb;
use dioxus_primitives::switch::Switch;

pub mod components;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Address {
    id: u64,
    street: String,
    postal: String,
    timestamp: u64,
    active: bool,
    expiry_time: u64, // milliseconds since epoch
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
enum Category {
    Active,
    Within6h,
    Within24h,
    WithinMonth,
    NotValid,
}

static CSS: Asset = asset!("../ui/assets/style.css");
static COMP: Asset = asset!("../ui/assets/dx-components-theme.css");

#[component]
pub fn App() -> Element {
    let mut addresses = use_signal(Vec::<Address>::new);
    let mut street_input = use_signal(String::new);
    let mut postal_input = use_signal(String::new);
    let mut pending_delete_index: Signal<Option<usize>> = use_signal(|| None);

    let add_address = move |_| {
        let street = street_input.read().trim().to_string();
        let postal = postal_input.read().trim().to_string();

        if street.is_empty() || postal.is_empty() {
            return;
        }

        let new_addr = Address {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            street,
            postal,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            active: true,
            expiry_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                + 24 * 60 * 60 * 1000,
        };

        addresses.push(new_addr);
        street_input.set(String::new());
        postal_input.set(String::new());
    };

    let add_gps = move |_| {
        let new_addr = Address {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            street: "GPS: 55.7047°N, 13.1910°E".to_string(),
            postal: "GPS".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            active: true,
            expiry_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                + 24 * 60 * 60 * 1000,
        };

        addresses.push(new_addr);
    };

    let mut remove_address = move |index: usize| {
        pending_delete_index.set(Some(index));
    };

    let mut toggle_address = move |index: usize| {
        if let Some(mut addr) = addresses.get_mut(index) {
            addr.active = !addr.active;
        }
    };

    let format_time_remaining = |expiry_time: u64| -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let remaining = if expiry_time > now {
            expiry_time - now
        } else {
            0
        };

        if remaining == 0 {
            return "00:00:00:00".to_string();
        }

        let days = remaining / (1000 * 60 * 60 * 24);
        let hours = (remaining % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60);
        let minutes = (remaining % (1000 * 60 * 60)) / (1000 * 60);
        let seconds = (remaining % (1000 * 60)) / 1000;

        format!(
            "{:02}:{:02}:{:02}:{:02}",
            days, hours, minutes, seconds
        )
    };

    let categorize_addresses = |addrs: &[Address]| -> HashMap<Category, Vec<(usize, Address)>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut categories: HashMap<Category, Vec<(usize, Address)>> = HashMap::new();
        categories.insert(Category::Active, Vec::new());
        categories.insert(Category::Within6h, Vec::new());
        categories.insert(Category::Within24h, Vec::new());
        categories.insert(Category::WithinMonth, Vec::new());
        categories.insert(Category::NotValid, Vec::new());

        let six_hours = 6 * 60 * 60 * 1000;
        let twenty_four_hours = 24 * 60 * 60 * 1000;
        let one_month = 30 * 24 * 60 * 60 * 1000;

        for (index, addr) in addrs.iter().enumerate() {
            if !addr.active {
                categories.get_mut(&Category::NotValid).unwrap().push((index, addr.clone()));
                continue;
            }

            let remaining = if addr.expiry_time > now {
                addr.expiry_time - now
            } else {
                0
            };

            let category = if remaining == 0 {
                Category::NotValid
            } else if remaining <= six_hours {
                Category::Within6h
            } else if remaining <= twenty_four_hours {
                Category::Within24h
            } else if remaining <= one_month {
                Category::WithinMonth
            } else {
                Category::Active
            };

            categories.get_mut(&category).unwrap().push((index, addr.clone()));
        }

        categories
    };

    let addrs_read = addresses.read();
    let categories = categorize_addresses(&addrs_read);
    drop(addrs_read); 

    let mut checked = use_signal(|| false);

    rsx! {
        Stylesheet { href: CSS }
        Stylesheet { href: COMP }
     
        div {
            class: "container",
            
            // Top Bar
            div {
                class: "top-bar",
                
                div {
                    class: "input-section",
                    
                    div {
                        class: "input-group",
                        input {
                            r#type: "text",
                            id: "streetInput",
                            placeholder: "Adress",
                            value: "{street_input}",
                            onchange: move |evt| street_input.set(evt.value()),
                        }
                        input {
                            r#type: "text",
                            id: "postalInput",
                            placeholder: "Postnummer",
                            value: "{postal_input}",
                            onchange: move |evt| postal_input.set(evt.value()),
                        }
                    }
                    
                        div {
                            class: "btn-group",
                            role: "group",
                            button {
                                class: "btn btn-add",
                                onclick: add_address,
                                "+ Adress"
                            }
                    
                            button {
                                class: "btn btn-gps",
                                onclick: add_gps,
                                "+ GPS"
                            }
                        }                    
                }
            }

            // Stored Addresses
            div {
                class: "stored-addresses",
                
                div { class: "stored-addresses-title", "Adresser" }
                
                div {
                    class: "address-list",
                    id: "addressList",
                    
                    if addresses.read().is_empty() {
                        div { class: "empty-message", "No addresses stored yet" }
                    } else {
                        for (index, addr) in addresses.read().iter().enumerate() {
                            div {
                                class: "address-item",
                                
                                div {
                                    class: "address-text",
                                    "{addr.street}, {addr.postal}"
                                }
                                
                                div {
                                    class: "address-actions",
                                    
                                    div { class: "switch-example",
                                        Switch {
                                                checked: checked(),
                                                aria_label: "Switch",
                                                on_checked_change: move |new_checked| {
                                                checked.set(new_checked);
                                            },
                                            SwitchThumb {}
                                        }
                                    }
                                    
                                    button {
                                        class: "btn btn-remove",
                                        onclick: move |_| remove_address(index),
                                        "X"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Categories Section
            div {
                class: "categories-section",
                
                // Active
                div {
                    class: "category-container category-active",
                    
                    div { class: "category-title", "Städning pågår" }
                    
                    div {
                        class: "category-content",
                        id: "categoryActive",
                        
                        if categories.get(&Category::Active).map(|v| v.is_empty()).unwrap_or(true) {
                            div { class: "empty-message", "No active addresses" }
                        } else {
                            for (_, addr) in &categories[&Category::Active] {
                                div {
                                    class: "address-row",
                                    
                                    div {
                                        class: "address-row-left",
                                        "{addr.street}, {addr.postal}"
                                    }
                                    
                                    div {
                                        class: "address-row-right",
                                        "{format_time_remaining(addr.expiry_time)}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Within 6h
                div {
                    class: "category-container category-6h",
                    
                    div { class: "category-title", "Inom 6h" }
                    
                    div {
                        class: "category-content",
                        id: "category6h",
                        
                        if categories.get(&Category::Within6h).map(|v| v.is_empty()).unwrap_or(true) {
                            div { class: "empty-message", "No addresses expiring within 6 hours" }
                        } else {
                            for (_, addr) in &categories[&Category::Within6h] {
                                div {
                                    class: "address-row",
                                    
                                    div {
                                        class: "address-row-left",
                                        "{addr.street}, {addr.postal}"
                                    }
                                    
                                    div {
                                        class: "address-row-right",
                                        "{format_time_remaining(addr.expiry_time)}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Within 24h
                div {
                    class: "category-container category-24h",
                    
                    div { class: "category-title", "Inom 24h" }
                    
                    div {
                        class: "category-content",
                        id: "category24h",
                        
                        if categories.get(&Category::Within24h).map(|v| v.is_empty()).unwrap_or(true) {
                            div { class: "empty-message", "No addresses expiring within 24 hours" }
                        } else {
                            for (_, addr) in &categories[&Category::Within24h] {
                                div {
                                    class: "address-row",
                                    
                                    div {
                                        class: "address-row-left",
                                        "{addr.street}, {addr.postal}"
                                    }
                                    
                                    div {
                                        class: "address-row-right",
                                        "{format_time_remaining(addr.expiry_time)}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Within a month
                div {
                    class: "category-container category-month",
                    
                    div { class: "category-title", "Inom en månad" }
                    
                    div {
                        class: "category-content",
                        id: "categoryMonth",
                        
                        if categories.get(&Category::WithinMonth).map(|v| v.is_empty()).unwrap_or(true) {
                            div { class: "empty-message", "No addresses expiring within a month" }
                        } else {
                            for (_, addr) in &categories[&Category::WithinMonth] {
                                div {
                                    class: "address-row",
                                    
                                    div {
                                        class: "address-row-left",
                                        "{addr.street}, {addr.postal}"
                                    }
                                    
                                    div {
                                        class: "address-row-right",
                                        "{format_time_remaining(addr.expiry_time)}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Not valid
                div {
                    class: "category-container category-invalid",
                    
                    div { class: "category-title", "Ingen städning" }
                    
                    div {
                        class: "category-content",
                        id: "categoryInvalid",
                        
                        if categories.get(&Category::NotValid).map(|v| v.is_empty()).unwrap_or(true) {
                            div { class: "empty-message", "No invalid addresses" }
                        } else {
                            for (_, addr) in &categories[&Category::NotValid] {
                                div {
                                    class: "address-row",
                                    
                                    div {
                                        class: "address-row-left",
                                        "{addr.street}, {addr.postal}"
                                    }
                                    
                                    div {
                                        class: "address-row-right",
                                        "{format_time_remaining(addr.expiry_time)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}