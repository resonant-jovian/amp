use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::{MdInfo, MdSettings};
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
///
/// # Example
/// ```rust
/// SettingsDropdown {
///     is_open: show_settings(),
///     on_close: handle_close_settings,
/// }
/// ```
#[component]
pub fn SettingsDropdown(
    is_open: bool,
    on_close: EventHandler<()>,
) -> Element {
    if !is_open {
        return None;
    }

    rsx! {
        div { 
            class: "settings-overlay",
            onclick: move |_| on_close.call(()),
            
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
                            // TODO: Navigate to general settings
                        },
                        Icon { icon: MdSettings, width: 20, height: 20 }
                        span { "Allmänna inställningar" }
                    }
                    
                    button { 
                        class: "settings-item",
                        onclick: move |_| {
                            info!("Notifications clicked");
                            // TODO: Navigate to notification settings
                        },
                        Icon { icon: MdNotifications, width: 20, height: 20 }
                        span { "Aviseringar" }
                    }
                    
                    button { 
                        class: "settings-item",
                        onclick: move |_| {
                            info!("About clicked");
                            // TODO: Navigate to about page
                        },
                        Icon { icon: MdInfo, width: 20, height: 20 }
                        span { "Om appen" }
                    }
                    
                    div { class: "settings-divider" }
                    
                    div { class: "settings-version",
                        "Version 1.0.0"
                    }
                }
            }
        }
    }
}
