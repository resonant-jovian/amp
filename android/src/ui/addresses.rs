use crate::ui::StoredAddress;
use crate::ui::confirm_dialog::ConfirmDialog;
use crate::ui::info_dialog::InfoDialog;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_device_icons::MdGraphicEq;
/// Address list component displaying all stored addresses with toggle and remove controls
///
/// Includes confirmation dialog for removals and info dialog for viewing address details.
/// Uses neumorphic design with smooth animations and state management.
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
    let mut show_confirm = use_signal(|| false);
    let mut pending_remove_id = use_signal(|| None::<usize>);
    let mut show_info = use_signal(|| false);
    let mut selected_address = use_signal(|| None::<StoredAddress>);
    let mut sorted_addresses = stored_addresses.clone();
    sorted_addresses.sort_by(|a, b| a.postal_code.cmp(&b.postal_code));
    let mut handle_remove_click = move |addr_id: usize| {
        info!("Remove button clicked for address id: {}", addr_id);
        pending_remove_id.set(Some(addr_id));
        show_confirm.set(true);
    };
    let handle_confirm_remove = move |_| {
        if let Some(id) = pending_remove_id() {
            info!("Removal confirmed for address id: {}", id);
            on_remove_address.call(id);
        }
        show_confirm.set(false);
        pending_remove_id.set(None);
    };
    let handle_cancel = move |_| {
        info!("Removal cancelled");
        show_confirm.set(false);
        pending_remove_id.set(None);
    };
    let mut handle_info_click = move |addr: StoredAddress| {
        info!(
            "Info button clicked for address: {} {}",
            addr.street, addr.street_number
        );
        selected_address.set(Some(addr));
        show_info.set(true);
    };
    let handle_close_info = move |_| {
        info!("Info dialog closed");
        show_info.set(false);
        selected_address.set(None);
    };
    rsx! {
        div { class: "category-container category-addresses",
            div { class: "category-title", "Adresser" }
            div { class: "category-content",
                if sorted_addresses.is_empty() {
                    div { class: "empty-state", "Inga adresser tillagda" }
                } else {
                    div { id: "addressList",
                        {
                            sorted_addresses
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
                                    let addr_clone = addr.clone();
                                    rsx! {
                                        div { key: "{addr_id}", class: "address-item",
                                            div { class: "address-text",
                                                button {
                                                    class: "address-info-icon",
                                                    onclick: move |_| handle_info_click(addr_clone.clone()),
                                                    Icon { icon: MdGraphicEq, width: 16, height: 16 }
                                                }
                                                span { "{address_display}" }
                                            }
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
                                                    onclick: move |_| handle_remove_click(addr_id),
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
            ConfirmDialog {
                is_open: show_confirm(),
                title: "Bekräfta borttagning".to_string(),
                message: "Är du säker på att du vill ta bort denna adress?".to_string(),
                on_confirm: handle_confirm_remove,
                on_cancel: handle_cancel,
            }
            InfoDialog {
                is_open: show_info(),
                address: selected_address(),
                on_close: handle_close_info,
            }
        }
    }
}
