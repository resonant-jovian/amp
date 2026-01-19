pub mod adresser;
pub mod paneler;
pub mod topbar;

use crate::ui::{
    adresser::Adresser,
    paneler::{Active, Day, Month, NotValid, Six},
    topbar::TopBar,
};

use dioxus::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

static CSS: Asset = asset!("/assets/style.css");

static ADDRESS_DATA: Asset = asset!("/assets/data/adress_info.parquet");


#[derive(Clone, Debug, PartialEq)]
struct Address {
    street: String,
    postal: String,
}

#[derive(Clone, Debug, PartialEq)]
enum ScheduleType {
    Now,
    SixHours,
    TwentyFourHours,
    Month,
    None,
}

#[derive(Clone, Debug, PartialEq)]
struct Schedule {
    kind: ScheduleType,
    deadline: Option<SystemTime>,
    active: bool,
}

fn schedule_key(addr: &Address) -> String {
    format!("{}-{}", addr.street, addr.postal)
}

fn format_postal_code(postal: &str) -> String {
    let digits: String = postal.chars().filter(|c| c.is_ascii_digit()).collect();
    let mut padded = digits;
    padded.push_str("00000");
    let padded = &padded[..5];
    format!("{} {}", &padded[0..3], &padded[3..5])
}

fn compute_deadline(kind: &ScheduleType) -> Option<SystemTime> {
    let now = SystemTime::now();
    match kind {
        ScheduleType::None => None,
        ScheduleType::Now => Some(now),
        ScheduleType::SixHours => Some(now + Duration::from_secs(6 * 60 * 60)),
        ScheduleType::TwentyFourHours => Some(now + Duration::from_secs(24 * 60 * 60)),
        ScheduleType::Month => Some(now + Duration::from_secs(30 * 24 * 60 * 60)),
    }
}

fn get_time_until(deadline: SystemTime) -> Option<Duration> {
    deadline.duration_since(SystemTime::now()).ok()
}

fn format_time(ms: u128) -> String {
    let total_seconds = (ms / 1000) as u64;
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    format!(
        "{:02}:{:02}:{:02}:{:02}",
        days, hours, minutes, seconds
    )
}

#[component]
pub fn App() -> Element {
    let mut addresses = use_signal::<Vec<Address>>(|| vec![]);
    let mut schedules = use_signal::<HashMap<String, Schedule>>(|| HashMap::new());

    // initializeSampleData equivalent (runs once)
    use_effect(move || {
        if !addresses.read().is_empty() {
            return;
        }

        let sample = vec![
            Address {
                street: "Storgatan 10".to_string(),
                postal: "22100".to_string(),
            },
            Address {
                street: "Kungsgatan 5".to_string(),
                postal: "22200".to_string(),
            },
            Address {
                street: "Järnvägsgatan 15".to_string(),
                postal: "22300".to_string(),
            },
            Address {
                street: "Södergatan 20".to_string(),
                postal: "22400".to_string(),
            },
            Address {
                street: "Västra vägen 8".to_string(),
                postal: "22500".to_string(),
            },
        ];

        let sched_types = vec![
            ScheduleType::Now,
            ScheduleType::SixHours,
            ScheduleType::TwentyFourHours,
            ScheduleType::Month,
            ScheduleType::None,
        ];

        addresses.set(sample.clone());

        let mut map = HashMap::new();
        for (addr, kind) in sample.into_iter().zip(sched_types.into_iter()) {
            let key = schedule_key(&addr);
            let deadline = compute_deadline(&kind);
            map.insert(
                key,
                Schedule {
                    kind,
                    deadline,
                    active: true,
                },
            );
        }

        schedules.set(map);
    });

    // addAddressManual equivalent (without DOM, using callbacks)
    let add_address_manual = {
        let mut addresses = addresses.to_owned();
        let mut schedules = schedules.to_owned();
        move |street: String, postal_input: String| {
            if street.trim().is_empty() || postal_input.trim().is_empty() {
                // in UI, show alert/toast instead
                return;
            }

            let postal = format_postal_code(&postal_input);
            let addr = Address {
                street: street.trim().to_string(),
                postal: postal.clone(),
            };
            let key = schedule_key(&addr);

            if schedules.read().contains_key(&key) {
                // address exists; show UI warning
                return;
            }

            addresses.write().push(addr.clone());
            schedules.write().insert(
                key,
                Schedule {
                    kind: ScheduleType::None,
                    deadline: None,
                    active: true,
                },
            );
        }
    };

    // removeAddress equivalent
    let remove_address = {
        let mut addresses = addresses.to_owned();
        let mut schedules = schedules.to_owned();
        move |index: usize| {
            let mut addrs = addresses.write();
            if index >= addrs.len() {
                return;
            }
            let addr = addrs.remove(index);
            let key = schedule_key(&addr);
            schedules.write().remove(&key);
        }
    };

    // toggleAddress equivalent
    let toggle_address = {
        let mut addresses = addresses.to_owned();
        let mut schedules = schedules.to_owned();
        move |index: usize| {
            let addrs = addresses.read();
            if index >= addrs.len() {
                return;
            }
            let addr = &addrs[index];
            let key = schedule_key(addr);
            if let Some(s) = schedules.write().get_mut(&key) {
                s.active = !s.active;
            }
        }
    };

    // updateSchedule equivalent
    let update_schedule = {
        let mut addresses = addresses.to_owned();
        let mut schedules = schedules.to_owned();
        move |index: usize, new_kind: ScheduleType| {
            let addrs = addresses.read();
            if index >= addrs.len() {
                return;
            }
            let addr = &addrs[index];
            let key = schedule_key(addr);
            if let Some(s) = schedules.write().get_mut(&key) {
                s.kind = new_kind.clone();
                s.deadline = compute_deadline(&new_kind);
            }
        }
    };

    // updateTimers equivalent: derive formatted time for each active scheduled address
    let timers: Vec<(Address, Option<String>)> = {
        let mut addrs = addresses.read();
        let mut scheds = schedules.read();
        addrs
            .iter()
            .filter_map(|addr| {
                let key = schedule_key(addr);
                let schedule = scheds.get(&key)?;
                if !schedule.active {
                    return None;
                }
                let formatted = schedule
                    .deadline
                    .and_then(get_time_until)
                    .map(|dur| format_time(dur.as_millis()));
                Some((addr.clone(), formatted))
            })
            .collect()
    };

    rsx! {
        Stylesheet { href: CSS }
        div {
            class: "app-wrapper",
            TopBar {  },
            div {
                class: "app-container",
                Adresser {  }
                div {
                    class: "categories-section",
                    Active {  },
                    Six {  },
                    Day {  },
                    Month {  },
                    NotValid {  },
                }
            }
        }
    }
}
