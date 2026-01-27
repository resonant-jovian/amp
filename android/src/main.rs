use dioxus::prelude::*;

pub mod components;
pub mod countdown;
pub mod matching;
pub mod static_data;
pub mod ui;

use ui::App;

fn main() {
    launch(App);
}
