use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::{MdBugReport, MdInfo, MdSettings};
use dioxus_free_icons::icons::md_social_icons::MdNotifications;
/// Settings dropdown panel component
///
/// Displays a slide-in panel from the top-right with settings menu items.
/// Closes when clicking outside the panel or the close button.
/// Uses neumorphic design system with animated entrance.
///
/// # Props
/// * `is_open` - Controls visibility of the dropdown
/// * `on_close` - Event handler called when user closes the panel
/// * `debug_mode` - Current debug mode state
/// * `on_toggle_debug` - Event handler called when debug mode is toggled
///
/// # Example
/// ```rust
/// SettingsDropdown {
///     is_open: show_settings(),
///     on_close: handle_close_settings,
///     debug_mode: is_debug_mode(),
///     on_toggle_debug: handle_toggle_debug,
/// }
/// ```
#[component]
pub fn SettingsDropdown(
    is_open: bool,
    on_close: EventHandler<()>,
    debug_mode: bool,
    on_toggle_debug: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx!();
    }
    rsx! {
        div { class: "settings-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "settings-dropdown",
                onclick: move |e| e.stop_propagation(),
                div { class: "settings-header",
                    h3 { "Inställningar" }
                    button {
                        class: "settings-close-btn",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "settings-content",
                    button {
                        class: "settings-item",
                        onclick: move |_| {
                            info!("General settings clicked");
                        },
                        Icon { icon: MdSettings, width: 20, height: 20 }
                        span { "Allmänna inställningar" }
                    }
                    button {
                        class: "settings-item",
                        onclick: move |_| {
                            info!("Notifications clicked");
                        },
                        Icon { icon: MdNotifications, width: 20, height: 20 }
                        span { "Aviseringar" }
                    }
                    button {
                        class: "settings-item",
                        onclick: move |_| {
                            info!("About clicked");
                        },
                        Icon { icon: MdInfo, width: 20, height: 20 }
                        span { "Om appen" }
                    }
                    div { class: "settings-divider" }
                    div { class: "settings-toggle-item",
                        div { class: "settings-toggle-label",
                            Icon { icon: MdBugReport, width: 20, height: 20 }
                            span { "Debugläge" }
                            span { class: "settings-toggle-hint", "(Exempeladresser)" }
                        }
                        label { class: "toggle-switch",
                            input {
                                r#type: "checkbox",
                                checked: debug_mode,
                                onchange: move |_| on_toggle_debug.call(()),
                            }
                            span { class: "switch-thumb" }
                        }
                    }
                    div { class: "settings-divider" }
                    div { class: "settings-version", "Version 1.0.0" }
                }
            }
        }
    }
}
