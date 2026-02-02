pub mod android_bridge;
pub mod components;
pub mod countdown;
pub mod geo;
pub mod matching;
pub mod notifications;
pub mod static_data;
pub mod storage;
pub mod ui;
use ui::App;
fn main() {
    dioxus::launch(App);
}
