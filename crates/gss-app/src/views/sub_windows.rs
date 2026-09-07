use iced::alignment::Horizontal;
use iced::{Alignment, Element, Length};
use iced::widget::center;
use iced::widget::{Button,  Column, Container, Scrollable, checkbox, column, container, row, text};

use crate::{App, Message};
use crate::views::logs::LoggingMessage;

#[derive(Debug, Clone)]
pub struct LogItem {
    pub id: usize,
    pub file_name: String,
    pub timestamp: String,
    pub checked: bool,
}

pub fn checkable_logs(app: &crate::App) -> Container<'_, Message> {
    let checkboxes = app.logging_view.log_items.iter()
        .filter(|log| !app.logging_view.is_current_file(&log.file_name))
        .map(|log|{
        checkbox(log.checked)
            .label(&log.file_name)
            .on_toggle(move |checked| LoggingMessage::ToggleLogsToRemove(log.id, checked).into())
            .width(Length::Fill)
            .into()
    });
    
    container(
        column(checkboxes).spacing(10)
            .padding(5)
    ).into()
}


pub fn manual_prune(app: &App) -> Element<'_, Message> {
    if app.logging_view.log_items.iter()
        .any(|log| !app.logging_view.is_current_file(&log.file_name)) {
        regular_prune_view(app)
    } else {
        message_window(app,None,Some("There are no logs to prune."))
    }
}

fn regular_prune_view(app: &App) -> Element<'_, Message> {
    let files_display: Column<Message> = column![
        container(
            checkbox(app.logging_view.prune_all)
                .label("Select All")
                .on_toggle(|toggle| LoggingMessage::PruneAllLogs(toggle).into())
                .width(Length::Fill)
        ).padding(5),
        Scrollable::new(checkable_logs(app)).height(Length::Fixed(360.)),
    ];

    let delete_btn = Button::new("Delete")
        .padding(8);
    let delete_btn = delete_btn.on_press(LoggingMessage::DeleteLogs.into());

    container(
        column![
            text("Select Logs to Delete: ").size(20.),
            files_display,
            row![
                delete_btn,
                Button::new("Close")
                    .on_press(Message::CloseWindow(app.manual_prune_window.unwrap()))
                    .padding(8)
            ].spacing(10)
            .height(Length::Fixed(60.))
        ]
        .spacing(10)
        .padding(15)
    )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .height(Length::Fill)
        .into()
}

pub fn message_window<'a>(app: &'a App,title: Option<&'a str>,message: Option<&'a str>) -> Element<'a, Message> {
    let window_id = app.manual_prune_window.expect("message window requires an open window");
    let mut message_content = column![];

    if let Some(title) = title {
        message_content = message_content.push(
            container(
                text(title).size(24)
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        );
    }

    if let Some(message) = message {
        message_content = message_content.push(
            container(
                text(message).size(16).width(Length::Fill)
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        );
    }

    message_content = message_content.push(
        container(
            Button::new("OK")
                .padding([8, 24])
                .on_press(Message::CloseWindow(window_id)),
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center),
    );

    center(
        container(
            message_content.spacing(24).padding(24),
        )
        .width(360)
        .max_width(360),
    )
    .into()
}