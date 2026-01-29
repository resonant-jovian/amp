use dioxus::prelude::*;
#[component]
pub fn TopBar() -> Element {
    rsx! {
        div { class: "top-bar",
            div { class: "input-section",
                div { class: "input-group",
                    input {
                        r#type: "text",
                        id: "streetInput",
                        placeholder: "Gata och nummer",
                    }
                    input {
                        r#type: "text",
                        id: "postalInput",
                        placeholder: "Postnummer",
                    }
                }
                div { class: "btn-group",
                    button { class: "btn btn-add", onclick: move |_| (), "➕ Lägg till" }
                    button { class: "btn btn-gps", onclick: move |_| (), "📍 Använd GPS" }
                }
            }
        }
    }
}
