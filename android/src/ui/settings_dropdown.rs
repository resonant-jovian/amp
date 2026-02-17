use crate::android_bridge::open_url;
use crate::components::notifications::{notify_active, notify_one_day, notify_six_hours};
use crate::components::settings::{load_settings, save_settings};
use crate::ui::StoredAddress;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_brands_icons::FaDev;
use dioxus_free_icons::icons::md_action_icons::{MdInfo, MdSettings};
use dioxus_free_icons::icons::md_navigation_icons::MdExpandLess;
use dioxus_free_icons::icons::md_social_icons::MdNotificationsActive;
use std::time::Duration;
/// Represents which settings section is currently open
#[derive(Clone, Copy, PartialEq)]
enum OpenSection {
    None,
    Aviseringar,
    Installningar,
    Info,
    Debug,
}
/// Settings dropdown panel component
///
/// Displays a slide-in panel from the top-right with expandable settings sections.
/// Only one section can be expanded at a time (accordion behaviour).
/// Switching between sections uses a 300ms delay for smooth animation.
/// Uses neumorphic design system with gradient header matching the HTML reference.
/// Settings items use scaled-down address-item container styling.
///
/// # Sections
/// * Aviseringar - Notification preferences (städas nu, 6h, 1 day)
/// * Inställningar - Theme toggle and language selector
/// * Info - App information
/// * Debug - Debug mode toggle + test notification buttons
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
    let mut open_section = use_signal(|| OpenSection::None);
    let toggle_section = move |target_section: OpenSection| {
        spawn(async move {
            let current = open_section();
            if current == target_section {
                open_section.set(OpenSection::None);
            } else if current != OpenSection::None {
                open_section.set(OpenSection::None);
                tokio::time::sleep(Duration::from_millis(300)).await;
                open_section.set(target_section);
            } else {
                open_section.set(target_section);
            }
        });
    };
    let on_toggle_stadning_nu = move |_| {
        let mut current = settings();
        current.notifications.stadning_nu = !current.notifications.stadning_nu;
        save_settings(&current);
        settings.set(current);
    };
    let on_toggle_sex_timmar = move |_| {
        let mut current = settings();
        current.notifications.sex_timmar = !current.notifications.sex_timmar;
        save_settings(&current);
        settings.set(current);
    };
    let on_toggle_en_dag = move |_| {
        let mut current = settings();
        current.notifications.en_dag = !current.notifications.en_dag;
        save_settings(&current);
        settings.set(current);
    };
    let trigger_active_notification = move || {
        let debug_address = StoredAddress {
            id: 99999,
            street: "Debug Street".to_string(),
            street_number: "42".to_string(),
            postal_code: "00000".to_string(),
            valid: true,
            active: true,
            matched_entry: None,
            parking_info: None,
        };
        eprintln!("[Debug] Triggering active notification");
        notify_active(&debug_address);
    };
    let trigger_six_hour_notification = move || {
        let debug_address = StoredAddress {
            id: 99998,
            street: "Debug Street".to_string(),
            street_number: "42".to_string(),
            postal_code: "00000".to_string(),
            valid: true,
            active: false,
            matched_entry: None,
            parking_info: None,
        };
        eprintln!("[Debug] Triggering 6-hour notification");
        notify_six_hours(&debug_address);
    };
    let trigger_one_day_notification = move || {
        let debug_address = StoredAddress {
            id: 99997,
            street: "Debug Street".to_string(),
            street_number: "42".to_string(),
            postal_code: "00000".to_string(),
            valid: true,
            active: false,
            matched_entry: None,
            parking_info: None,
        };
        eprintln!("[Debug] Triggering 1-day notification");
        notify_one_day(&debug_address);
    };
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
                    div { class: "settings-toggle-item",
                        div { class: "settings-item-text",
                            div { class: "settings-item-label", "Issue report" }
                            div { class: "settings-item-description",
                                "Gå till en websida för att rapportera problem eller ge nya idéer"
                            }
                        }
                        button {
                            class: "btn-debug-trigger",
                            onclick: move |_| {
                                open_url("https://github.com/resonant-jovian/amp/issues/new");
                            },
                            "💿"
                        }
                    }
                    div { class: "settings-section",
                        button {
                            class: "settings-section-header",
                            onclick: move |_| toggle_section(OpenSection::Aviseringar),
                            "aria-expanded": if open_section() == OpenSection::Aviseringar { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon {
                                    icon: MdNotificationsActive,
                                    width: 16,
                                    height: 16,
                                }
                                span { "Aviseringar" }
                            }
                            span { class: "settings-section-arrow",
                                if open_section() == OpenSection::Aviseringar {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::Aviseringar { "false" } else { "true" },
                            div { class: "settings-section-body",
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label", "Städas nu" }
                                        div { class: "settings-item-description",
                                            "Avisera när gatustädning pågår"
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "checkbox",
                                            checked: settings().notifications.stadning_nu,
                                            onchange: on_toggle_stadning_nu,
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().notifications.stadning_nu { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label", "6 timmar" }
                                        div { class: "settings-item-description",
                                            "Avisera 6 timmar före gatustädning"
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "checkbox",
                                            checked: settings().notifications.sex_timmar,
                                            onchange: on_toggle_sex_timmar,
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().notifications.sex_timmar { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label", "1 dag" }
                                        div { class: "settings-item-description",
                                            "Avisera 1 dag före gatustädning"
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "checkbox",
                                            checked: settings().notifications.en_dag,
                                            onchange: on_toggle_en_dag,
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().notifications.en_dag { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "settings-section",
                        button {
                            class: "settings-section-header",
                            onclick: move |_| toggle_section(OpenSection::Installningar),
                            "aria-expanded": if open_section() == OpenSection::Installningar { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon { icon: MdSettings, width: 16, height: 16 }
                                span { "Inställningar" }
                            }
                            span { class: "settings-section-arrow",
                                if open_section() == OpenSection::Installningar {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::Installningar { "false" } else { "true" },
                            div { class: "settings-section-body",
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text" }
                                }
                            }
                        }
                    }
                    div { class: "settings-section",
                        button {
                            class: "settings-section-header",
                            onclick: move |_| toggle_section(OpenSection::Info),
                            "aria-expanded": if open_section() == OpenSection::Info { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon { icon: MdInfo, width: 16, height: 16 }
                                span { "Info" }
                            }
                            span { class: "settings-section-arrow",
                                if open_section() == OpenSection::Info {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::Info { "false" } else { "true" },
                            div { class: "settings-section-body",
                                h4 { class: "info-heading", "Om appen" }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text", "Välkommen till amp. " }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text",
                                        "Vi tar inget ansvar för vad Malmö stad väljer att göra, detta är ett verktyg, inget mer. "
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text",
                                        "Appen tar data Malmö lägger upp, formaterar den bättre och använder en kopplings algoritm för att skapa en databas som sedan används här för att du som användare förhoppningsvis ska få mindre böter och Malmö ska kunna städa sina gator utan problem. Inget mer, inget mindre. "
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text",
                                        "Hantering av dagar 29 och 30 i Februari är oklart då Malmös system deklarerar data med en dag i månaden mellan 1 och 30 per datapunkt. Detta innebär bland annat att ingen städning ska hända enligt dem den 31 i månader med det datumet. De säger inget om hur månaden Februari hanteras varken under vanliga år eller skåttår. Nu ignoreras de relevanta adresserna för månad Februari och hamnar istället görs i nästa månad. Är detta rätt? Ingen aning! "
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text",
                                        "Målet är att inte kräva någon internet uppkoppling i appen men för närvarande pga. UI systemet jag använder så kommer appen krascha om jag inter har nätverks rättigheter. All komplicerad koppling sker på en server som skickar en universell app uppdatering när Malmös data uppdateras. "
                                    }
                                }
                            }
                        }
                    }
                    div { class: "settings-section",
                        button {
                            class: "settings-section-header",
                            onclick: move |_| toggle_section(OpenSection::Debug),
                            "aria-expanded": if open_section() == OpenSection::Debug { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon { icon: FaDev, width: 16, height: 16 }
                                span { "Debug" }
                            }
                            span { class: "settings-section-arrow",
                                if open_section() == OpenSection::Debug {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                } else {
                                    Icon {
                                        icon: MdExpandLess,
                                        width: 16,
                                        height: 16,
                                    }
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::Debug { "false" } else { "true" },
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
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if debug_mode { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label", "Test Städas nu" }
                                        div { class: "settings-item-description",
                                            "Skicka aktiv städning-avisering"
                                        }
                                    }
                                    button {
                                        class: "btn-debug-trigger",
                                        onclick: move |_| trigger_active_notification(),
                                        "🚫"
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label", "Test 6 timmar" }
                                        div { class: "settings-item-description",
                                            "Skicka 6-timmars varning"
                                        }
                                    }
                                    button {
                                        class: "btn-debug-trigger",
                                        onclick: move |_| trigger_six_hour_notification(),
                                        "⏰"
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label", "Test 1 dag" }
                                        div { class: "settings-item-description",
                                            "Skicka 1-dags påminnelse"
                                        }
                                    }
                                    button {
                                        class: "btn-debug-trigger",
                                        onclick: move |_| trigger_one_day_notification(),
                                        "📅"
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
