use std::path::PathBuf;
use iced::{Element, Length, Alignment, Background, Color};
use iced::widget::{row, column, Scrollable, text, Button, container, pick_list};

use crate::components::custom_widgets as cw;
use crate::{MainMessage, Message};
use crate::utils::log_utils::{get_log_path, LogLevel, FATAL, get_log_data};
use crate::log_utils::{LogData, get_app_logs, parse_logs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Search,
    Thresholds,
    Sales,
    Settings,
    Logs,
    Internal,
    All,
    None,
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Screen::Search => write!(f,"Search"),
            Screen::Thresholds => write!(f,"Thresholds"),
            Screen::Sales => write!(f,"Sales Preview"),
            Screen::Settings => write!(f,"Settings"),
            Screen::Logs => write!(f,"Logs"),
            Screen::Internal => write!(f,"Internal"),
            Screen::All => write!(f,"All"),
            Screen::None => write!(f,"-")
        }
    }
}

impl Screen {
    pub const OPTIONS: [Screen; 7] = [
        Screen::All,
        Screen::Search,
        Screen::Thresholds,
        Screen::Sales,
        Screen::Settings,
        Screen::Logs,
        Screen::Internal
    ];
}

impl From<String> for Screen {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Search" => Screen::Search,
            "Thresholds" => Screen::Thresholds,
            "Sales Preview" => Screen::Sales,
            "Settings" => Screen::Settings,
            "Logs" => Screen::Logs,
            "Internal" => Screen::Internal,
            "All" => Screen::All,
            _ => Screen::None,
        }
    }
}


// #[derive(Debug, Clone)]
// pub enum LoggingMessage {
//     LevelChanged(usize),
//     LogFileChanged(String),
//     LogScreenChanged(Screen),
//     // Message(s) to communicate up to App
//     Exit,
//     //Message(s) communicated back from App
//     AddLogToBatch(String),
//     UpdateLogFile,
// }

// pub struct LoggingView {
//     current_log_file: String,
//     log_batch: String,
//     log_slider_idx: usize,
//     log_file_selected: Option<String>,
//     log_selected_screen: Option<Screen>,
// }

// impl Default for LoggingView {
//     fn default() -> Self {
//         Self {
//             current_log_file: String::new(),
//             log_batch: String::new(),
//             log_slider_idx: 0,
//             log_file_selected: None,
//             log_selected_screen: Some(Screen::All),
//         }
//     }
// }

// impl LoggingView {
//
//     pub fn new(curr_log_path: &str) -> Self {
//         Self {
//             current_log_file: curr_log_path.into(),
//             log_batch: String::new(),
//             log_slider_idx: 0,
//             log_file_selected: None,
//             log_selected_screen: Some(Screen::All),
//         }
//     }
//
//     pub fn update(&mut self, message: LoggingMessage) -> Task<LoggingMessage> {
//         match message {
//             LoggingMessage::LevelChanged(idx) => {
//                 self.log_slider_idx = idx;
//                 Task::none()
//             }
//             LoggingMessage::LogFileChanged(dt_str) => {
//                 self.log_file_selected = Some(dt_str);
//                 Task::none()
//             }
//             LoggingMessage::LogScreenChanged(screen) => {
//                 self.log_selected_screen = Some(screen);
//                 Task::none()
//             }
//             LoggingMessage::Exit => { Task::none() }
//             LoggingMessage::AddLogToBatch(log_msg) => {
//                 self.log_batch.push_str(&log_msg);
//                 Task::none()
//             }
//             LoggingMessage::UpdateLogFile => {
//                 if !self.log_batch.is_empty() && !self.current_log_file.is_empty() {
//                     general::append_to_file(&self.current_log_file, &self.log_batch);
//                     self.log_batch.clear();
//                 }
//                 Task::none()
//             }
//         }
//     }
//
//     pub fn view(&self) -> Element<'_, LoggingMessage> {
//         let complete_logs = parse_logs(&self.current_log_file, &self.log_batch);
//
//         let level_options = vec![
//             LogLevel::DEBUG.to_string(),
//             LogLevel::WARN.to_string(),
//             LogLevel::INFO.to_string(),
//             LogLevel::ERROR.to_string(),
//             "FATAL".into()
//         ];
//
//         let logs_header = row![
//             text("Timestamp").width(Length::Fixed(310.)),
//             text("Level").width(Length::FillPortion(1)),
//             text("Screen").width(Length::FillPortion(1)),
//             text("Message").width(Length::FillPortion(3))
//         ];
//
//         let mut log_cols = column![];
//         for (idx, log) in complete_logs.iter().enumerate() {
//             let level_idx = level_options.iter().position(|r| r == &log.level).unwrap_or(0);
//             if self.log_slider_idx <= level_idx && ( self.log_selected_screen == Some(Screen::All) ||
//                 self.log_selected_screen == Some(log.screen.clone().into())){
//                 let log_copy = log.clone();
//                 log_cols = log_cols.push(
//                     // row![
//                     //     text(log.timestamp.clone()).width(Length::Fixed(300.)),
//                     //     text(log.level.clone()).width(Length::FillPortion(1)),
//                     //     text(log.screen.clone()).width(Length::FillPortion(1)),
//                     //     text(log.message.clone()).width(Length::FillPortion(3))
//                     // ]
//                     //     .spacing(10)
//                     log_row(log.clone(), idx)
//                 );
//             }
//         }
//         let logs_display = Scrollable::new(log_cols)
//             .width(Length::Fill)
//             .height(Length::Fill);
//
//         column![
//             container(cw::incremental_slider(level_options, self.log_slider_idx, 600.))
//                 .height(Length::Fixed(80.))
//                 .center_x(Length::Fill),
//             row![
//                 container(
//                     row![
//                         text("Date:"),
//                         // pick_list(
//                         //     &vec!["value"],
//                         //     Some(app.log_file_selected.clone()),
//                         //     LoggingMessage::LogFileChanged,
//                         // ),
//                     ].spacing(10)
//                     .height(Length::Fixed(30.))
//                 ),
//                 container(
//                     row![
//                         text("Screen:"),
//                         pick_list(
//                             &Screen::OPTIONS[..],
//                             self.log_selected_screen.clone(),
//                             |screen| LoggingMessage::LogScreenChanged(screen),
//                         )
//                     ].spacing(5)
//                     .height(Length::Fixed(30.))
//                 ).padding(6),
//             ].spacing(20),
//             logs_header,
//             logs_display,
//             row![
//                 Button::new(text("Prune Logs"))
//                     .padding(8),
//                 Button::new(text("Close"))
//                     .on_press(LoggingMessage::Exit)
//                     .padding(8),
//             ]
//             .spacing(10)
//         ]
//         .into()
//     }
// }

pub fn view(app: &crate::App) -> Element<'_, Message> {

    let complete_logs = if let Some(file_path) = &app.log_file_selected {
        if file_path == &app.logger.get_filename() {
            let raw_logs = app.logger.get_full_logs();
            parse_logs(raw_logs)
        } else {
            let path_buf: PathBuf = [&get_log_path(), &file_path].iter().collect();
            let raw_logs = get_log_data(&path_buf.display().to_string());
            parse_logs(raw_logs)
        }
    } else {
        Vec::new()
    };

    let level_options = vec![
      LogLevel::DEBUG.to_string(),
      LogLevel::WARN.to_string(),
      LogLevel::INFO.to_string(),
      LogLevel::ERROR.to_string(),
      FATAL.into()
    ];

    let logs_header = row![
        text("Timestamp").width(Length::Fixed(310.)),
        text("Level").width(Length::FillPortion(1)),
        text("Screen").width(Length::FillPortion(1)),
        text("Message").width(Length::FillPortion(3))
    ];
    let mut log_cols = column![];
    for (idx, log) in complete_logs.iter().enumerate() {
        let level_idx = level_options.iter().position(|r| r == &log.level).unwrap_or(0);
        if app.log_slider_idx <= level_idx && ( app.log_selected_screen == Some(Screen::All) || app.log_selected_screen == Some(log.screen.clone().into())){
            log_cols = log_cols.push(log_row(log.clone(), idx));
        }
    }
    let logs_display = Scrollable::new(log_cols)
        .width(Length::Fill)
        .height(Length::Fill);

    let logs = get_app_logs();

    column![
        container(cw::incremental_slider(level_options, app.log_slider_idx, 600.))
            .height(Length::Fixed(80.))
            .center_x(Length::Fill),
        row![
            container(
                row![
                    text("Log File:"),
                    pick_list(
                        logs,
                        app.log_file_selected.clone(),
                        |dt_str| MainMessage::LogFileChanged(Some(dt_str)).into(),
                    ),
                ].spacing(10)
                .height(Length::Fixed(30.))
            ),
            container(
                row![
                    text("Screen:"),
                    pick_list(
                        &Screen::OPTIONS[..],
                        app.log_selected_screen.clone(),
                        |screen| MainMessage::LogScreenChanged(screen).into(),
                    )
                ].spacing(5)
                .height(Length::Fixed(30.))
            ).padding(6),
        ].spacing(20),
        logs_header,
        logs_display,
        row![
            Button::new(text("Prune Logs"))
                .on_press(MainMessage::OpenManualPrune.into())
                .padding(8),
            Button::new(text("Close"))
                .on_press(MainMessage::CloseLogsView.into())
                .padding(8),
        ]
        .spacing(10)
    ]
    .into()
}

fn log_row<M: Clone + 'static>(log: LogData, index: usize) -> Element<'static, M> {
    let background = if index % 2 == 0 {
        Color::from_rgb8(10, 11, 12)
    } else {
        Color::BLACK
    };

    container(
        row![
            text(log.timestamp.clone()).width(Length::Fixed(300.0)),
            text(log.level.clone()).width(Length::FillPortion(1)),
            text(log.screen.clone()).width(Length::FillPortion(1)),
            text(log.message.clone()).width(Length::FillPortion(3)),
        ]
            .spacing(16)
            .align_y(Alignment::Center),
    )
        .width(Length::Fill)
        .padding([10, 14])
        .style(move |_theme| container::Style {
            background: Some(Background::Color(background)),
            ..Default::default()
        })
        .into()
}