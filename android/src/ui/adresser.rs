use crate::matching::{MatchResult, match_address, validate_input};
use crate::static_data::StaticAddressEntry;
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Default)]
pub struct AddressInputState {
    pub gata: String,
    pub gatunummer: String,
    pub postnummer: String,
}


#[derive(Clone, Debug)]
pub enum ValidationStatus {
    None,
    Valid(StaticAddressEntry),
    Invalid(String),
}

#[component]
pub fn Adresser(on_add_valid_address: EventHandler<StaticAddressEntry>) -> Element {
    let mut input = use_signal(AddressInputState::default);
    let mut validation_status = use_signal(|| ValidationStatus::None);

    let handle_add = move |_| {
        let current = input.read().clone();

        // Validate input fields are not empty
        if !validate_input(&current.gata, &current.gatunummer, &current.postnummer) {
            validation_status.set(ValidationStatus::Invalid(
                "Alla fält måste fyllas i".to_string(),
            ));
            return;
        }

        // Check against static correlations
        let result = match_address(&current.gata, &current.gatunummer, &current.postnummer);

        match result {
            MatchResult::Valid(entry) => {
                validation_status.set(ValidationStatus::Valid(entry.clone()));
                // Call parent handler with validated address
                on_add_valid_address.call(entry);
                // Reset form
                input.set(AddressInputState::default());
            }
            MatchResult::Invalid => {
                validation_status.set(ValidationStatus::Invalid(
                    "Adressen finns inte i systemet".to_string(),
                ));
            }
        }
    };

    let status_display = match validation_status() {
        ValidationStatus::None => rsx! { "" },
        ValidationStatus::Valid(entry) => rsx! {
            div { class: "validation-message success",
                p { "✓ Adress hittad!" }
                p { "Dag: {entry.dag}, Tid: {entry.tid}" }
            }
        },
        ValidationStatus::Invalid(msg) => rsx! {
            div { class: "validation-message error",
                p { "✗ {msg}" }
            }
        },
    };

    rsx! {
        div { class: "stored-addresses",
            h2 { "Adresser" }

            div { class: "input-section",
                div { class: "input-group",
                    input {
                        class: "address-input",
                        placeholder: "Gata",
                        value: "{input.read().gata}",
                        onchange: move |evt: Event<FormData>| {
                            input.write().gata = evt.value();
                        },
                    }
                    input {
                        class: "address-input",
                        placeholder: "Gatunummer",
                        value: "{input.read().gatunummer}",
                        onchange: move |evt: Event<FormData>| {
                            input.write().gatunummer = evt.value();
                        },
                    }
                    input {
                        class: "address-input",
                        placeholder: "Postnummer",
                        value: "{input.read().postnummer}",
                        onchange: move |evt: Event<FormData>| {
                            input.write().postnummer = evt.value();
                        },
                    }
                    button {
                        class: "add-button",
                        onclick: handle_add,
                        "+"
                    }
                }

                // Validation feedback
                {status_display}
            }

            div { id: "addressList" }
        }
    }
}
