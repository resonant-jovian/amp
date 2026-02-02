pub mod android_bridge;
pub mod components;
pub mod ui;

use ui::App;

fn main() {
    dioxus::launch(App);
}
