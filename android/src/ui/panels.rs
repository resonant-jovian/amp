//! Time-categorized display panels for parking restrictions.
//!
//! This module provides Dioxus components that organize and display addresses
//! into time-based categories with live countdown timers.
//!
//! # Overview
//!
//! Addresses are automatically sorted into panels based on when their parking
//! restrictions become active:
//!
//! | Panel | Time Range | Update Interval | Priority |
//! |-------|------------|-----------------|----------|
//! | **Active** | Currently active | 1 second | 🔴 Urgent |
//! | **6 Hours** | Active within 6h | 1 second | 🟠 Warning |
//! | **1 Day** | Active within 24h | 1 second | 🟡 Caution |
//! | **1 Month** | Active within 30d | 1 minute | 🟢 Planning |
//! | **30+ Days** | Active beyond 30d | 1 minute | 🔵 Info |
//! | **Invalid** | No parking data | Static | ⚪ Disabled |
//!
//! # Architecture
//!
//! ```text
//! App
//!  └─ categories-section
//!      ├─ ActivePanel
//!      │   └─ AddressItem (countdown: 1s)
//!      ├─ SixHoursPanel
//!      │   └─ AddressItem (countdown: 1s)
//!      ├─ OneDayPanel
//!      │   └─ AddressItem (countdown: 1s)
//!      ├─ OneMonthPanel
//!      │   └─ AddressItem (countdown: 1m)
//!      ├─ MoreThan1MonthPanel
//!      │   └─ AddressItem (countdown: 1m)
//!      └─ InvalidPanel
//!          └─ AddressItem (static)
//! ```
//!
//! # Time Bucketing
//!
//! Uses [`TimeBucket`] from the countdown module to categorize addresses:
//!
//! ```rust,ignore
//! pub enum TimeBucket {
//!     Now,              // Active right now
//!     Within6Hours,     // 0-6 hours away
//!     Within1Day,       // 6-24 hours away
//!     Within1Month,     // 1-30 days away
//!     MoreThan1Month,   // >30 days away
//!     Invalid,          // No parking data
//! }
//! ```
//!
//! # Real-time Updates
//!
//! Each [`AddressItem`] spawns an async task that:
//! 1. Calculates initial countdown
//! 2. Determines update interval based on urgency
//! 3. Updates countdown in loop until unmounted
//!
//! **Update intervals:**
//! - Urgent (Now, 6h, 1d): 1 second
//! - Planning (1m, 30d+): 1 minute
//! - Invalid: No updates
//!
//! # Sorting
//!
//! Within each panel, addresses are sorted by:
//! - **Time panels**: Earliest restriction first
//! - **Invalid panel**: Postal code order
//!
//! See [`sorting_time`] for implementation.
//!
//! # Examples
//!
//! ## Using Panels in App
//!
//! ```rust,ignore
//! use amp_android::ui::panels::*;
//!
//! rsx! {
//!     div { class: "categories-section",
//!         ActivePanel { addresses: addresses.clone() }
//!         SixHoursPanel { addresses: addresses.clone() }
//!         OneDayPanel { addresses: addresses.clone() }
//!         OneMonthPanel { addresses: addresses.clone() }
//!         MoreThan1MonthPanel { addresses: addresses.clone() }
//!         InvalidPanel { addresses: addresses.clone() }
//!     }
//! }
//! ```
//!
//! ## Custom Filtering
//!
//! ```rust,ignore
//! use amp_android::ui::StoredAddress;
//! use amp_android::components::countdown::{bucket_for, TimeBucket};
//!
//! let active_now: Vec<StoredAddress> = addresses
//!     .into_iter()
//!     .filter(|a| a.valid && a.active)
//!     .filter(|a| {
//!         a.matched_entry
//!             .as_ref()
//!             .map(|e| matches!(bucket_for(e), TimeBucket::Now))
//!             .unwrap_or(false)
//!     })
//!     .collect();
//! ```
//!
//! # Performance
//!
//! - **Initial render**: O(n) - filters and sorts all addresses
//! - **Per-address update**: 1-60 seconds depending on panel
//! - **Memory**: ~100 bytes per AddressItem for countdown task
//!
//! With 100 addresses across 6 panels:
//! - Render time: ~50-100ms
//! - Update overhead: ~10KB memory for async tasks
//!
//! # Styling
//!
//! Panels use CSS classes from `assets/style.css`:
//! - `.category-container`: Panel wrapper
//! - `.category-active`: Active panel (red theme)
//! - `.category-6h`: 6-hour panel (orange theme)
//! - `.category-24h`: 1-day panel (yellow theme)
//! - `.category-month`: 1-month panel (green theme)
//! - `.category-later`: 30+ day panel (blue theme)
//! - `.category-invalid`: Invalid panel (gray theme)
//! - `.address-item`: Individual address display
//! - `.countdown-text`: Countdown timer text
//!
//! # See Also
//!
//! - [`crate::components::countdown`]: Countdown calculation logic
//! - [`crate::ui::StoredAddress`]: Address data structure
//! - [`crate::ui::App`]: Root component using panels
use crate::components::countdown::{
    TimeBucket, bucket_for, format_countdown, time_until_next_occurrence, time_until_next_start,
};
use crate::ui::StoredAddress;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_navigation_icons::{MdExpandLess, MdExpandMore};
use tokio::time::Duration;
/// Display an address with countdown timer in appropriate category.
///
/// This component:
/// - Shows address in format "Street Number, PostalCode"
/// - Displays live countdown to parking restriction
/// - Updates at interval based on time bucket (1s or 1m)
/// - Stops updating when unmounted
///
/// # Props
/// * `addr` - StoredAddress to display
/// * `index` - Position in list (for React-style keying)
/// * `on_remove` - Event handler for remove button (currently unused)
///
/// # Countdown Updates
///
/// The component spawns an async task that:
/// 1. Calculates initial countdown using [`format_countdown`]
/// 2. Determines update interval:
///    - **1 second**: For Now, 6h, 1d buckets (urgent)
///    - **1 minute**: For 1m, 30d+ buckets (planning)
/// 3. Updates countdown in loop using Tokio sleep
/// 4. Automatically stops when component unmounts
///
/// # Examples
///
/// ```rust,ignore
/// use amp_android::ui::StoredAddress;
///
/// rsx! {
///     AddressItem {
///         addr: address.clone(),
///         index: 0,
///         on_remove: move |_| { /* handle remove */ },
///     }
/// }
/// ```
#[component]
fn AddressItem(addr: StoredAddress, index: usize, on_remove: EventHandler<usize>) -> Element {
    let mut countdown = use_signal(|| "...".to_string());
    let addr_clone = addr.clone();
    use_future(move || {
        let addr_for_future = addr_clone.clone();
        async move {
            let bucket = addr_for_future
                .matched_entry
                .as_ref()
                .map(bucket_for)
                .unwrap_or(TimeBucket::Invalid);
            if let Some(matched) = &addr_for_future.matched_entry {
                countdown.set(format_countdown(matched).unwrap_or_else(|| "...".to_string()));
            }
            if bucket == TimeBucket::Invalid {
                return;
            }
            let update_interval = match bucket {
                TimeBucket::Now | TimeBucket::Within6Hours | TimeBucket::Within1Day => {
                    Duration::from_secs(1)
                }
                _ => Duration::from_secs(60),
            };
            loop {
                tokio::time::sleep(update_interval).await;
                let new_countdown = addr_for_future
                    .matched_entry
                    .as_ref()
                    .and_then(format_countdown)
                    .unwrap_or_else(|| "...".to_string());
                countdown.set(new_countdown);
            }
        }
    });
    let address_display = addr.display_name();
    rsx! {
        div { class: "address-item",
            div { class: "address-text", "{address_display}" }
            div { class: "countdown-text", "{countdown()}" }
        }
    }
}
/// Sort addresses by time until next restriction occurrence.
///
/// Addresses with earlier restrictions are sorted first. This function uses
/// [`time_until_next_occurrence`] which handles both current month and next
/// month occurrences, ensuring proper sorting even for expired restrictions.
///
/// # Arguments
/// * `active_addrs` - Vector of addresses to sort (consumed)
///
/// # Returns
/// Sorted vector with earliest restrictions first
///
/// # Algorithm
/// Uses [`time_until_next_occurrence`] to calculate time until next occurrence, then:
/// - Compares durations (shorter = earlier in list)
/// - Addresses with data come before those without
/// - Addresses without data maintain relative order
///
/// # Complexity
/// O(n log n) where n is number of addresses
///
/// # Examples
///
/// ```rust,ignore
/// use amp_android::ui::panels::sorting_time;
///
/// let addresses = vec![addr_in_2h, addr_in_1h, addr_in_5h];
/// let sorted = sorting_time(addresses);
/// // Result: [addr_in_1h, addr_in_2h, addr_in_5h]
/// ```
pub fn sorting_time(mut active_addrs: Vec<StoredAddress>) -> Vec<StoredAddress> {
    active_addrs.sort_by(|a, b| {
        let time_a = a
            .matched_entry
            .as_ref()
            .and_then(time_until_next_occurrence);
        let time_b = b
            .matched_entry
            .as_ref()
            .and_then(time_until_next_occurrence);
        match (time_a, time_b) {
            (Some(dur_a), Some(dur_b)) => dur_a.cmp(&dur_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    active_addrs
}
/// Sort addresses by time until next restriction start (for non-Active panels).
///
/// Like [`sorting_time`] but uses start time instead of end time,
/// so upcoming restrictions are sorted by when they begin.
pub fn sorting_time_by_start(mut addrs: Vec<StoredAddress>) -> Vec<StoredAddress> {
    addrs.sort_by(|a, b| {
        let time_a = a.matched_entry.as_ref().and_then(time_until_next_start);
        let time_b = b.matched_entry.as_ref().and_then(time_until_next_start);
        match (time_a, time_b) {
            (Some(dur_a), Some(dur_b)) => dur_a.cmp(&dur_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    addrs
}
/// Panel displaying addresses with parking restrictions currently active.
///
/// Shows addresses where the parking restriction is happening **right now**.
/// This is the highest priority panel, typically styled in red/urgent colors.
///
/// # Filtering
/// Displays addresses where:
/// - `valid == true` (matched in database)
/// - `active == true` (user enabled)
/// - Time bucket is [`TimeBucket::Now`]
///
/// # Sorting
/// Addresses sorted by time using [`sorting_time`] (earliest first).
///
/// # Update Frequency
/// Countdown updates every **1 second** for real-time accuracy.
///
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (automatically filtered)
///
/// # Examples
///
/// ```rust,ignore
/// use amp_android::ui::panels::ActivePanel;
///
/// rsx! {
///     ActivePanel { addresses: all_addresses.clone() }
/// }
/// ```
#[component]
pub fn ActivePanel(addresses: Vec<StoredAddress>) -> Element {
    let mut active_addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::Now)
            } else {
                false
            }
        })
        .collect();
    active_addrs = sorting_time(active_addrs);
    let active_count = active_addrs.len();
    let mut is_open = use_signal(|| false);
    rsx! {
        div { class: "category-container category-active",
            button {
                class: "category-title",
                onclick: move |_| is_open.set(!is_open()),
                "aria-expanded": if is_open() { "true" } else { "false" },
                span { "Städas nu" }
                span { class: "category-toggle-arrow",
                    if is_open() {
                        Icon { icon: MdExpandLess, width: 16, height: 16 }
                    } else {
                        Icon { icon: MdExpandMore, width: 16, height: 16 }
                    }
                }
            }
            div {
                class: "category-content",
                id: "categoryActive",
                "aria-hidden": if is_open() { "false" } else { "true" },
                if active_count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            active_addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
/// Panel displaying addresses with restrictions becoming active within 6 hours.
///
/// Shows addresses requiring attention soon, typically styled in orange/warning colors.
///
/// # Filtering
/// Displays addresses where:
/// - `valid == true` (matched in database)
/// - `active == true` (user enabled)
/// - Time bucket is [`TimeBucket::Within6Hours`]
///
/// # Sorting
/// Addresses sorted by time using [`sorting_time`] (earliest first).
///
/// # Update Frequency
/// Countdown updates every **1 second**.
///
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (automatically filtered)
///
/// # Examples
///
/// ```rust,ignore
/// use amp_android::ui::panels::SixHoursPanel;
///
/// rsx! {
///     SixHoursPanel { addresses: all_addresses.clone() }
/// }
/// ```
#[component]
pub fn SixHoursPanel(addresses: Vec<StoredAddress>) -> Element {
    let mut addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::Within6Hours)
            } else {
                false
            }
        })
        .collect();
    addrs = sorting_time_by_start(addrs);
    let count = addrs.len();
    let mut is_open = use_signal(|| false);
    rsx! {
        div { class: "category-container category-6h",
            button {
                class: "category-title",
                onclick: move |_| is_open.set(!is_open()),
                "aria-expanded": if is_open() { "true" } else { "false" },
                span { "Inom 6 timmar" }
                span { class: "category-toggle-arrow",
                    if is_open() {
                        Icon { icon: MdExpandLess, width: 16, height: 16 }
                    } else {
                        Icon { icon: MdExpandMore, width: 16, height: 16 }
                    }
                }
            }
            div {
                class: "category-content",
                id: "category6h",
                "aria-hidden": if is_open() { "false" } else { "true" },
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
/// Panel displaying addresses with restrictions becoming active within 24 hours.
///
/// Shows addresses requiring attention today, typically styled in yellow/caution colors.
///
/// # Filtering
/// Displays addresses where:
/// - `valid == true` (matched in database)
/// - `active == true` (user enabled)
/// - Time bucket is [`TimeBucket::Within1Day`]
///
/// # Sorting
/// Addresses sorted by time using [`sorting_time`] (earliest first).
///
/// # Update Frequency
/// Countdown updates every **1 second**.
///
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (automatically filtered)
///
/// # Examples
///
/// ```rust,ignore
/// use amp_android::ui::panels::OneDayPanel;
///
/// rsx! {
///     OneDayPanel { addresses: all_addresses.clone() }
/// }
/// ```
#[component]
pub fn OneDayPanel(addresses: Vec<StoredAddress>) -> Element {
    let mut addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::Within1Day)
            } else {
                false
            }
        })
        .collect();
    addrs = sorting_time_by_start(addrs);
    let count = addrs.len();
    let mut is_open = use_signal(|| false);
    rsx! {
        div { class: "category-container category-24h",
            button {
                class: "category-title",
                onclick: move |_| is_open.set(!is_open()),
                "aria-expanded": if is_open() { "true" } else { "false" },
                span { "Inom 1 dag" }
                span { class: "category-toggle-arrow",
                    if is_open() {
                        Icon { icon: MdExpandLess, width: 16, height: 16 }
                    } else {
                        Icon { icon: MdExpandMore, width: 16, height: 16 }
                    }
                }
            }
            div {
                class: "category-content",
                id: "category24h",
                "aria-hidden": if is_open() { "false" } else { "true" },
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
/// Panel displaying addresses with restrictions becoming active within 1 month (30 days).
///
/// Shows addresses for planning ahead, typically styled in green colors.
///
/// # Filtering
/// Displays addresses where:
/// - `valid == true` (matched in database)
/// - `active == true` (user enabled)
/// - Time bucket is [`TimeBucket::Within1Month`]
///
/// # Sorting
/// Addresses sorted by time using [`sorting_time`] (earliest first).
///
/// # Update Frequency
/// Countdown updates every **1 minute** (less urgent).
///
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (automatically filtered)
///
/// # Examples
///
/// ```rust,ignore
/// use amp_android::ui::panels::OneMonthPanel;
///
/// rsx! {
///     OneMonthPanel { addresses: all_addresses.clone() }
/// }
/// ```
#[component]
pub fn OneMonthPanel(addresses: Vec<StoredAddress>) -> Element {
    let mut addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::Within1Month)
            } else {
                false
            }
        })
        .collect();
    addrs = sorting_time_by_start(addrs);
    let count = addrs.len();
    let mut is_open = use_signal(|| false);
    rsx! {
        div { class: "category-container category-month",
            button {
                class: "category-title",
                onclick: move |_| is_open.set(!is_open()),
                "aria-expanded": if is_open() { "true" } else { "false" },
                span { "Inom 1 månad" }
                span { class: "category-toggle-arrow",
                    if is_open() {
                        Icon { icon: MdExpandLess, width: 16, height: 16 }
                    } else {
                        Icon { icon: MdExpandMore, width: 16, height: 16 }
                    }
                }
            }
            div {
                class: "category-content",
                id: "categoryMonth",
                "aria-hidden": if is_open() { "false" } else { "true" },
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
/// Panel displaying addresses with restrictions more than 1 month away (>30 days).
///
/// Shows addresses for long-term planning, typically styled in blue/info colors.
///
/// # Filtering
/// Displays addresses where:
/// - `valid == true` (matched in database)
/// - `active == true` (user enabled)
/// - Time bucket is [`TimeBucket::MoreThan1Month`]
///
/// # Sorting
/// Addresses sorted by time using [`sorting_time`] (earliest first).
///
/// # Update Frequency
/// Countdown updates every **1 minute** (low urgency).
///
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (automatically filtered)
///
/// # Examples
///
/// ```rust,ignore
/// use amp_android::ui::panels::MoreThan1MonthPanel;
///
/// rsx! {
///     MoreThan1MonthPanel { addresses: all_addresses.clone() }
/// }
/// ```
#[component]
pub fn MoreThan1MonthPanel(addresses: Vec<StoredAddress>) -> Element {
    let mut addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active)
        .filter(|a| {
            if let Some(entry) = &a.matched_entry {
                matches!(bucket_for(entry), TimeBucket::MoreThan1Month)
            } else {
                false
            }
        })
        .collect();
    addrs = sorting_time_by_start(addrs);
    let count = addrs.len();
    let mut is_open = use_signal(|| false);
    rsx! {
        div { class: "category-container category-later",
            button {
                class: "category-title",
                onclick: move |_| is_open.set(!is_open()),
                "aria-expanded": if is_open() { "true" } else { "false" },
                span { "30+ dagar" }
                span { class: "category-toggle-arrow",
                    if is_open() {
                        Icon { icon: MdExpandLess, width: 16, height: 16 }
                    } else {
                        Icon { icon: MdExpandMore, width: 16, height: 16 }
                    }
                }
            }
            div {
                class: "category-content",
                id: "category-later",
                "aria-hidden": if is_open() { "false" } else { "true" },
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
/// Panel displaying addresses with only parking zone data (no street cleaning schedule).
///
/// Shows addresses that have parking fee/zone information (taxa, platser, typ)
/// but no time-based restrictions (dag, tid). These addresses are valid but have
/// no countdown timer since there's no cleaning schedule.
///
/// Styled in brown/sienna colors to represent parking fees.
///
/// # Filtering
/// Displays addresses where:
/// - `valid == true`
/// - `active == true`
/// - `matched_entry` is None (no miljödata)
/// - `parking_info` is Some (has parking zone data)
///
/// # Sorting
/// Addresses sorted by **postal code** (no time-based sorting).
///
/// # Update Frequency
/// **Static** - no countdown updates.
///
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (automatically filtered)
#[component]
pub fn ParkingOnlyPanel(addresses: Vec<StoredAddress>) -> Element {
    let mut addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.valid && a.active && a.matched_entry.is_none() && a.parking_info.is_some())
        .collect();
    addrs.sort_by(
        |a, b| match (a.postal_code.is_empty(), b.postal_code.is_empty()) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.postal_code.cmp(&b.postal_code),
        },
    );
    let count = addrs.len();
    let mut is_open = use_signal(|| false);
    rsx! {
        div { class: "category-container category-parking-only",
            button {
                class: "category-title",
                onclick: move |_| is_open.set(!is_open()),
                "aria-expanded": if is_open() { "true" } else { "false" },
                span { "Endast parkeringsavgift" }
                span { class: "category-toggle-arrow",
                    if is_open() {
                        Icon { icon: MdExpandLess, width: 16, height: 16 }
                    } else {
                        Icon { icon: MdExpandMore, width: 16, height: 16 }
                    }
                }
            }
            div {
                class: "category-content",
                id: "categoryParkingOnly",
                "aria-hidden": if is_open() { "false" } else { "true" },
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
/// Panel displaying addresses with no valid parking restriction data.
///
/// Shows addresses that:
/// - Failed validation (not found in database)
/// - Have validation errors (Feb 30, etc.)
/// - Are in the database but currently invalid
///
/// Typically styled in gray/disabled colors.
///
/// # Filtering
/// Displays addresses where:
/// - `active == true` (user enabled)
/// - `valid == false` (no database match or invalid)
///
/// # Sorting
/// Addresses sorted by **postal code** (not by time).
///
/// # Update Frequency
/// **Static** - no countdown updates.
///
/// # Props
/// * `addresses` - Vector of all StoredAddress entries (automatically filtered)
///
/// # Examples
///
/// ```rust,ignore
/// use amp_android::ui::panels::InvalidPanel;
///
/// rsx! {
///     InvalidPanel { addresses: all_addresses.clone() }
/// }
/// ```
#[component]
pub fn InvalidPanel(addresses: Vec<StoredAddress>) -> Element {
    let mut addrs: Vec<_> = addresses
        .into_iter()
        .filter(|a| a.active && !a.valid)
        .collect();
    addrs.sort_by(
        |a, b| match (a.postal_code.is_empty(), b.postal_code.is_empty()) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.postal_code.cmp(&b.postal_code),
        },
    );
    let count = addrs.len();
    let mut is_open = use_signal(|| false);
    rsx! {
        div { class: "category-container category-invalid",
            button {
                class: "category-title",
                onclick: move |_| is_open.set(!is_open()),
                "aria-expanded": if is_open() { "true" } else { "false" },
                span { "Ingen städning" }
                span { class: "category-toggle-arrow",
                    if is_open() {
                        Icon { icon: MdExpandLess, width: 16, height: 16 }
                    } else {
                        Icon { icon: MdExpandMore, width: 16, height: 16 }
                    }
                }
            }
            div {
                class: "category-content",
                id: "categoryInvalid",
                "aria-hidden": if is_open() { "false" } else { "true" },
                if count == 0 {
                    div { class: "empty-state", "Inga adresser" }
                } else {
                    div { class: "address-list",
                        {
                            addrs
                                .into_iter()
                                .enumerate()
                                .map(|(i, addr)| {
                                    rsx! {
                                        AddressItem {
                                            key: "{i}",
                                            addr: addr.clone(),
                                            index: i,
                                            on_remove: move |_| {},
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
