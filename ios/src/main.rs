use dioxus::prelude::*;

pub mod ui;
pub mod components;

use ui::App;

fn main() {
    dioxus::launch(App);
}
