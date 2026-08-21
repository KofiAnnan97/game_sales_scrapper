use std::fs;
use serde_json::from_str;

use iced::{Element, Length};
use iced::widget::{row, column, Scrollable, text, Button};
use serde::Deserialize;
use crate::{MainMessage, Message};
use crate::log_utils::{LogLevel};
use crate::utils::log_utils::get_app_logs_path;

#[derive(Debug, Deserialize)]
pub struct LogData{
    timestamp: String,
    level: String,
    // screen: String,
    message: String
}

fn _get_app_logs() -> Vec<String> {
    let mut log_names: Vec<String> = Vec::new();
    match fs::read_dir(get_app_logs_path()) {
        Ok(dir) => {
            for d in dir {
                if let Some(file_type) = d.ok() {
                    log_names.push(file_type.file_name().to_str().unwrap().to_string());
                }
            }
        }
        Err(_) => {}
    }
    log_names
}


fn parse_logs(file_path: &str, log_batch: &str) -> Vec<LogData> {
    let mut log_data: Vec<LogData> = Vec::new();
    let raw_data = files::general::get_contents(file_path) + log_batch;
    for line in raw_data.lines() {
        match from_str::<LogData>(line.trim()) {
            Ok(data) => log_data.push(data),
            Err(_) => {}
        }
    }
    log_data
}

pub fn view(app: &crate::App) -> Element<'_, Message> {
    let complete_logs = parse_logs(&app.current_log_file, &app.log_batch);

    let logs_header = row![
        text("Timestamp").width(Length::Fixed(310.)),
        text("Level").width(Length::FillPortion(1)),
        text("Message").width(Length::FillPortion(2))
    ];
    let mut log_cols = column![];
    for log in &complete_logs {
        log_cols = log_cols.push(
            row![
                text(log.timestamp.clone()).width(Length::Fixed(300.)),
                text(log.level.clone()).width(Length::FillPortion(1)),
                text(log.message.clone()).width(Length::FillPortion(2))
            ]
            .spacing(10)
        );
    }
    let logs_display = Scrollable::new(log_cols)
        .width(Length::Fill)
        .height(Length::Fill);

    column![
        logs_header,
        logs_display,
        row![
            Button::new(text("Prune Logs"))
                .padding(8), 
            Button::new(text("Close"))
                .on_press(MainMessage::CloseLogsView.into())
                .padding(8),
        ]
        .spacing(10)
    ].into()
}
