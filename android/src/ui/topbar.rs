use dioxus::prelude::*;
#[component]
pub fn TopBar(mut on_add_address: EventHandler<(String, String, String)>) -> Element {
    let mut address_input = use_signal(String::new);
    let mut postnummer_input = use_signal(String::new);
    let handle_add_click = move |_| {
        let address_str = address_input();
        let postnummer = postnummer_input();
        info!("Add button clicked: address='{}', postal='{}'", address_str, postnummer);
        if address_str.trim().is_empty() || postnummer.trim().is_empty() {
            warn!("Validation failed: empty fields");
            return;
        }
        let street_words: Vec<&str> = address_str.split_whitespace().collect();
        if street_words.len() < 2 {
            warn!("Address parsing failed: need at least 2 words");
            return;
        }
        let gatunummer = street_words[street_words.len() - 1].to_string();
        let gata = street_words[..street_words.len() - 1].join(" ");
        info!(
            "Parsed: gata='{}', gatunummer='{}', postnummer='{}'", gata, gatunummer,
            postnummer
        );
        on_add_address.call((gata, gatunummer, postnummer.to_string()));
        address_input.set(String::new());
        postnummer_input.set(String::new());
        info!("Address added successfully");
    };
    let handle_gps_click = move |_| {
        info!("GPS button clicked - TODO: implement location reading");
    };
    let handle_settings_click = move |_| {
        info!("Settings button clicked - TODO: implement settings");
    };
    let svg_bg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" preserveAspectRatio="xMidYMid slice" style="position:absolute;top:0;left:0;width:100%;height:100%;z-index:0">
        <defs>
            <radialGradient id="Gradient1" cx="50%" cy="50%" fx="0.441602%" fy="50%" r=".5">
                <animate attributeName="fx" dur="34s" values="0%;3%;0%" repeatCount="indefinite"/>
                <stop offset="0%" stop-color="rgba(255, 0, 255, 1)"/>
                <stop offset="100%" stop-color="rgba(255, 0, 255, 0)"/>
            </radialGradient>
            <radialGradient id="Gradient2" cx="50%" cy="50%" fx="2.68147%" fy="50%" r=".5">
                <animate attributeName="fx" dur="23.5s" values="0%;3%;0%" repeatCount="indefinite"/>
                <stop offset="0%" stop-color="rgba(255, 255, 0, 1)"/>
                <stop offset="100%" stop-color="rgba(255, 255, 0, 0)"/>
            </radialGradient>
            <radialGradient id="Gradient3" cx="50%" cy="50%" fx="0.836536%" fy="50%" r=".5">
                <animate attributeName="fx" dur="21.5s" values="0%;3%;0%" repeatCount="indefinite"/>
                <stop offset="0%" stop-color="rgba(0, 255, 255, 1)"/>
                <stop offset="100%" stop-color="rgba(0, 255, 255, 0)"/>
            </radialGradient>
        </defs>
        <rect x="13.744%" y="1.18473%" width="100%" height="100%" fill="url(#Gradient1)" transform="rotate(334.41 50 50)">
            <animate attributeName="x" dur="20s" values="25%;0%;25%" repeatCount="indefinite"/>
            <animate attributeName="y" dur="21s" values="0%;25%;0%" repeatCount="indefinite"/>
            <animateTransform attributeName="transform" type="rotate" from="0 50 50" to="360 50 50" dur="7s" repeatCount="indefinite"/>
        </rect>
        <rect x="-2.17916%" y="35.4267%" width="100%" height="100%" fill="url(#Gradient2)" transform="rotate(255.072 50 50)">
            <animate attributeName="x" dur="23s" values="-25%;0%;-25%" repeatCount="indefinite"/>
            <animate attributeName="y" dur="24s" values="0%;50%;0%" repeatCount="indefinite"/>
            <animateTransform attributeName="transform" type="rotate" from="0 50 50" to="360 50 50" dur="12s" repeatCount="indefinite"/>
        </rect>
        <rect x="9.00483%" y="14.5733%" width="100%" height="100%" fill="url(#Gradient3)" transform="rotate(139.903 50 50)">
            <animate attributeName="x" dur="25s" values="0%;25%;0%" repeatCount="indefinite"/>
            <animate attributeName="y" dur="12s" values="0%;25%;0%" repeatCount="indefinite"/>
            <animateTransform attributeName="transform" type="rotate" from="360 50 50" to="0 50 50" dur="9s" repeatCount="indefinite"/>
        </rect>
    </svg>"#;
    rsx! {
        div { class: "category-container topbar-container",
            div { class: "category-title topbar-title",
                div { class: "topbar-bg-wrap", dangerous_inner_html: svg_bg }
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
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "white",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            line {
                                x1: "3",
                                y1: "6",
                                x2: "21",
                                y2: "6",
                            }
                            line {
                                x1: "3",
                                y1: "12",
                                x2: "21",
                                y2: "12",
                            }
                            line {
                                x1: "3",
                                y1: "18",
                                x2: "21",
                                y2: "18",
                            }
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
