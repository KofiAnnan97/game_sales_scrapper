use constants::operations::settings::{STEAM_STORE_ID, GOG_STORE_ID, MICROSOFT_STORE_ID};
use iced::widget::image::Handle;
use iced::{Alignment, alignment::Horizontal, Color, Theme, Renderer};
use iced::widget::{
    Container, Text, button, container, row, column, table, text, Image
};
use iced::widget::table::Table;
use iced::{Element, Length, alignment, Alignment::Center};
use iced_aw::{iced_aw_font};

use types::internal::data::SaleInfo;

use crate::Message;
use crate::components::custom_styles::{bold_text, cmp_row_style, dialog_style, normal_price_style, best_price_style};
use crate::utils::pricing_utils::{SaleInfoCompare, StoreSale};

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

fn colored_badge(value: String, font_size: u32, color: Color) -> iced::Element<'static ,Message> {
    container(
        text(value)
            .size(font_size)
            .color(Color::WHITE)
    )
    .padding([6, 12])
    .style(move |_| container::Style {
        background: Some(color.into()),
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

fn price_change<'a>(old_price: f64, new_price: f64) -> Element<'a, Message> {
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

pub fn game_store_card<'a>(copied_link: &'a Option<String>, data: &'a StoreSale) -> Container<'a, Message> {
    let discount: String = format!("{}% OFF", &data.info.discount_percentage);
    let handler = data.icon_handler.clone().unwrap_or_else(|| Handle::from_bytes(vec![]));
    container(
        row![
            container(Image::new(handler)
                .height(100)).align_y(Alignment::Center)
                .padding(10),
            column![
                bold_text(&data.info.title).size(24),
                row![
                    price_change(data.info.original_price, data.info.current_price),
                    colored_badge(discount, 14, Color::from_rgb8(76, 175, 80)),
                ].align_y(Alignment::Center),
                store_link(
                    if copied_link == &Some(data.game_id.clone()) {
                        "✔ Copied to Clipboard"
                    } else {
                        "Copy Store Page Link"
                    },
                    &data.game_id, 
                    data.info.store_page_link.clone())
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

pub fn store_price_cell<'a>(price: &'a Option<f64>, is_best: bool) -> Container<'a, Message> {
    let price_str;
    if let Some(price_val) = price {
        price_str = format!("{}", price_val);
    }else {
        price_str = String::from(" - ");
    }
    container(
        text(price_str.clone())
    )
    .center_x(Length::Fill)
    .padding(8)
    .style(if is_best {
        best_price_style
    } else {
        normal_price_style
    })
}

pub fn game_comparison_row<'a>(data: &'a SaleInfoCompare, idx: usize) -> Container<'a, Message>{
    
    // Retrieve image handler from struct
    let handler = data.icon_handler.clone().unwrap_or_else(|| Handle::from_bytes(vec![]));

    // Retrieve store(s) with lowest price
    let mut steam_lowest = data.lowest_price_stores.contains(&String::from(STEAM_STORE_ID));
    let mut gog_lowest = data.lowest_price_stores.contains(&String::from(GOG_STORE_ID));
    let mut ms_lowest = data.lowest_price_stores.contains(&String::from(MICROSOFT_STORE_ID));

    // Get count of stores with available sales
    let mut sale_avaiable_count: usize = 0;
    sale_avaiable_count += if data.steam_price.is_some() { 1 } else { 0 };
    sale_avaiable_count += if data.gog_price.is_some() { 1 } else { 0 };
    sale_avaiable_count += if data.microsoft_store_price.is_some() { 1 } else { 0 }; 

    // Toggle lowest pricing visual off if every available sale is the lowest
    if sale_avaiable_count > 1 && sale_avaiable_count == data.lowest_price_stores.len() {
        steam_lowest = false;
        gog_lowest = false;
        ms_lowest = false;
    }

    container(
        row![
            row![
                container(Image::new(handler)
                    .height(100)).align_y(Alignment::Center)
                    .padding(10),
                text(data.title.clone())
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .width(Length::FillPortion(2)),
            store_price_cell(&data.steam_price, steam_lowest)
                .width(Length::FillPortion(1)),
            store_price_cell(&data.gog_price, gog_lowest)
                .width(Length::FillPortion(1)),
            store_price_cell(&data.microsoft_store_price, ms_lowest)
                .width(Length::FillPortion(1)),
        ]
        .spacing(20)
        .padding(12)
        .align_y(Alignment::Center)
    )
    .style(cmp_row_style(idx))
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