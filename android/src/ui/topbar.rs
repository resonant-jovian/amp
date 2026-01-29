use dioxus::prelude::*;
#[component]
pub fn TopBar(mut on_add_address: EventHandler<(String, String, String)>) -> Element {
    let mut address_input = use_signal(String::new);
    let mut postnummer_input = use_signal(String::new);
    let handle_add_click = move |_| {
        let address_str = address_input.read().clone();
        let postnummer = postnummer_input.read().clone();
        if !address_str.trim().is_empty() && !postnummer.trim().is_empty() {
            let street_words: Vec<&str> = address_str.split_whitespace().collect();
            if street_words.len() >= 2 {
                let gatunummer = street_words[street_words.len() - 1].to_string();
                let gata = street_words[..street_words.len() - 1].join(" ");
                on_add_address.call((gata, gatunummer, postnummer));
                address_input.set(String::new());
                postnummer_input.set(String::new());
            }
        }
    };
    let handle_gps_click = move |_| {
        eprintln!("GPS button clicked - TODO: implement location reading");
    };
    rsx! {
        div { class: "top-bar",
            div { class: "input-column",
                input {
                    id: "addressInput",
                    placeholder: "T.ex: Storgatan 10",
                    r#type: "text",
                    value: "{address_input.read()}",
                    onchange: move |evt: Event<FormData>| {
                        address_input.set(evt.value());
                    },
                }
                input {
                    id: "postalInput",
                    placeholder: "Postnummer",
                    r#type: "text",
                    value: "{postnummer_input.read()}",
                    onchange: move |evt: Event<FormData>| {
                        postnummer_input.set(evt.value());
                    },
                }
            }
            div { class: "button-row",
                button { class: "btn", onclick: handle_add_click, id: "addBtn", "Lägg till" }
                button { class: "btn", onclick: handle_gps_click, id: "gpsBtn", "GPS" }
            }
        }
    }
}
