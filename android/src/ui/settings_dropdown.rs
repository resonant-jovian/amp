use crate::components::settings::{Language, Theme, load_settings, save_settings};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::{MdBugReport, MdInfo, MdSettings};
use dioxus_free_icons::icons::md_navigation_icons::{MdExpandLess, MdExpandMore};
use dioxus_free_icons::icons::md_social_icons::MdNotifications;
/// Settings dropdown panel component
///
/// Displays a slide-in panel from the top-right with expandable settings sections.
/// Each section can be independently expanded/collapsed.
/// Uses neumorphic design system with gradient header matching the HTML reference.
/// Settings items use scaled-down address-item container styling.
///
/// # Sections
/// * Aviseringar - Notification preferences (städas nu, 6h, 1 day)
/// * Inställningar - Theme toggle and language selector
/// * Info - App information
/// * Debug - Debug mode toggle
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
    let mut settings = use_signal(load_settings);
    let mut aviseringar_open = use_signal(|| false);
    let mut installningar_open = use_signal(|| false);
    let mut info_open = use_signal(|| false);
    let mut debug_open = use_signal(|| false);
    if !is_open {
        return rsx!();
    }
    // let toggle_stadning_nu = move |_| {
    //     let mut s = settings.write();
    //     s.notifications.stadning_nu = !s.notifications.stadning_nu;
    //     let _ = save_settings(&s);
    // };
    // let toggle_sex_timmar = move |_| {
    //     let mut s = settings.write();
    //     s.notifications.sex_timmar = !s.notifications.sex_timmar;
    //     let _ = save_settings(&s);
    // };
    // let toggle_en_dag = move |_| {
    //     let mut s = settings.write();
    //     s.notifications.en_dag = !s.notifications.en_dag;
    //     let _ = save_settings(&s);
    // };
    // let toggle_theme = move |_| {
    //     let mut s = settings.write();
    //     s.theme = match s.theme {
    //         Theme::Dark => Theme::Light,
    //         Theme::Light => Theme::Dark,
    //     };
    //     let _ = save_settings(&s);
    // };
    // let change_language = move |evt: Event<FormData>| {
    //     let value = evt.value();
    //     let mut s = settings.write();
    //     s.language = match value.as_str() {
    //         "English" => Language::English,
    //         "Español" => Language::Espanol,
    //         "Français" => Language::Francais,
    //         _ => Language::Svenska,
    //     };
    //     let _ = save_settings(&s);
    // };
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
                    div { class: "settings-section",
                        button {
                            class: "settings-section-header",
                            onclick: move |_| aviseringar_open.set(!aviseringar_open()),
                            "aria-expanded": if aviseringar_open() { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon {
                                    icon: MdNotifications,
                                    width: 16,
                                    height: 16,
                                }
                                span { "Aviseringar" }
                            }
                            span { class: "settings-section-arrow",
                                if aviseringar_open() {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon {
                                        icon: MdExpandMore,
                                        width: 16,
                                        height: 16,
                                    }
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if aviseringar_open() { "false" } else { "true" },
                            div { class: "settings-section-body",
                                 div { class: "settings-toggle-item",
                            //         div { class: "settings-item-text",
                            //             div { class: "settings-item-label", "Städas nu" }
                            //             div { class: "settings-item-description",
                            //                 "Avisering när städning pågår"
                            //             }
                            //         }
                            //         label { class: "settings-toggle-switch",
                            //             input {
                            //                 r#type: "checkbox",
                            //                 checked: settings.read().notifications.stadning_nu,
                            //                 onchange: toggle_stadning_nu,
                            //             }
                            //             span { class: "settings-switch-container",
                            //                 span {
                            //                     class: "settings-switch-thumb",
                            //                     "data-active": if settings.read().notifications.stadning_nu { "true" } else { "false" },
                            //                     span { class: "settings-led" }
                            //                 }
                            //             }
                            //         }
                            //     }
                            //     div { class: "settings-toggle-item",
                            //         div { class: "settings-item-text",
                            //             div { class: "settings-item-label", "6 timmar" }
                            //             div { class: "settings-item-description",
                            //                 "Avisering 6 timmar innan städning"
                            //             }
                            //         }
                            //         label { class: "settings-toggle-switch",
                            //             input {
                            //                 r#type: "checkbox",
                            //                 checked: settings.read().notifications.sex_timmar,
                            //                 onchange: toggle_sex_timmar,
                            //             }
                            //             span { class: "settings-switch-container",
                            //                 span {
                            //                     class: "settings-switch-thumb",
                            //                     "data-active": if settings.read().notifications.sex_timmar { "true" } else { "false" },
                            //                     span { class: "settings-led" }
                            //                 }
                            //             }
                            //         }
                            //     }
                            //     div { class: "settings-toggle-item",
                            //         div { class: "settings-item-text",
                            //             div { class: "settings-item-label", "1 dag" }
                            //             div { class: "settings-item-description",
                            //                 "Avisering 1 dag innan städning"
                            //             }
                            //         }
                            //         label { class: "settings-toggle-switch",
                            //             input {
                            //                 r#type: "checkbox",
                            //                 checked: settings.read().notifications.en_dag,
                            //                 onchange: toggle_en_dag,
                            //             }
                            //             span { class: "settings-switch-container",
                            //                 span {
                            //                     class: "settings-switch-thumb",
                            //                     "data-active": if settings.read().notifications.en_dag { "true" } else { "false" },
                            //                     span { class: "settings-led" }
                            //                 }
                            //             }
                            //         }
                                 }
                            }
                        }
                    }
                    div { class: "settings-section",
                        button {
                            class: "settings-section-header",
                            onclick: move |_| installningar_open.set(!installningar_open()),
                            "aria-expanded": if installningar_open() { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon { icon: MdSettings, width: 16, height: 16 }
                                span { "Inställningar" }
                            }
                            span { class: "settings-section-arrow",
                                if installningar_open() {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon {
                                        icon: MdExpandMore,
                                        width: 16,
                                        height: 16,
                                    }
                                }
                            }
                        }
                         div {
                             class: "settings-section-content",
                             "aria-hidden": if installningar_open() { "false" } else { "true" },
                             div { class: "settings-section-body",
                                div { class: "settings-toggle-item",
                                     div { class: "settings-item-text",
                        //                 div { class: "settings-item-label", "Mörkt läge" }
                        //                 div { class: "settings-item-description",
                        //                     "Växla mellan ljust och mörkt tema"
                        //                 }
                        //             }
                        //             label { class: "settings-toggle-switch",
                        //                 input {
                        //                     r#type: "checkbox",
                        //                     checked: settings.read().theme == Theme::Dark,
                        //                     onchange: toggle_theme,
                        //                 }
                        //                 span { class: "settings-switch-container",
                        //                     span {
                        //                         class: "settings-switch-thumb",
                        //                         "data-active": if settings.read().theme == Theme::Dark { "true" } else { "false" },
                        //                         span { class: "settings-led" }
                        //                     }
                        //                 }
                        //             }
                        //         }
                        //         div { class: "settings-select-item",
                        //             div { class: "settings-item-text",
                        //                 div { class: "settings-item-label", "Språk" }
                        //                 div { class: "settings-item-description",
                        //                     "Välj ditt föredragna språk"
                        //                 }
                        //             }
                        //             select {
                        //                 class: "settings-select",
                        //                 value: settings.read().language.as_str(),
                        //                 onchange: change_language,
                        //                 option { value: "Svenska", "Svenska" }
                        //                 option { value: "English", "English" }
                        //                 option { value: "Español", "Español" }
                        //                 option { value: "Français", "Français" }
                                     }
                                 }
                             }
                        }
                    }
                    div { class: "settings-section",
                        button {
                            class: "settings-section-header",
                            onclick: move |_| info_open.set(!info_open()),
                            "aria-expanded": if info_open() { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon { icon: MdInfo, width: 16, height: 16 }
                                span { "Info" }
                            }
                            span { class: "settings-section-arrow",
                                if info_open() {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon {
                                        icon: MdExpandMore,
                                        width: 16,
                                        height: 16,
                                    }
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if info_open() { "false" } else { "true" },
                            div { class: "settings-section-body",
                                h4 { class: "info-heading", "Om appen" }
                                p { class: "info-text",
                                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut "
                                    "labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco "
                                    "laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in "
                                    "voluptate velit esse cillum dolore eu fugiat nulla pariatur."
                                }
                            }
                        }
                    }
                    div { class: "settings-section",
                        button {
                            class: "settings-section-header",
                            onclick: move |_| debug_open.set(!debug_open()),
                            "aria-expanded": if debug_open() { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon {
                                    icon: MdBugReport,
                                    width: 16,
                                    height: 16,
                                }
                                span { "Debug" }
                            }
                            span { class: "settings-section-arrow",
                                if debug_open() {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon {
                                        icon: MdExpandMore,
                                        width: 16,
                                        height: 16,
                                    }
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if debug_open() { "false" } else { "true" },
                            div { class: "settings-section-body",
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label", "Debug adresser" }
                                        div { class: "settings-item-description",
                                            "Visa felsökningsinformation för adresser"
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "checkbox",
                                            checked: debug_mode,
                                            onchange: move |_| on_toggle_debug.call(()),
                                        }
                                        span { class: "settings-switch-container",
                                            span {
                                                class: "settings-switch-thumb",
                                                "data-active": if debug_mode { "true" } else { "false" },
                                                span { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
