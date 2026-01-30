use crate::ui::StoredAddress;
use dioxus::prelude::*;
/// Address list component displaying all stored addresses with toggle and remove controls
///
/// # Props
/// * `stored_addresses` - Vector of StoredAddress entries to display
/// * `on_toggle_active` - Event handler for toggling address active state
/// * `on_remove_address` - Event handler for removing an address
#[component]
pub fn Addresses(
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
                                .map(|addr| {
                                    let address_display = format!(
                                        "{} {}, {}",
                                        addr.street,
                                        addr.street_number,
                                        addr.postal_code,
                                    );
                                    let is_active = addr.active;
                                    let addr_id = addr.id;
                                    rsx! {
                                        div { key: "{addr_id}", class: "address-item",
                                            div { class: "address-text", "{address_display}" }
                                            div { class: "address-actions",
                                                div {
                                                    class: "toggle-switch",
                                                    onclick: move |_| on_toggle_active.call(addr_id),
                                                    div { class: "switch-container",
                                                        div { class: "switch-thumb", "data-active": "{is_active}",
                                                            div { class: "led" }
                                                        }
                                                    }
                                                }
                                                button {
                                                    class: "btn-remove",
                                                    onclick: move |_| on_remove_address.call(addr_id),
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
