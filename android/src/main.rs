use dioxus::prelude::*;

pub mod components;
pub mod ui;

use ui::App;

fn main() {
    launch(App);
}
