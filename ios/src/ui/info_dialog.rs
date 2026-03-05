use crate::components::settings::AppSettings;
use crate::components::translations::t;
use crate::ui::StoredAddress;
use dioxus::prelude::*;
/// Information dialog component for displaying address details
///
/// Shows comprehensive address information in a modal overlay with formatted rows.
/// Closes when clicking overlay or close button. Uses neumorphic design system.
///
/// # Props
/// * `is_open` - Controls visibility of the dialog
/// * `address` - Optional StoredAddress to display (None when closed)
/// * `on_close` - Event handler called when user closes the dialog
///
/// # Example
/// ```rust
/// InfoDialog {
///     is_open: show_info(),
///     address: selected_address(),
///     on_close: handle_close_info,
/// }
/// ```
#[component]
pub fn InfoDialog(
    is_open: bool,
    address: Option<StoredAddress>,
    on_close: EventHandler<()>,
) -> Element {
    if !is_open || address.is_none() {
        return rsx!();
    }
    let addr = address.unwrap();
    let app_settings = use_context::<Signal<AppSettings>>();
    let tr = move |key: &'static str| t(key, &app_settings().language);
    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "modal-container info-dialog",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h3 { class: "modal-title", {tr("info_dialog.title")} }
                    button {
                        class: "modal-close-btn",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "modal-body info-content",
                    div { class: "info-row",
                        span { class: "info-label", {tr("info_dialog.street")} }
                        span { class: "info-value", "{addr.street}" }
                    }
                    div { class: "info-row",
                        span { class: "info-label", {tr("info_dialog.street_number")} }
                        span { class: "info-value", "{addr.street_number}" }
                    }
                    if !addr.postal_code.is_empty() {
                        div { class: "info-row",
                            span { class: "info-label", {tr("info_dialog.postal_code")} }
                            span { class: "info-value", "{addr.postal_code}" }
                        }
                    }
                    div { class: "info-row",
                        span { class: "info-label", {tr("info_dialog.status")} }
                        span { class: if addr.active { "info-value status-active" } else { "info-value status-inactive" },
                            if addr.active {
                                {tr("info_dialog.active")}
                            } else {
                                {tr("info_dialog.inactive")}
                            }
                        }
                    }
                    div { class: "info-row",
                        span { class: "info-label", {tr("info_dialog.validated")} }
                        span { class: if addr.valid { "info-value status-active" } else { "info-value status-inactive" },
                            if addr.valid {
                                {tr("info_dialog.yes")}
                            } else {
                                {tr("info_dialog.no")}
                            }
                        }
                    }
                    if let Some(ref entry) = addr.matched_entry {
                        if let Some(ref taxa) = entry.taxa {
                            div { class: "info-row",
                                span { class: "info-label", {tr("info_dialog.taxa")} }
                                span { class: "info-value", "{taxa}" }
                            }
                        }
                        if let Some(ref info) = entry.info {
                            div { class: "info-row",
                                span { class: "info-label", {tr("info_dialog.info")} }
                                span { class: "info-value", "{info}" }
                            }
                        }
                        if let Some(ref typ) = entry.typ_av_parkering {
                            div { class: "info-row",
                                span { class: "info-label", {tr("info_dialog.type")} }
                                span { class: "info-value", "{typ}" }
                            }
                        }
                        if let Some(platser) = entry.antal_platser {
                            div { class: "info-row",
                                span { class: "info-label", {tr("info_dialog.spots")} }
                                span { class: "info-value", "{platser}" }
                            }
                        }
                    }
                    if addr.matched_entry.as_ref().is_none_or(|e| e.taxa.is_none()) {
                        if let Some(ref parking) = addr.parking_info {
                            if let Some(ref taxa) = parking.taxa {
                                div { class: "info-row",
                                    span { class: "info-label", {tr("info_dialog.taxa")} }
                                    span { class: "info-value", "{taxa}" }
                                }
                            }
                            if let Some(ref typ) = parking.typ_av_parkering {
                                div { class: "info-row",
                                    span { class: "info-label", {tr("info_dialog.type")} }
                                    span { class: "info-value", "{typ}" }
                                }
                            }
                            if let Some(platser) = parking.antal_platser {
                                div { class: "info-row",
                                    span { class: "info-label", {tr("info_dialog.spots")} }
                                    span { class: "info-value", "{platser}" }
                                }
                            }
                        }
                    }
                }
                div { class: "modal-actions",
                    button {
                        class: "modal-btn modal-btn-primary",
                        onclick: move |_| on_close.call(()),
                        {tr("info_dialog.close")}
                    }
                }
            }
        }
    }
}
