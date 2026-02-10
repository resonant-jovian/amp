use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_device_icons::MdGraphicEq;
#[component]
pub fn Adresser() -> Element {
    rsx! {
        div { class: "stored-addresses",
            h2 { "Adresser" }
            div { id: "addressList",
                div { class: "address-item",
                    div { class: "address-text",
                        button { class: "address-info-icon", onclick: move |_| (),
                            Icon { icon: MdGraphicEq, width: 16, height: 16 }
                        }
                        span { "Example Address 1" }
                    }
                    div { class: "address-actions",
                        label { class: "switch",
                            input { r#type: "checkbox" }
                            span { class: "slider" }
                        }
                        button { class: "btn-remove", onclick: move |_| (), "×" }
                    }
                }
            }
        }
    }
}
