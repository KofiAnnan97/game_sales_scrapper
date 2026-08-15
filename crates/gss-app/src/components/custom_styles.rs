use iced::widget::{ MouseArea, Text, button, container, mouse_area, text};
use iced::{Background, Border, Color, Font, Length, Shadow, Theme, font};

use crate::Message;

pub fn bold_text<'a>(data: &'a str) -> Text<'a>{
    text(data).font(Font{
            weight: font::Weight::Bold,
            ..Font::DEFAULT
    })
}

pub fn backdrop<'a>(message: Message) -> MouseArea<'a, Message>{
    mouse_area(
        container("")
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| {
                container::Style {
                    background: Some(
                        iced::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.55,
                        }
                        .into(),
                    ),
                    ..Default::default()
                }
            }
        )
    )
    .on_press(message)
}

pub fn dialog_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(47, 47, 47).into()),
        border: Border {
            radius: 14.0.into(),
            width: 1.0,
            color: Color::from_rgb8(220, 220, 220),
        },
        shadow: Shadow {
            color: Color::BLACK.scale_alpha(0.15),
            offset: iced::Vector::new(0.0, 6.0),
            blur_radius: 20.0,
        },
        text_color: None,
        snap: false
    }
}

pub fn normal_price_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: None,
        text_color: Some(Color::from_rgb8(255, 255, 255)),

        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: false
    }
}

pub fn best_price_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(34, 170, 59))),
        text_color: Some(Color::from_rgb8(255, 255, 255)),
        border: Border {
            radius: 20.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
            ..Default::default()
        },
        shadow: Shadow::default(),
        snap: false
    }
} 

pub fn cmp_row_style(index: usize) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        let bg = if index % 2 == 0 {
            Color::from_rgb8(76, 122, 165)
        } else {
            Color::from_rgb8(45, 76, 105)
        };

        container::Style {
            background: Some(Background::Color(bg)),
            text_color: None,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

pub fn highlight_on_click_style(theme: &iced::Theme, status: button::Status, selected: bool) -> button::Style {
    let mut style = button::text(theme, status);
    if selected {
        style.background = Some(iced::Background::Color(
            iced::Color::from_rgb8(51, 128, 255),
        ));
    }
    style
}