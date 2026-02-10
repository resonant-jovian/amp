use dioxus::prelude::*;
/// Confirmation dialog component for destructive actions
///
/// Displays a modal overlay with confirm/cancel buttons to prevent accidental operations.
/// Uses neumorphic design system with matching shadows and gradients.
///
/// # Props
/// * `is_open` - Controls visibility of the dialog
/// * `title` - Dialog header text
/// * `message` - Confirmation message body text
/// * `on_confirm` - Event handler called when user confirms action
/// * `on_cancel` - Event handler called when user cancels or dismisses dialog
///
/// # Example
/// ```rust
/// ConfirmDialog {
///     is_open: show_confirm(),
///     title: "Bekräfta borttagning".to_string(),
///     message: "Är du säker på att du vill ta bort denna adress?".to_string(),
///     on_confirm: handle_confirm_remove,
///     on_cancel: handle_cancel,
/// }
/// ```
#[component]
pub fn ConfirmDialog(
    is_open: bool,
    title: String,
    message: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx!();
    }
    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_cancel.call(()),
            div {
                class: "modal-container confirm-dialog",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h3 { class: "confirm-dialog-title", "{title}" }
                }
                div { class: "modal-body",
                    p { "{message}" }
                }
                div { class: "modal-actions",
                    button {
                        class: "modal-btn modal-btn-cancel",
                        onclick: move |_| on_cancel.call(()),
                        "Avbryt"
                    }
                    button {
                        class: "modal-btn modal-btn-confirm",
                        onclick: move |_| on_confirm.call(()),
                        "Ta bort"
                    }
                }
            }
        }
    }
}
