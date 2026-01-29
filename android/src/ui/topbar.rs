use dioxus::prelude::*;

#[component]
pub fn TopBar(mut on_add_address: EventHandler<(String, String, String)>) -> Element {
    let mut address_input = use_signal(String::new());
    let mut postnummer_input = use_signal(String::new());
    
    let handle_add_click = move |_| {
        let address_str = address_input();
        let postnummer = postnummer_input();
        
        tracing::info!("Add button clicked: address='{}', postal='{}'", address_str, postnummer);
        
        if address_str.trim().is_empty() || postnummer.trim().is_empty() {
            tracing::warn!("Validation failed: empty fields");
            return;
        }
        
        let street_words: Vec<&str> = address_str.trim().split_whitespace().collect();
        if street_words.len() < 2 {
            tracing::warn!("Address parsing failed: need at least 2 words");
            return;
        }
        
        let gatunummer = street_words[street_words.len() - 1].to_string();
        let gata = street_words[..street_words.len() - 1].join(" ");
        
        tracing::info!("Parsed: gata='{}', gatunummer='{}', postnummer='{}'", gata, gatunummer, postnummer);
        
        on_add_address.call((gata, gatunummer, postnummer.to_string()));
        
        address_input.set(String::new());
        postnummer_input.set(String::new());
        
        tracing::info!("Address added successfully");
    };
    
    let handle_gps_click = move |_| {
        tracing::info!("GPS button clicked - TODO: implement location reading");
    };
    
    let handle_settings_click = move |_| {
        tracing::info!("Settings button clicked - TODO: implement settings");
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
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "24",
                            height: "24",
                            viewBox: "0 0 24 24",
                            fill: "none",
                            stroke: "white",
                            "stroke-width": "2",
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            line { x1: "3", y1: "6", x2: "21", y2: "6" }
                            line { x1: "3", y1: "12", x2: "21", y2: "12" }
                            line { x1: "3", y1: "18", x2: "21", y2: "18" }
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
                            value: "{postnummer_input}",
                            oninput: move |evt: FormEvent| {
                                postnummer_input.set(evt.value());
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
