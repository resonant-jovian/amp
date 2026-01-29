use crate::ui::StoredAddress;
use dioxus::prelude::*;
#[component]
pub fn Adresser(
    stored_addresses: Vec<StoredAddress>,
    on_toggle_active: EventHandler<usize>,
    on_remove_address: EventHandler<usize>,
) -> Element {
    rsx! {
        div { class: "category-container category-addresses",
            div { class: "category-title", "Adresser" }
            div { class: "category-content",
                if stored_addresses.is_empty() {
                    div { class: "empty-state", "Inga adresser tillagda" }
                } else {
                    div { id: "addressList",
                        {
                            stored_addresses
                                .iter()
                                .enumerate()
                                .map(|(idx, addr)| {
                                    let address_display = format!(
                                        "{} {}, {}",
                                        addr.gata,
                                        addr.gatunummer,
                                        addr.postnummer,
                                    );
                                    let is_active = addr.active;
                                    rsx! {
                                        div { key: "{idx}", class: "address-item",
                                            div { class: "address-text", "{address_display}" }
                                            div { class: "address-actions",
                                                div {
                                                    class: "toggle-switch",
                                                    onclick: move |_| on_toggle_active.call(idx),
                                                    div { class: "switch-container",
                                                        div {
                                                            class: "switch-thumb",
                                                            "data-active": "{is_active}",
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
}
