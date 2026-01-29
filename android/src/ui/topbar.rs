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
    rsx! {
        div { class: "category-container topbar-container",
            div { class: "category-title topbar-title",
                div { class: "topbar-bg-wrap",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        attr:"view-box": "0 0 100 100",
                        attr:"preserveAspectRatio": "xMidYMid slice",
                        defs {
                            "radial-gradient" {
                                attr:id: "Gradient1",
                                attr:cx: "50%",
                                attr:cy: "50%",
                                attr:fx: "0.441602%",
                                attr:fy: "50%",
                                attr:r: ".5",
                                animate {
                                    attr:"attributeName": "fx",
                                    attr:dur: "34s",
                                    attr:values: "0%;3%;0%",
                                    attr:"repeatCount": "indefinite",
                                }
                                stop {
                                    attr:offset: "0%",
                                    attr:"stop-color": "rgba(255, 0, 255, 1)",
                                }
                                stop {
                                    attr:offset: "100%",
                                    attr:"stop-color": "rgba(255, 0, 255, 0)",
                                }
                            }
                            "radial-gradient" {
                                attr:id: "Gradient2",
                                attr:cx: "50%",
                                attr:cy: "50%",
                                attr:fx: "2.68147%",
                                attr:fy: "50%",
                                attr:r: ".5",
                                animate {
                                    attr:"attributeName": "fx",
                                    attr:dur: "23.5s",
                                    attr:values: "0%;3%;0%",
                                    attr:"repeatCount": "indefinite",
                                }
                                stop {
                                    attr:offset: "0%",
                                    attr:"stop-color": "rgba(255, 255, 0, 1)",
                                }
                                stop {
                                    attr:offset: "100%",
                                    attr:"stop-color": "rgba(255, 255, 0, 0)",
                                }
                            }
                            "radial-gradient" {
                                attr:id: "Gradient3",
                                attr:cx: "50%",
                                attr:cy: "50%",
                                attr:fx: "0.836536%",
                                attr:fy: "50%",
                                attr:r: ".5",
                                animate {
                                    attr:"attributeName": "fx",
                                    attr:dur: "21.5s",
                                    attr:values: "0%;3%;0%",
                                    attr:"repeatCount": "indefinite",
                                }
                                stop {
                                    attr:offset: "0%",
                                    attr:"stop-color": "rgba(0, 255, 255, 1)",
                                }
                                stop {
                                    attr:offset: "100%",
                                    attr:"stop-color": "rgba(0, 255, 255, 0)",
                                }
                            }
                        }
                        rect {
                            attr:x: "13.744%",
                            attr:y: "1.18473%",
                            attr:width: "100%",
                            attr:height: "100%",
                            attr:fill: "url(#Gradient1)",
                            attr:transform: "rotate(334.41 50 50)",
                            animate {
                                attr:"attributeName": "x",
                                attr:dur: "20s",
                                attr:values: "25%;0%;25%",
                                attr:"repeatCount": "indefinite",
                            }
                            animate {
                                attr:"attributeName": "y",
                                attr:dur: "21s",
                                attr:values: "0%;25%;0%",
                                attr:"repeatCount": "indefinite",
                            }
                            "animate-transform" {
                                attr:"attributeName": "transform",
                                attr:type: "rotate",
                                attr:from: "0 50 50",
                                attr:to: "360 50 50",
                                attr:dur: "7s",
                                attr:"repeatCount": "indefinite",
                            }
                        }
                        rect {
                            attr:x: "-2.17916%",
                            attr:y: "35.4267%",
                            attr:width: "100%",
                            attr:height: "100%",
                            attr:fill: "url(#Gradient2)",
                            attr:transform: "rotate(255.072 50 50)",
                            animate {
                                attr:"attributeName": "x",
                                attr:dur: "23s",
                                attr:values: "-25%;0%;-25%",
                                attr:"repeatCount": "indefinite",
                            }
                            animate {
                                attr:"attributeName": "y",
                                attr:dur: "24s",
                                attr:values: "0%;50%;0%",
                                attr:"repeatCount": "indefinite",
                            }
                            "animate-transform" {
                                attr:"attributeName": "transform",
                                attr:type: "rotate",
                                attr:from: "0 50 50",
                                attr:to: "360 50 50",
                                attr:dur: "12s",
                                attr:"repeatCount": "indefinite",
                            }
                        }
                        rect {
                            attr:x: "9.00483%",
                            attr:y: "14.5733%",
                            attr:width: "100%",
                            attr:height: "100%",
                            attr:fill: "url(#Gradient3)",
                            attr:transform: "rotate(139.903 50 50)",
                            animate {
                                attr:"attributeName": "x",
                                attr:dur: "25s",
                                attr:values: "0%;25%;0%",
                                attr:"repeatCount": "indefinite",
                            }
                            animate {
                                attr:"attributeName": "y",
                                attr:dur: "12s",
                                attr:values: "0%;25%;0%",
                                attr:"repeatCount": "indefinite",
                            }
                            "animate-transform" {
                                attr:"attributeName": "transform",
                                attr:type: "rotate",
                                attr:from: "360 50 50",
                                attr:to: "0 50 50",
                                attr:dur: "9s",
                                attr:"repeatCount": "indefinite",
                            }
                        }
                    }
                }
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
                            attr:"view-box": "0 0 24 24",
                            fill: "none",
                            stroke: "white",
                            attr:"stroke-width": "2",
                            attr:"stroke-linecap": "round",
                            attr:"stroke-linejoin": "round",
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
