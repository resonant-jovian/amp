use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_image_icons::MdBlurOn;
#[component]
pub fn TopBar() -> Element {
    rsx! {
        div { class: "top-bar",
            div { class: "top-bar-header",
                h1 { class: "top-bar-title", "AMP" }
                button { class: "menu-icon-btn", onclick: move |_| (),
                    Icon { icon: MdBlurOn, width: 24, height: 24 }
                }
            }
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
