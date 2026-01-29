use dioxus::prelude::*;
#[component]
pub fn TopBar(mut on_add_address: EventHandler<(String, String, String)>) -> Element {
    let mut adress_input = use_signal(String::new);
    let mut gatunummer_input = use_signal(String::new);
    let mut postnummer_input = use_signal(String::new);
    let handle_add_click = move |_| {
        let gata = adress_input.read().clone();
        let gatunummer = gatunummer_input.read().clone();
        let postnummer = postnummer_input.read().clone();
        if !gata.trim().is_empty() && !gatunummer.trim().is_empty()
            && !postnummer.trim().is_empty()
        {
            on_add_address.call((gata, gatunummer, postnummer));
            adress_input.set(String::new());
            gatunummer_input.set(String::new());
            postnummer_input.set(String::new());
        }
    };
    let handle_gps_click = move |_| {
        eprintln!("GPS button clicked - TODO: implement location reading");
    };
    rsx! {
        div { class: "top-bar",
            div { class: "input-column",
                input {
                    id: "streetInput",
                    placeholder: "Adress",
                    r#type: "text",
                    value: "{adress_input.read()}",
                    onchange: move |evt: Event<FormData>| {
                        adress_input.set(evt.value());
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
