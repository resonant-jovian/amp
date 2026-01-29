use dioxus::prelude::*;
#[component]
pub fn Adresser() -> Element {
    rsx! {
        div { class: "stored-addresses",
            h2 { "Adresser" }
            div { id: "addressList" }
        }
    }
}
