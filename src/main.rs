use iced::Element;
use iced::widget::{button, column, container, row, scrollable, text_input};

#[derive(Default)]
struct State {
    content: String,
}

#[derive(Debug, Clone)]
enum Message {
    ContentChanged(String),
    ButtonPressed,
}

pub fn main() -> iced::Result {
    iced::run("amp", update, view)
}

fn view(state: &State) -> Element<'_, Message> {
    column![
        container(row![
            text_input("Lägg till adress...", &state.content).on_input(Message::ContentChanged),
            button("+").on_press(Message::ButtonPressed),
        ])
        .padding(10)
        .style(container::rounded_box),
        container(scrollable(column![
            //Parse stored addresses from JSON
            "test123",
        ]))
        .padding(10)
        .style(container::rounded_box)
    ]
    .into()
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::ContentChanged(content) => {
            state.content = content;
        }
        Message::ButtonPressed => {
            let _content = state.content.clone(); //Add to JSON list and write
        }
    }
}
