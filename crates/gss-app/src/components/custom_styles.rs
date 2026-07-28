use iced::widget::{ Text, text};
use iced::{font, Font};

pub fn bold_text<'a>(data: &'a str) -> Text<'a>{
    text(data).font(Font{
            weight: font::Weight::Bold,
            ..Font::DEFAULT
    })
}