use iced::Element;
use iced::Element;
use iced::widget::text_input;

#[derive(Default)]
struct State {
    content: String,
}

#[derive(Debug, Clone)]
enum Message {
    ContentChanged(String),
}

pub fn main() -> iced::Result {
    iced::run("amp", update, view)
}

fn view(state: &State) -> Element<'_, Message> {
    text_input("Lägg till adress...", &state.content)
        .on_input(Message::ContentChanged)
        .into()
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::ContentChanged(content) => {
            state.content = content;
        }
    }
}
