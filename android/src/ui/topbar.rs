use dioxus::prelude::*;

#[component]
pub fn TopBar(mut on_add_address: EventHandler<(String, String, String)>) -> Element {
    let mut gata_input = use_signal(String::new);
    let mut gatunummer_input = use_signal(String::new);
    let mut postnummer_input = use_signal(String::new);

    let handle_add_click = move |_| {
        let gata = gata_input.read().clone();
        let gatunummer = gatunummer_input.read().clone();
        let postnummer = postnummer_input.read().clone();

        if !gata.trim().is_empty()
            && !gatunummer.trim().is_empty()
            && !postnummer.trim().is_empty()
        {
            on_add_address.call((gata, gatunummer, postnummer));
            // Clear inputs after adding
            gata_input.set(String::new());
            gatunummer_input.set(String::new());
            postnummer_input.set(String::new());
        }
    };

    let handle_gps_click = move |_| {
        // TODO: Implement GPS location reading for Android
        // This will need to call native Android code to fetch device location
        // and populate the input fields with the matched address from the database
        eprintln!("GPS button clicked - TODO: implement location reading");
    };

    rsx! {
        div { class: "top-bar",
            div { class: "input-section",
                div { class: "input-group",
                    input {
                        r#type: "text",
                        id: "streetInput",
                        placeholder: "Gata och nummer",
                        value: "{gata_input.read()}",
                        onchange: move |evt: Event<FormData>| {
                            gata_input.set(evt.value());
                        },
                    }
                    input {
                        r#type: "text",
                        id: "streetNumberInput",
                        placeholder: "Gatunummer",
                        value: "{gatunummer_input.read()}",
                        onchange: move |evt: Event<FormData>| {
                            gatunummer_input.set(evt.value());
                        },
                    }
                    input {
                        r#type: "text",
                        id: "postalInput",
                        placeholder: "Postnummer",
                        value: "{postnummer_input.read()}",
                        onchange: move |evt: Event<FormData>| {
                            postnummer_input.set(evt.value());
                        },
                    }
                }
                div { class: "btn-group",
                    button {
                        class: "btn btn-add",
                        onclick: handle_add_click,
                        "➕ Lägg till"
                    }
                    button {
                        class: "btn btn-gps",
                        onclick: handle_gps_click,
                        "📍 Använd GPS"
                    }
                }
            }
        }
    }
}
