use constants::operations::settings::{STEAM_STORE_ID, GOG_STORE_ID, MICROSOFT_STORE_ID};
use iced::widget::image::Handle;
use iced::{Alignment, alignment::Horizontal, Color, Theme, Renderer, Background};
use iced::widget::{Container, Text, button, container, row, column, table, text, Image, slider};
use iced::widget::table::Table;
use iced::{Element, Length, alignment, Alignment::Center};
use iced_aw::{iced_aw_font};

use types::internal::data::SaleInfo;

use crate::{MainMessage, Message};
use crate::components::custom_styles::{bold_text, cmp_row_style, dialog_style, normal_price_style, best_price_style, rounded_background, custom_button_style};
use crate::utils::pricing_utils::{SaleInfoCompare, StoreSale};
// Butttons

pub fn submenu_button(label: &str) -> Element<'_, Message> {
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

pub fn menu_text_button(label: &str, message: Message) -> Element<'_, Message> {
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

pub fn closable_window_button(label: &str, click_msg: Message, close_msg: Message, is_clicked: bool) -> Element<'_, Message> {
    let unclicked_color = Color::from_rgb8(20, 60, 120);
    let clicked_color = Color::from_rgb8(30, 100, 200);
    let hover_color = Color::from_rgb8(40, 120, 220);

    let background = if is_clicked {
        clicked_color
    } else {
        unclicked_color
    };

    let label_button = button(
        text(label)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left),
    )
    .on_press(click_msg)
    .padding([6, 8])
    .style(move |_, status| {
        let bg = if is_clicked {
            background
        } else {
            match status {
                button::Status::Hovered => hover_color.clone(),
                _ => background,
            }
        };
        custom_button_style(Some(Background::Color(bg)), Color::WHITE, 0.0)
    });

    let content = if is_clicked {
        row![
            label_button,
            button(
                text("×")
                    .size(16)
                    .align_x(alignment::Horizontal::Center),
            )
            .on_press(close_msg)
            .padding([4, 6])
            .style(move |_, status| {
                let background = match status {
                    button::Status::Hovered => {
                        Some(Background::Color(hover_color.clone()))
                    }
                    _ => None,
                };
                custom_button_style(background, Color::WHITE, 4.0)
            })
        ]
        .spacing(2)
        .align_y(alignment::Vertical::Center)
    } else {
        row![label_button]
            .align_y(alignment::Vertical::Center)
    };

    container(content)
        .width(Length::Shrink)
        .padding(2)
        .style(move |_| rounded_background(background, 6.0))
        .into()
}

// Badges

fn colored_badge<M: Clone + 'static>(value: String, font_size: u32, color: Color) -> iced::Element<'static ,M> {
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

fn price_change<'a, M: Clone + 'static>(old_price: f64, new_price: f64) -> Element<'a, M> {
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

fn store_link<'a, M: Clone + 'static>(label: &'a str, id: &'a str, url: String,
                                      on_link_click: impl Fn(String, String) -> M + 'static
) -> iced::Element<'a, M> {
    text::Rich::<String, M, Theme, Renderer>::with_spans([
        text::Span::new(label)
            .link(&url)
    ])
        .on_link_click(move |_| on_link_click(id.into(), url.clone()))
        .size(16)
        .into()
}

// Tables

pub fn _create_sales_table<'a>(sales_info: &'a Vec<SaleInfo>) -> Table<'a, Message>{
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

pub fn game_store_card<'a, M: Clone + 'static>(copied_link: &'a Option<String>, data: &'a StoreSale, 
    on_link_click: impl Fn(String, String) -> M + 'static
) -> Container<'a, M> {
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
                    data.info.store_page_link.clone(), 
                    on_link_click
                )
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

pub fn store_price_cell<M: Clone + 'static>(price: &Option<f64>, is_best: bool) -> Container<'_, M> {
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

pub fn game_sale_row<M: Clone + 'static>(data: &StoreSale, idx: usize) -> Container<'_, M> {
    let handler = data.icon_handler.clone().unwrap_or_else(|| Handle::from_bytes(vec![]));
    container(
        row![
            row![
                container(Image::new(handler)
                    .height(100)).align_y(Alignment::Center)
                    .padding(10),
                text(data.info.title.clone())
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .width(Length::FillPortion(2)),
            container(text(data.store.get_name())).width(Length::FillPortion(1)),
                container(text(data.info.current_price)).width(Length::FillPortion(1))
        ]
        .spacing(20)
        .padding(12)
        .align_y(Alignment::Center)
    )
    .style(cmp_row_style(idx))
}

pub fn game_comparison_row<M: Clone + 'static>(data: &SaleInfoCompare, idx: usize) -> Container<'_, M> {
    
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

pub fn message_dialog<'a, M: Clone + 'static>(title: &'a str, body: &'a str, message: M) -> Container<'a, M>{
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

// Elements

pub fn incremental_slider(choices: Vec<String>, selected: usize, max_width: f32) -> Element<'static, Message> {
    if choices.is_empty() {
        return text("").into();
    }

    let selected = selected.min(choices.len() - 1);

    let labels = row(
        choices.iter()
            .map(|option| {
                container(
                    text(option.clone()).size(15)
                )
                    .width(Length::FillPortion(1))
                    .center_x(Length::Fill)
                    .into()
            })
            .collect::<Vec<Element<'_, Message>>>(),
    )
        .width(Length::Fixed(max_width));

    let slider = slider(
        0.0..=(choices.len() - 1) as f64,
        selected as f64,
        |idx| MainMessage::LevelChanged(idx as usize).into(),
    )
        .step(1.0)
        .width(Length::Fixed(max_width*0.8));

    let showing = choices[selected..].join(", ");

    column![
        text(format!("Showing: {showing}")),
        container(slider)
            .center_x(Length::Fill)
            .padding([2, 0]),
        labels,
    ]
    .spacing(4)
    .width(Length::Fixed(max_width))
    .into()
}

// Simple Animations

pub fn text_loading_indicator<'a>(description: &str, current_frame: usize, frame_size: usize) -> Text<'a, Theme> {
    text(
        format!("{}{}", 
        description,
        ".".repeat((current_frame % frame_size) + 1)
    )).size(16)
}