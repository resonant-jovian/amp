use dioxus::prelude::*;

pub mod ui;

use ui::App;

fn main() {
    dioxus::launch(App);
}
