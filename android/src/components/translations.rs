use crate::components::settings::Language;
/// Return the localised string for the given key.
///
/// Falls back to the key itself when no translation is registered.
pub fn t(key: &str, lang: &Language) -> &'static str {
    match lang {
        Language::English => t_en(key),
        _ => t_sv(key),
    }
}
fn t_sv(key: &str) -> &'static str {
    match key {
        "topbar.address_placeholder" => "Adress",
        "topbar.postal_placeholder" => "Postnummer (valfritt)",
        "topbar.add" => "Lägg till",
        "topbar.settings_title" => "Inställningar",
        "topbar.gps_error_title" => "GPS-fel",
        "topbar.gps_not_found" => "Ingen adress hittades inom 20 m från din GPS-position.",
        "topbar.gps_no_permission" => {
            "Kunde inte läsa GPS-position. Kontrollera att platsbehörighet är beviljad och försök igen."
        }
        "topbar.ok" => "OK",
        "settings.title" => "Inställningar",
        "settings.issue_report.label" => "Issue report",
        "settings.issue_report.desc" => {
            "Gå till en websida för att rapportera problem eller ge nya idéer"
        }
        "settings.import_export.title" => "Import / Export",
        "settings.import_export.addresses_heading" => "Adresser",
        "settings.import_export.export_label" => "Exportera",
        "settings.import_export.export_addr_desc" => "Spara adresser till fil",
        "settings.import_export.import_label" => "Importera",
        "settings.import_export.import_addr_desc" => "Ladda adresser från fil",
        "settings.import_export.settings_heading" => "Inställningar",
        "settings.import_export.export_sett_desc" => "Spara inställningar till fil",
        "settings.import_export.import_sett_desc" => "Ladda inställningar från fil",
        "settings.notifications.title" => "Aviseringar",
        "settings.notifications.now_label" => "Städas nu",
        "settings.notifications.now_desc" => "Avisera när gatustädning pågår",
        "settings.notifications.6h_label" => "6 timmar",
        "settings.notifications.6h_desc" => "Avisera 6 timmar före gatustädning",
        "settings.notifications.1d_label" => "1 dag",
        "settings.notifications.1d_desc" => "Avisera 1 dag före gatustädning",
        "settings.settings.title" => "Inställningar",
        "settings.settings.datasource_heading" => "Datakälla",
        "settings.settings.both_label" => "Miljö + Parkering",
        "settings.settings.both_desc" => "Standard",
        "settings.settings.miljo_label" => "Miljö",
        "settings.settings.miljo_desc" => "Enbart gatustädning",
        "settings.settings.parkering_label" => "Parkering",
        "settings.settings.parkering_desc" => "Enbart parkeringszoner",
        "settings.settings.all_label" => "Alla adresser",
        "settings.settings.all_desc" => "Alla ~60k adresser i Malmö (kan vara långsamt)",
        "settings.settings.darkmode_label" => "Mörkt läge",
        "settings.settings.darkmode_desc" => "Växla mörkt/ljust tema",
        "settings.settings.language_heading" => "Språk",
        "settings.settings.lang_sv" => "Svenska",
        "settings.settings.lang_en" => "English",
        "settings.info.title" => "Info",
        "settings.info.about_heading" => "Om appen",
        "settings.debug.title" => "Debug",
        "settings.debug.addresses_label" => "Debug adresser",
        "settings.debug.addresses_desc" => "Visa felsökningsinformation för adresser",
        "settings.debug.test_now_label" => "Test Städas nu",
        "settings.debug.test_now_desc" => "Skicka aktiv städning-avisering",
        "settings.debug.test_6h_label" => "Test 6 timmar",
        "settings.debug.test_6h_desc" => "Skicka 6-timmars varning",
        "settings.debug.test_1d_label" => "Test 1 dag",
        "settings.debug.test_1d_desc" => "Skicka 1-dags påminnelse",
        "dialog.confirm_import.title" => "Bekräfta import",
        "dialog.confirm_import.body" => {
            "Detta kommer att ersätta all nuvarande data. Vill du fortsätta?"
        }
        "dialog.confirm_import.cancel" => "Avbryt",
        "dialog.confirm_import.ok" => "Importera",
        "dialog.error.title" => "Fel",
        "dialog.ok" => "OK",
        "dialog.success.title" => "Klart",
        "msg.export_success.addresses" => "Adresser exporterade!",
        "msg.export_success.settings" => "Inställningar exporterade!",
        "msg.export_fail" => "Export misslyckades",
        "msg.export_no_addr_file" => "Kunde inte hitta adressfilen",
        "msg.export_no_sett_file" => "Kunde inte hitta inställningsfilen",
        "msg.import_success.addresses" => "Adresser importerade!",
        "msg.import_success.settings" => "Inställningar importerade!",
        "msg.import_fail" => "Import misslyckades",
        // panels
        "panel.cleaning_now" => "Städas nu",
        "panel.within_6h" => "Inom 6 timmar",
        "panel.within_1d" => "Inom 1 dag",
        "panel.within_1m" => "Inom 1 månad",
        "panel.more_than_1m" => "30+ dagar",
        "panel.parking_only" => "Endast parkeringsavgift",
        "panel.invalid" => "Ingen städning",
        "panel.no_addresses" => "Inga adresser",
        // addresses panel
        "addresses.title" => "Adresser",
        "addresses.empty" => "Inga adresser tillagda",
        "addresses.confirm_remove_title" => "Bekräfta borttagning",
        "addresses.confirm_remove_msg" => "Är du säker på att du vill ta bort denna adress?",
        // confirm dialog buttons
        "confirm_dialog.cancel" => "Avbryt",
        "confirm_dialog.confirm" => "Ta bort",
        // info dialog
        "info_dialog.title" => "Adressinformation",
        "info_dialog.street" => "Gata:",
        "info_dialog.street_number" => "Gatunummer:",
        "info_dialog.postal_code" => "Postnummer:",
        "info_dialog.status" => "Status:",
        "info_dialog.validated" => "Validerad:",
        "info_dialog.taxa" => "Taxa:",
        "info_dialog.info" => "Info:",
        "info_dialog.type" => "Typ:",
        "info_dialog.spots" => "Platser:",
        "info_dialog.active" => "Aktiv",
        "info_dialog.inactive" => "Inaktiv",
        "info_dialog.yes" => "Ja",
        "info_dialog.no" => "Nej",
        "info_dialog.close" => "Stäng",
        // settings info paragraphs
        "settings.info.p1" => "Välkommen till amp.",
        "settings.info.p2" => {
            "Vi tar inget ansvar för vad Malmö stad väljer att göra, detta är ett verktyg, inget mer."
        }
        "settings.info.p3" => {
            "Appen tar data Malmö lägger upp, formaterar den bättre och använder en kopplings algoritm för att skapa en databas som sedan används här för att du som användare förhoppningsvis ska få mindre böter och Malmö ska kunna städa sina gator utan problem. Inget mer, inget mindre."
        }
        "settings.info.p4" => {
            "Hantering av dagar 29 och 30 i Februari är oklart då Malmös system deklarerar data med en dag i månaden mellan 1 och 30 per datapunkt. Detta innebär bland annat att ingen städning ska hända enligt dem den 31 i månader med det datumet. De säger inget om hur månaden Februari hanteras vare sig under vanliga år eller skottår. Nu ignoreras de relevanta adresserna för månad Februari och hamnar istället i nästa månad. Är detta rätt? Ingen aning!"
        }
        "settings.info.p5" => {
            "Målet är att inte kräva någon internet uppkoppling i appen men för närvarande pga. UI systemet jag använder så kommer appen krascha om jag inte har nätverks rättigheter. All komplicerad koppling sker på en server som skickar en universell app uppdatering när Malmös data uppdateras."
        }
        _ => "",
    }
}
fn t_en(key: &str) -> &'static str {
    match key {
        "topbar.address_placeholder" => "Address",
        "topbar.postal_placeholder" => "Postal code (optional)",
        "topbar.add" => "Add",
        "topbar.settings_title" => "Settings",
        "topbar.gps_error_title" => "GPS error",
        "topbar.gps_not_found" => "No address found within 20 m of your GPS position.",
        "topbar.gps_no_permission" => {
            "Could not read GPS position. Check that location permission is granted and try again."
        }
        "topbar.ok" => "OK",
        "settings.title" => "Settings",
        "settings.issue_report.label" => "Issue report",
        "settings.issue_report.desc" => "Go to a webpage to report issues or share ideas",
        "settings.import_export.title" => "Import / Export",
        "settings.import_export.addresses_heading" => "Addresses",
        "settings.import_export.export_label" => "Export",
        "settings.import_export.export_addr_desc" => "Save addresses to file",
        "settings.import_export.import_label" => "Import",
        "settings.import_export.import_addr_desc" => "Load addresses from file",
        "settings.import_export.settings_heading" => "Settings",
        "settings.import_export.export_sett_desc" => "Save settings to file",
        "settings.import_export.import_sett_desc" => "Load settings from file",
        "settings.notifications.title" => "Notifications",
        "settings.notifications.now_label" => "Cleaning now",
        "settings.notifications.now_desc" => "Notify when street cleaning is happening",
        "settings.notifications.6h_label" => "6 hours",
        "settings.notifications.6h_desc" => "Notify 6 hours before street cleaning",
        "settings.notifications.1d_label" => "1 day",
        "settings.notifications.1d_desc" => "Notify 1 day before street cleaning",
        "settings.settings.title" => "Settings",
        "settings.settings.datasource_heading" => "Data source",
        "settings.settings.both_label" => "Miljö + Parking",
        "settings.settings.both_desc" => "Default",
        "settings.settings.miljo_label" => "Miljö",
        "settings.settings.miljo_desc" => "Street cleaning only",
        "settings.settings.parkering_label" => "Parking",
        "settings.settings.parkering_desc" => "Parking zones only",
        "settings.settings.all_label" => "All addresses",
        "settings.settings.all_desc" => "All ~60k addresses in Malmö (may be slow)",
        "settings.settings.darkmode_label" => "Dark mode",
        "settings.settings.darkmode_desc" => "Toggle dark/light theme",
        "settings.settings.language_heading" => "Language",
        "settings.settings.lang_sv" => "Svenska",
        "settings.settings.lang_en" => "English",
        "settings.info.title" => "Info",
        "settings.info.about_heading" => "About the app",
        "settings.debug.title" => "Debug",
        "settings.debug.addresses_label" => "Debug addresses",
        "settings.debug.addresses_desc" => "Show debug info for addresses",
        "settings.debug.test_now_label" => "Test cleaning now",
        "settings.debug.test_now_desc" => "Send active cleaning notification",
        "settings.debug.test_6h_label" => "Test 6 hours",
        "settings.debug.test_6h_desc" => "Send 6-hour warning",
        "settings.debug.test_1d_label" => "Test 1 day",
        "settings.debug.test_1d_desc" => "Send 1-day reminder",
        "dialog.confirm_import.title" => "Confirm import",
        "dialog.confirm_import.body" => {
            "This will replace all current data. Do you want to continue?"
        }
        "dialog.confirm_import.cancel" => "Cancel",
        "dialog.confirm_import.ok" => "Import",
        "dialog.error.title" => "Error",
        "dialog.ok" => "OK",
        "dialog.success.title" => "Done",
        "msg.export_success.addresses" => "Addresses exported!",
        "msg.export_success.settings" => "Settings exported!",
        "msg.export_fail" => "Export failed",
        "msg.export_no_addr_file" => "Could not find address file",
        "msg.export_no_sett_file" => "Could not find settings file",
        "msg.import_success.addresses" => "Addresses imported!",
        "msg.import_success.settings" => "Settings imported!",
        "msg.import_fail" => "Import failed",
        // panels
        "panel.cleaning_now" => "Cleaning now",
        "panel.within_6h" => "Within 6 hours",
        "panel.within_1d" => "Within 1 day",
        "panel.within_1m" => "Within 1 month",
        "panel.more_than_1m" => "30+ days",
        "panel.parking_only" => "Parking fee only",
        "panel.invalid" => "No cleaning",
        "panel.no_addresses" => "No addresses",
        // addresses panel
        "addresses.title" => "Addresses",
        "addresses.empty" => "No addresses added",
        "addresses.confirm_remove_title" => "Confirm removal",
        "addresses.confirm_remove_msg" => "Are you sure you want to remove this address?",
        // confirm dialog buttons
        "confirm_dialog.cancel" => "Cancel",
        "confirm_dialog.confirm" => "Remove",
        // info dialog
        "info_dialog.title" => "Address information",
        "info_dialog.street" => "Street:",
        "info_dialog.street_number" => "Street number:",
        "info_dialog.postal_code" => "Postal code:",
        "info_dialog.status" => "Status:",
        "info_dialog.validated" => "Validated:",
        "info_dialog.taxa" => "Taxa:",
        "info_dialog.info" => "Info:",
        "info_dialog.type" => "Type:",
        "info_dialog.spots" => "Spots:",
        "info_dialog.active" => "Active",
        "info_dialog.inactive" => "Inactive",
        "info_dialog.yes" => "Yes",
        "info_dialog.no" => "No",
        "info_dialog.close" => "Close",
        // settings info paragraphs
        "settings.info.p1" => "Welcome to amp.",
        "settings.info.p2" => {
            "We take no responsibility for what the city of Malmö chooses to do, this is a tool, nothing more."
        }
        "settings.info.p3" => {
            "The app takes data Malmö publishes, formats it better and uses a correlation algorithm to create a database which is then used here so that you as a user hopefully get fewer fines and Malmö can clean its streets without problems. Nothing more, nothing less."
        }
        "settings.info.p4" => {
            "Handling of days 29 and 30 in February is unclear as Malmö's system declares data with a day of month between 1 and 30 per data point. This means among other things that no cleaning should happen according to them on the 31st in months with that date. They say nothing about how the month of February is handled in either regular years or leap years. The relevant addresses for February are currently ignored and end up in the next month instead. Is this correct? No idea!"
        }
        "settings.info.p5" => {
            "The goal is to not require any internet connection in the app but currently due to the UI system used the app will crash without network permissions. All complex correlation happens on a server that sends a universal app update when Malmö's data is updated."
        }
        _ => "",
    }
}
