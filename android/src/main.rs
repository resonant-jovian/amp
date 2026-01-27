use dioxus::prelude::*;

pub mod components;
pub mod ui;
pub mod static_data;
pub mod matching;
pub mod countdown;

use ui::App;

fn main() {
    launch(App);
}
