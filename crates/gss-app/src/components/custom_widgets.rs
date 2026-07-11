use iced::Theme;
use iced::widget::{
    Text, button, container, row, table, text
};
use iced::widget::table::Table;
use iced::{Element, Length, alignment, Alignment::Center};
use iced_aw::{iced_aw_font};

use structs::internal::data::SaleInfo;

use crate::Message;
use crate::components::custom_styles::bold_text;

// Butttons

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

// Tables

pub fn create_sales_table<'a>(sales_info: &'a Vec<SaleInfo>) -> Table<'a, Message>{
    let columns = [
        table::column(bold_text("Title"), |game: &SaleInfo| text(&game.title))
            .width(Length::FillPortion(2)),
        table::column(bold_text("MSRP"), |game: &SaleInfo| text(&game.original_price).align_x(Center))
            .width(Length::FillPortion(1)),
        table::column(bold_text("Current Price"), |game: &SaleInfo| text(&game.current_price).align_x(Center))
            .width(Length::FillPortion(1)),
        table::column(bold_text("Discount %"), |game: &SaleInfo| text(&game.discount_percentage).align_x(Center))
            .width(Length::FillPortion(1)),
    ];

    table(columns, sales_info)
        .separator_x(2.0)
        .separator_y(1.0)
        .padding_x(10.0)
        .padding_y(10.0)
}

// Simple Animations
pub fn text_loading_indicator<'a>(description: &str, current_frame: usize, frame_size: usize) -> Text<'a, Theme> {
    text(
        format!("{}{}", 
        description,
        ".".repeat((current_frame % frame_size) + 1)
    )).size(16)
}