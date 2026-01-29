use dioxus::prelude::*;

#[component]
pub fn TopBar(mut on_add_address: EventHandler<(String, String, String)>) -> Element {
    let mut address_input = use_signal(String::new);
    let mut postnummer_input = use_signal(String::new);
    
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
    
    rsx! {
        div { class: "top-bar",
            div { class: "input-column",
                input {
                    id: "addressInput",
                    placeholder: "T.ex: Storgatan 10",
                    r#type: "text",
                    value: "{address_input}",
                    oninput: move |evt: FormEvent| {
                        address_input.set(evt.value());
                    },
                }
                input {
                    id: "postalInput",
                    placeholder: "Postnummer",
                    r#type: "text",
                    value: "{postnummer_input}",
                    oninput: move |evt: FormEvent| {
                        postnummer_input.set(evt.value());
                    },
                }
            }
            div { class: "button-row",
                button {
                    class: "btn",
                    id: "addBtn",
                    onclick: handle_add_click,
                    "Lägg till"
                }
                button {
                    class: "btn",
                    id: "gpsBtn",
                    onclick: handle_gps_click,
                    "GPS"
                }
            }
        }
    }
}
