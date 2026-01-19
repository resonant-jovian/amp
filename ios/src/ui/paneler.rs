use dioxus::prelude::*;

#[component]
pub fn Active() -> Element {
    rsx! {

        div {
            class: "category-container category-active",
            div {
                class: "category-title", "Städas nu"
            }
            div {
                class: "category-content",
                div {
                    class: "category-addresses", id: "container-active"
                }
                div {
                    class: "category-timers", id: "timers-active"
                }
            }
        }
    }
}

#[component]
pub fn Six() -> Element {
    rsx! {

        div {
            class: "category-container category-6h",
            div {
                class: "category-title", "Inom 6 timmar"
            }
            div {
                class: "category-content",
                div {
                    class: "category-addresses", id: "container-6h"
                }
                div {
                    class: "category-timers", id: "timers-6h"
                }
            }
        }
    }
}

#[component]
pub fn Day() -> Element {
    rsx! {

        div {
            class: "category-container category-24h",
            div {
                class: "category-title", "Inom 1 dag"
            }
            div {
                class: "category-content",
                div {
                    class: "category-addresses", id: "container-24h"
                }
                div {
                    class: "category-timers", id: "timers-24h"
                }
            }
        }
    }
}

#[component]
pub fn Month() -> Element {
    rsx! {

        div {
            class: "category-container category-month",
            div {
                class: "category-title", "Inom 1 månad"
            }
            div {
                class: "category-content",
                div {
                    class: "category-addresses", id: "container-month"
                }
                div {
                    class: "category-timers", id: "timers-month"
                }
            }
        }
    }
}

#[component]
pub fn NotValid() -> Element {
    rsx! {

        div {
            class: "category-container category-invalid",
            div {
                class: "category-title", "Ingen städning"
            }
            div {
                class: "category-content",
                div {
                    class: "category-addresses", id: "container-none"
                }
                div {
                    class: "category-timers", id: "timers-none"
                }
            }
        }
    }
}
