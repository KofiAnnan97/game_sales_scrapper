
use iced::widget::{
    button, container, row, text
};
use iced::{Element, Length, alignment};
use iced_aw::{iced_aw_font};

use crate::Message;

pub fn submenu_button<'a>(label: &'a str) -> Element<'a, Message> {
    row![
        text(label)
            .width(Length::Fill)
            .align_y(iced::Alignment::Center),
        iced_aw_font::right_open()
            .width(Length::Shrink)
            .align_y(iced::Alignment::Center),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn menu_text_button<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
    button(
        container(text(label))
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left),
    )
    .on_press(message)
    .style(button::text)
    .width(Length::Fill)
    .padding(0)
    .into()
}