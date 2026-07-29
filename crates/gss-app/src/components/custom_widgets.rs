use iced::{Alignment, alignment::Horizontal, Color, Theme, Renderer};
use iced::widget::{
    Container, Text, button, container, row, column, table, text, Image
};
use iced::widget::table::Table;
use iced::{Element, Length, alignment, Alignment::Center};
use iced_aw::{iced_aw_font};

use structs::internal::data::SaleInfo;

use crate::Message;
use crate::components::custom_styles::{bold_text, dialog_style};
use crate::utils::pricing_utils::SaleInfoWithHandler;

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

fn store_link<'a>(label: &'a str, id: &'a str, url: String,) -> iced::Element<'a, Message> {
    text::Rich::<String, Message, Theme, Renderer>::with_spans([
        text::Span::new(label)
            .link(&url)
    ])
    .on_link_click(move |_| Message::CopyLinkToClipboard(String::from(id), url.clone()))
    .size(16)
    .into()
}

// Badges

fn discount_badge(value: String) -> iced::Element<'static ,Message> {
    container(
        text(value)
            .size(14)
            .color(Color::WHITE)
    )
    .padding([6, 12])
    .style(|_| container::Style {
        background: Some(Color::from_rgb8(76, 175, 80).into()), // #4CAF50
        border: iced::Border {
            radius: 20.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .padding(10)
    .into()
}

// Text 

fn price_change<'a>(old_price: &'a str, new_price: &'a str) -> Element<'a, Message> {
    row![
        text::Rich::with_spans([
            text::Span::<()>::new(old_price)
                .size(16)
                .color(Color::from_rgb8(154, 163, 173))
                .strikethrough(true)
                .padding(10)
        ]),
        text(new_price)
            .size(24)
            .color(Color::from_rgb8(142, 245, 142))
    ].align_y(Alignment::Center)
    .spacing(10)
    .padding(10)
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

// Containers

pub fn game_store_card(copied_link: Option<String>, data: &SaleInfoWithHandler) -> Container<'_, Message> {
    let discount: String = format!("{}% OFF", &data.sale_info.discount_percentage);
    container(
        row![
            container(Image::new(&data.icon_handler)
                .height(100)).align_y(Alignment::Center)
                .padding(10),
            column![
                bold_text(&data.sale_info.title).size(24),
                row![
                    price_change(&data.sale_info.original_price, &data.sale_info.current_price),
                    discount_badge(discount),
                ].align_y(Alignment::Center),
                store_link(
                    if copied_link == Some(data.game_id.clone()) {
                        "✔ Copied to Clipboard"
                    } else {
                        "Copy Store Page Link"
                    },
                    &data.game_id, 
                    data.sale_info.store_page_link.clone())
            ]
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .padding(10)
    .style(|_| { 
        container::Style::from(Color::from_rgb8(45, 76, 105))
    })
}

pub fn message_dialog<'a>(title: &'a str, body: &'a str, message: Message) -> Container<'a, Message>{
    container(
        column![
            text(title).size(24),
            text(body).size(16).width(Length::Fill),
            column![
                button("OK")
                    .padding([8, 24])
                    .on_press(message),
            ]
            .width(Length::Fill)
            .align_x(Horizontal::Right),
        ]
        .spacing(24)
        .padding(24),
    )
    .width(360)
    .max_width(360)
    .style(dialog_style)
}

// Simple Animations

pub fn text_loading_indicator<'a>(description: &str, current_frame: usize, frame_size: usize) -> Text<'a, Theme> {
    text(
        format!("{}{}", 
        description,
        ".".repeat((current_frame % frame_size) + 1)
    )).size(16)
}