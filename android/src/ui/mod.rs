pub mod adresser;
pub mod paneler;
pub mod topbar;

use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/style.css");

use crate::ui::{
    adresser::Adresser,
    paneler::{Active, Day, Month, NotValid, Six},
    topbar::TopBar,
};

#[component]
pub fn App() -> Element {
    // TopBar handles input, JavaScript populates the data containers
    let handle_add_address = move |args: (String, String, String)| {
        let (gata, gatunummer, postnummer) = args;
        // TODO: Pass to JavaScript/Android layer to:
        // 1. Validate against database
        // 2. Add to storage
        // 3. Update UI containers
        eprintln!(
            "Address added: {} {}, {}",
            gata, gatunummer, postnummer
        );
    };

    rsx! {
        Stylesheet { href: CSS }
        div {
            class: "app-wrapper",
            TopBar {
                on_add_address,
            },
            div {
                class: "app-container",
                Adresser {},
                div {
                    class: "categories-section",
                    Active {},
                    Six {},
                    Day {},
                    Month {},
                    NotValid {},
                }
            }
        }
    }
}
