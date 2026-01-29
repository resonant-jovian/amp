use crate::ui::StoredAddress;
use dioxus::prelude::*;

#[component]
pub fn Adresser(
    stored_addresses: Vec<StoredAddress>,
    on_toggle_active: EventHandler<usize>,
    on_remove_address: EventHandler<usize>,
) -> Element {
    rsx! {

        div { class: "stored-addresses",
            h2 { "Adresser" }
            div { id: "addressList",
                {stored_addresses.iter().enumerate().map(|(idx, addr)| {
                    let validation_indicator = if addr.valid {
                        rsx! { span { class: "valid-indicator", "✓" } }
                    } else {
                        rsx! { span { class: "invalid-indicator", "✗" } }
                    };

                    let active_class = if addr.active { "active" } else { "inactive" };

                    rsx! {
                        div { key: "{idx}", class: "address-item {active_class}",
                            div { class: "address-header",
                                {validation_indicator}
                                div { class: "address-text",
                                    "{addr.gata} {addr.gatunummer}, {addr.postnummer}"
                                }
                            }
                            div { class: "address-controls",
                                button {
                                    class: "toggle-button",
                                    onclick: move |_| on_toggle_active.call(idx),
                                    if addr.active { "Dölj" } else { "Visa" }
                                }
                                button {
                                    class: "remove-button",
                                    onclick: move |_| on_remove_address.call(idx),
                                    "×"
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}
