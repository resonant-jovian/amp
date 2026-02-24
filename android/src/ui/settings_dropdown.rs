#![allow(clippy::suspicious_else_formatting)]
use crate::android_bridge::{export_file_jni, import_file_jni, open_url};
use crate::components::notifications::{notify_active, notify_one_day, notify_six_hours};
use crate::components::settings::{
    AppSettings, AutocompleteSource, Language, Theme, get_settings_storage_path,
    import_settings_from_path, load_settings, save_settings,
};
use crate::components::storage::{get_local_storage_path, import_local_from_path};
use crate::components::translations::t;
use crate::ui::StoredAddress;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_brands_icons::FaDev;
use dioxus_free_icons::icons::md_action_icons::{MdInfo, MdSettings, MdSwapHoriz};
use dioxus_free_icons::icons::md_navigation_icons::MdExpandLess;
use dioxus_free_icons::icons::md_social_icons::MdNotificationsActive;
use std::time::Duration;
/// Represents which settings section is currently open
#[derive(Clone, Copy, PartialEq)]
enum OpenSection {
    None,
    ImportExport,
    Aviseringar,
    Installningar,
    Info,
    Debug,
}
/// Type of data being imported
#[derive(Clone, Copy, PartialEq)]
enum ImportType {
    Addresses,
    Settings,
}
/// Settings dropdown panel component
#[component]
pub fn SettingsDropdown(
    is_open: bool,
    on_close: EventHandler<()>,
    debug_mode: bool,
    on_toggle_debug: EventHandler<()>,
    on_data_imported: EventHandler<()>,
) -> Element {
    let mut settings = use_context::<Signal<AppSettings>>();
    let mut open_section = use_signal(|| OpenSection::None);
    let mut show_overwrite_warning = use_signal(|| false);
    let mut pending_import_type = use_signal(|| ImportType::Addresses);
    let mut show_error_dialog = use_signal(|| false);
    let mut error_message = use_signal(String::new);
    let mut show_success_dialog = use_signal(|| false);
    let mut success_message = use_signal(String::new);
    let tr = move |key: &'static str| t(key, &settings().language);
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
    let on_toggle_darkmode = move |_| {
        let mut current = settings();
        current.theme = if current.theme == Theme::Dark {
            Theme::Light
        } else {
            Theme::Dark
        };
        save_settings(&current);
        settings.set(current);
    };
    let on_set_language_sv = move |_| {
        let mut current = settings();
        current.language = Language::Svenska;
        save_settings(&current);
        settings.set(current);
    };
    let on_set_language_en = move |_| {
        let mut current = settings();
        current.language = Language::English;
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
    let handle_export_addresses = move |_| {
        spawn(async move {
            match get_local_storage_path() {
                Ok(path) => match export_file_jni(&path, "local.parquet") {
                    Ok(()) => {
                        success_message.set(
                            t("msg.export_success.addresses", &settings().language).to_string(),
                        );
                        show_success_dialog.set(true);
                    }
                    Err(e) if e == "cancelled" => {}
                    Err(e) => {
                        error_message.set(format!(
                            "{}: {}",
                            t("msg.export_fail", &settings().language),
                            e,
                        ));
                        show_error_dialog.set(true);
                    }
                },
                Err(e) => {
                    error_message.set(format!(
                        "{}: {}",
                        t("msg.export_no_addr_file", &settings().language),
                        e,
                    ));
                    show_error_dialog.set(true);
                }
            }
        });
    };
    let handle_export_settings = move |_| {
        spawn(async move {
            match get_settings_storage_path() {
                Ok(path) => match export_file_jni(&path, "settings.parquet") {
                    Ok(()) => {
                        success_message.set(
                            t("msg.export_success.settings", &settings().language).to_string(),
                        );
                        show_success_dialog.set(true);
                    }
                    Err(e) if e == "cancelled" => {}
                    Err(e) => {
                        error_message.set(format!(
                            "{}: {}",
                            t("msg.export_fail", &settings().language),
                            e,
                        ));
                        show_error_dialog.set(true);
                    }
                },
                Err(e) => {
                    error_message.set(format!(
                        "{}: {}",
                        t("msg.export_no_sett_file", &settings().language),
                        e,
                    ));
                    show_error_dialog.set(true);
                }
            }
        });
    };
    let handle_import_addresses_request = move |_| {
        pending_import_type.set(ImportType::Addresses);
        show_overwrite_warning.set(true);
    };
    let handle_import_settings_request = move |_| {
        pending_import_type.set(ImportType::Settings);
        show_overwrite_warning.set(true);
    };
    let handle_confirm_import = move |_| {
        show_overwrite_warning.set(false);
        let import_type = pending_import_type();
        spawn(async move {
            match import_file_jni() {
                Ok(Some(temp_path)) => {
                    let result = match import_type {
                        ImportType::Addresses => import_local_from_path(&temp_path),
                        ImportType::Settings => import_settings_from_path(&temp_path),
                    };
                    let _ = std::fs::remove_file(&temp_path);
                    match result {
                        Ok(()) => {
                            let msg = match import_type {
                                ImportType::Addresses => {
                                    t("msg.import_success.addresses", &settings().language)
                                }
                                ImportType::Settings => {
                                    t("msg.import_success.settings", &settings().language)
                                }
                            };
                            success_message.set(msg.to_string());
                            show_success_dialog.set(true);
                            on_data_imported.call(());
                            if import_type == ImportType::Settings {
                                let reloaded = load_settings();
                                settings.set(reloaded);
                            }
                        }
                        Err(e) => {
                            error_message.set(e);
                            show_error_dialog.set(true);
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    error_message.set(format!(
                        "{}: {}",
                        t("msg.import_fail", &settings().language),
                        e,
                    ));
                    show_error_dialog.set(true);
                }
            }
        });
    };
    let handle_cancel_import = move |_| {
        show_overwrite_warning.set(false);
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
                    h3 { {tr("settings.title")} }
                    button {
                        class: "settings-close-btn",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "settings-content",
                    div { class: "settings-toggle-item",
                        div { class: "settings-item-text",
                            div { class: "settings-item-label", {tr("settings.issue_report.label")} }
                            div { class: "settings-item-description",
                                {tr("settings.issue_report.desc")}
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
                            onclick: move |_| toggle_section(OpenSection::ImportExport),
                            "aria-expanded": if open_section() == OpenSection::ImportExport { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon {
                                    icon: MdSwapHoriz,
                                    width: 16,
                                    height: 16,
                                }
                                span { {tr("settings.import_export.title")} }
                            }
                            span { class: "settings-section-arrow",
                                Icon {
                                    icon: MdExpandLess,
                                    width: 16,
                                    height: 16,
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::ImportExport { "false" } else { "true" },
                            div { class: "settings-section-body",
                                h4 { class: "info-heading",
                                    {tr("settings.import_export.addresses_heading")}
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.import_export.export_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.import_export.export_addr_desc")}
                                        }
                                    }
                                    button {
                                        class: "btn-debug-trigger",
                                        onclick: handle_export_addresses,
                                        "📤"
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.import_export.import_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.import_export.import_addr_desc")}
                                        }
                                    }
                                    button {
                                        class: "btn-debug-trigger",
                                        onclick: handle_import_addresses_request,
                                        "📥"
                                    }
                                }
                                h4 { class: "info-heading",
                                    {tr("settings.import_export.settings_heading")}
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.import_export.export_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.import_export.export_sett_desc")}
                                        }
                                    }
                                    button {
                                        class: "btn-debug-trigger",
                                        onclick: handle_export_settings,
                                        "📤"
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.import_export.import_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.import_export.import_sett_desc")}
                                        }
                                    }
                                    button {
                                        class: "btn-debug-trigger",
                                        onclick: handle_import_settings_request,
                                        "📥"
                                    }
                                }
                            }
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
                                span { {tr("settings.notifications.title")} }
                            }
                            span { class: "settings-section-arrow",
                                Icon {
                                    icon: MdExpandLess,
                                    width: 16,
                                    height: 16,
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::Aviseringar { "false" } else { "true" },
                            div { class: "settings-section-body",
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.notifications.now_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.notifications.now_desc")}
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
                                        div { class: "settings-item-label",
                                            {tr("settings.notifications.6h_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.notifications.6h_desc")}
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
                                        div { class: "settings-item-label",
                                            {tr("settings.notifications.1d_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.notifications.1d_desc")}
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
                                span { {tr("settings.settings.title")} }
                            }
                            span { class: "settings-section-arrow",
                                Icon {
                                    icon: MdExpandLess,
                                    width: 16,
                                    height: 16,
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::Installningar { "false" } else { "true" },
                            div { class: "settings-section-body",
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.settings.darkmode_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.settings.darkmode_desc")}
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "checkbox",
                                            checked: settings().theme == Theme::Dark,
                                            onchange: on_toggle_darkmode,
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().theme == Theme::Dark { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                h4 { class: "info-heading",
                                    {tr("settings.settings.language_heading")}
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.settings.lang_sv")}
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "radio",
                                            name: "language",
                                            checked: settings().language == Language::Svenska,
                                            onchange: on_set_language_sv,
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().language == Language::Svenska { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.settings.lang_en")}
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "radio",
                                            name: "language",
                                            checked: settings().language == Language::English,
                                            onchange: on_set_language_en,
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().language == Language::English { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                h4 { class: "info-heading",
                                    {tr("settings.settings.datasource_heading")}
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.settings.both_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.settings.both_desc")}
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "radio",
                                            name: "autocomplete_source",
                                            checked: settings().autocomplete_source == AutocompleteSource::Both,
                                            onchange: move |_| {
                                                let mut current = settings();
                                                current.autocomplete_source = AutocompleteSource::Both;
                                                save_settings(&current);
                                                settings.set(current);
                                            },
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().autocomplete_source == AutocompleteSource::Both { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.settings.miljo_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.settings.miljo_desc")}
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "radio",
                                            name: "autocomplete_source",
                                            checked: settings().autocomplete_source == AutocompleteSource::MiljoOnly,
                                            onchange: move |_| {
                                                let mut current = settings();
                                                current.autocomplete_source = AutocompleteSource::MiljoOnly;
                                                save_settings(&current);
                                                settings.set(current);
                                            },
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().autocomplete_source == AutocompleteSource::MiljoOnly { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.settings.parkering_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.settings.parkering_desc")}
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "radio",
                                            name: "autocomplete_source",
                                            checked: settings().autocomplete_source == AutocompleteSource::ParkeringOnly,
                                            onchange: move |_| {
                                                let mut current = settings();
                                                current.autocomplete_source = AutocompleteSource::ParkeringOnly;
                                                save_settings(&current);
                                                settings.set(current);
                                            },
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().autocomplete_source == AutocompleteSource::ParkeringOnly { "true" } else { "false" },
                                                div { class: "settings-led" }
                                            }
                                        }
                                    }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.settings.all_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.settings.all_desc")}
                                        }
                                    }
                                    label { class: "settings-toggle-switch",
                                        input {
                                            r#type: "radio",
                                            name: "autocomplete_source",
                                            checked: settings().autocomplete_source == AutocompleteSource::AllAddresses,
                                            onchange: move |_| {
                                                let mut current = settings();
                                                current.autocomplete_source = AutocompleteSource::AllAddresses;
                                                save_settings(&current);
                                                settings.set(current);
                                            },
                                        }
                                        div { class: "settings-switch-container",
                                            div {
                                                class: "settings-switch-thumb",
                                                "data-active": if settings().autocomplete_source == AutocompleteSource::AllAddresses { "true" } else { "false" },
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
                            onclick: move |_| toggle_section(OpenSection::Info),
                            "aria-expanded": if open_section() == OpenSection::Info { "true" } else { "false" },
                            div { class: "settings-section-header-left",
                                Icon { icon: MdInfo, width: 16, height: 16 }
                                span { {tr("settings.info.title")} }
                            }
                            span { class: "settings-section-arrow",
                                Icon {
                                    icon: MdExpandLess,
                                    width: 16,
                                    height: 16,
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::Info { "false" } else { "true" },
                            div { class: "settings-section-body",
                                h4 { class: "info-heading", {tr("settings.info.about_heading")} }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text", {tr("settings.info.p1")} }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text", {tr("settings.info.p2")} }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text", {tr("settings.info.p3")} }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text", {tr("settings.info.p4")} }
                                }
                                div { class: "settings-toggle-item",
                                    div { class: "info-text", {tr("settings.info.p5")} }
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
                                span { {tr("settings.debug.title")} }
                            }
                            span { class: "settings-section-arrow",
                                Icon {
                                    icon: MdExpandLess,
                                    width: 16,
                                    height: 16,
                                }
                            }
                        }
                        div {
                            class: "settings-section-content",
                            "aria-hidden": if open_section() == OpenSection::Debug { "false" } else { "true" },
                            div { class: "settings-section-body",
                                div { class: "settings-toggle-item",
                                    div { class: "settings-item-text",
                                        div { class: "settings-item-label",
                                            {tr("settings.debug.addresses_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.debug.addresses_desc")}
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
                                        div { class: "settings-item-label",
                                            {tr("settings.debug.test_now_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.debug.test_now_desc")}
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
                                        div { class: "settings-item-label",
                                            {tr("settings.debug.test_6h_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.debug.test_6h_desc")}
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
                                        div { class: "settings-item-label",
                                            {tr("settings.debug.test_1d_label")}
                                        }
                                        div { class: "settings-item-description",
                                            {tr("settings.debug.test_1d_desc")}
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
        if show_overwrite_warning() {
            div { class: "modal-overlay", onclick: handle_cancel_import,
                div {
                    class: "modal-container confirm-dialog",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "modal-header",
                        h3 { class: "confirm-dialog-title", {tr("dialog.confirm_import.title")} }
                    }
                    div { class: "modal-body",
                        p { {tr("dialog.confirm_import.body")} }
                    }
                    div { class: "modal-actions",
                        button {
                            class: "modal-btn modal-btn-cancel",
                            onclick: handle_cancel_import,
                            {tr("dialog.confirm_import.cancel")}
                        }
                        button {
                            class: "modal-btn modal-btn-confirm",
                            onclick: handle_confirm_import,
                            {tr("dialog.confirm_import.ok")}
                        }
                    }
                }
            }
        }
        if show_error_dialog() {
            div {
                class: "modal-overlay",
                onclick: move |_| show_error_dialog.set(false),
                div {
                    class: "modal-container confirm-dialog",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "modal-header",
                        h3 { class: "confirm-dialog-title", {tr("dialog.error.title")} }
                    }
                    div { class: "modal-body",
                        p { "{error_message}" }
                    }
                    div { class: "modal-actions",
                        button {
                            class: "modal-btn modal-btn-cancel",
                            onclick: move |_| show_error_dialog.set(false),
                            {tr("dialog.ok")}
                        }
                    }
                }
            }
        }
        if show_success_dialog() {
            div {
                class: "modal-overlay",
                onclick: move |_| show_success_dialog.set(false),
                div {
                    class: "modal-container confirm-dialog",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "modal-header",
                        h3 { class: "confirm-dialog-title", {tr("dialog.success.title")} }
                    }
                    div { class: "modal-body",
                        p { "{success_message}" }
                    }
                    div { class: "modal-actions",
                        button {
                            class: "modal-btn modal-btn-cancel",
                            onclick: move |_| show_success_dialog.set(false),
                            {tr("dialog.ok")}
                        }
                    }
                }
            }
        }
    }
}
