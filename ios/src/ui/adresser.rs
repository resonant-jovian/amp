use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaCircleInfo;
use dioxus_free_icons::Icon;

#[component]
pub fn Adresser() -> Element {
    rsx! {
        div { class: "stored-addresses",
            h2 { "Adresser" }
            div { id: "addressList",
                // Example address items with info icons
                div { class: "address-item",
                    div { class: "address-text",
                        button {
                            class: "address-info-icon",
                            onclick: move |_| (),
                            Icon {
                                icon: FaCircleInfo,
                                width: 16,
                                height: 16,
                            }
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
