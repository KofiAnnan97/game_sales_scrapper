use iced::{Element, Length};
use iced::widget::{Button,  Column, Container, Scrollable, checkbox, column, container, row, text};

use crate::{App, MainMessage, Message};


pub struct LogItem {
    pub id: usize,
    pub name: String,
    pub checked: bool,
}

pub fn checkable_logs(app: &crate::App) -> Container<'_, Message> {
    let checkboxes = app.log_items.iter().map(|log|{
        checkbox(log.checked)
            .label(&log.name)
            .on_toggle(move |checked| MainMessage::ToggleLogsToRemove(log.id, checked).into())
            .width(Length::Fill)
            .into()
    });
    
    container(
        column(checkboxes).spacing(10)
            .padding(5)
    ).into()
}


pub fn manual_prune(app: &App) -> Element<'_, Message> {
    let files_display: Column<Message> = column![
            checkbox(app.prune_all)
                .label("Select All")
                .on_toggle(|toggle| MainMessage::PruneAllLogs(toggle).into())
                .width(Length::Fill),
            checkable_logs(app)
        ];

    container(
        column![
            text("Select logs to be removed:").size(20.),
            Scrollable::new(files_display).height(Length::Fixed(660.)),
            row![
                Button::new("Delete")
                    .on_press(MainMessage::DeleteLogs.into())
                    .padding(8),
                Button::new("Close")
                    .on_press(Message::CloseWindow(app.manual_prune_window.unwrap()))
                    .padding(8)
            ].spacing(10)
            .height(Length::Fixed(60.))
        ]
        .spacing(10)
        .padding(20)
    )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}