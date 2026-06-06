use iced::widget::{Button, Scrollable, column, row, text};
use iced::{Element, Length};

use crate::Message;

pub fn view_actions(app: &crate::App) -> Element<'_, Message> {
    let log_body = Scrollable::new(text(&app.log).size(30))
        .height(Length::Fill);

    column![
        row![
            Button::new(text("Check prices"))
                .on_press(Message::CheckPrices)
                .padding(10),
            Button::new(text("Send Test Email"))
                .on_press(Message::SendEmail)
                .padding(10),
            Button::new(text("Update Game Cache"))
                .on_press(Message::UpdateCache)
                .padding(10),
        ]
        .spacing(10),
        text(format!("Status: {}", app.status_message)),
        log_body,
    ]
    .spacing(15)
    .padding(10)
    .into()
}