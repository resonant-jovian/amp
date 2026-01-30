use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaBlurb;
use dioxus_free_icons::Icon;

/// Top navigation bar with address input and controls
///
/// Provides input fields for adding new addresses and buttons for GPS and settings.
///
/// # Props
/// * `on_add_address` - Event handler called with (street, street_number, postal_code) tuple
#[component]
pub fn TopBar(mut on_add_address: EventHandler<(String, String, String)>) -> Element {
    let mut address_input = use_signal(String::new);
    let mut postal_code_input = use_signal(String::new);
    let handle_add_click = move |_| {
        let address_str = address_input();
        let postal_code = postal_code_input();
        info!(
            "Add button clicked: address='{}', postal_code='{}'",
            address_str, postal_code
        );
        if address_str.trim().is_empty() || postal_code.trim().is_empty() {
            warn!("Validation failed: empty fields");
            return;
        }
        let street_words: Vec<&str> = address_str.split_whitespace().collect();
        if street_words.len() < 2 {
            warn!("Address parsing failed: need at least 2 words");
            return;
        }
        let street_number = street_words[street_words.len() - 1].to_string();
        let street = street_words[..street_words.len() - 1].join(" ");
        info!(
            "Parsed: street='{}', street_number='{}', postal_code='{}'",
            street, street_number, postal_code
        );
        on_add_address.call((street, street_number, postal_code.to_string()));
        address_input.set(String::new());
        postal_code_input.set(String::new());
        info!("Address added successfully");
    };
    let handle_gps_click = move |_| {
        info!("GPS button clicked - TODO: implement location reading");
    };
    let handle_settings_click = move |_| {
        info!("Settings button clicked - TODO: implement settings");
    };
    rsx! {
        div { class: "category-container topbar-container",
            div { class: "category-title topbar-title",
                div { class: "topbar-title-content",
                    span { class: "topbar-title-text", "amp" }
                    button {
                        class: "topbar-settings-btn",
                        onclick: handle_settings_click,
                        title: "Inställningar",
                        Icon {
                            icon: FaBlurb,
                            width: 20,
                            height: 20,
                        }
                    }
                }
            }
            div { class: "category-content topbar-content",
                div { class: "topbar-inputs-row",
                    div { class: "address-item topbar-input-item",
                        input {
                            id: "addressInput",
                            placeholder: "T.ex: Storgatan 10",
                            r#type: "text",
                            class: "topbar-input",
                            value: "{address_input}",
                            oninput: move |evt: FormEvent| {
                                address_input.set(evt.value());
                            },
                        }
                    }
                    div { class: "address-item topbar-input-item",
                        input {
                            id: "postalInput",
                            placeholder: "Postnummer",
                            r#type: "text",
                            class: "topbar-input",
                            value: "{postal_code_input}",
                            oninput: move |evt: FormEvent| {
                                postal_code_input.set(evt.value());
                            },
                        }
                    }
                }
                div { class: "topbar-buttons-row",
                    button {
                        class: "topbar-btn",
                        id: "addBtn",
                        onclick: handle_add_click,
                        "Lägg till"
                    }
                    button {
                        class: "topbar-btn",
                        id: "gpsBtn",
                        onclick: handle_gps_click,
                        "GPS"
                    }
                }
            }
        }
    }
}
