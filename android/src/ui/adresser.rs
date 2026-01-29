use crate::ui::StoredAddress;
use dioxus::prelude::*;

#[component]
pub fn Adresser(
    stored_addresses: Vec<StoredAddress>,
    on_toggle_active: EventHandler<usize>,
    on_remove_address: EventHandler<usize>,
) -> Element {
    rsx! {

        div { class: "stored-addresses-container",
            div { class: "stored-addresses-header",
                "Adresser"
            }
            div { class: "stored-addresses-content",
                div { id: "addressList",
                    {
                        stored_addresses
                            .iter()
                            .enumerate()
                            .map(|(idx, addr)| {
                                let active_class = if addr.active { "active" } else { "inactive" };
                                let address_display = format!(
                                    "{} {}, {}",
                                    addr.gata,
                                    addr.gatunummer,
                                    addr.postnummer,
                                );
                                rsx! {
                                    div { key: "{idx}", class: "address-item {active_class}",
                                        div { class: "address-info",
                                            div { class: "address-text", "{address_display}" }
                                        }
                                        div { class: "address-actions",
                                            div {
                                                class: "toggle-switch",
                                                onclick: move |_| on_toggle_active.call(idx),
                                                div { class: "switch-container",
                                                    div { class: "switch-thumb {active_class}",
                                                        div { class: "led" }
                                                    }
                                                }
                                            }
                                            button {
                                                class: "btn-remove",
                                                onclick: move |_| on_remove_address.call(idx),
                                                "×"
                                            }
                                        }
                                    }
                                }
                            })
                    }
                }
            }
        }
    }
}
