use std::path::PathBuf;
use iced::{Element, Length, Alignment, Background, Color, Task};
use iced::widget::{row, Scrollable, text, Button, container, pick_list, column};
use crate::components::custom_widgets as cw;
use crate::utils::log_utils::{LogLevel, get_log_data, get_log_path};
use crate::log_utils::{LogData, parse_logs};
use crate::utils::log_utils;
use crate::views::sub_windows::LogItem;

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

#[derive(Debug, Clone)]
pub enum LoggingMessage {
    LevelChanged(usize),
    LogFileChanged(Option<LogFileOption>),
    LogScreenChanged(Screen),
    ToggleLogsToRemove(usize, bool),
    DeleteLogs,
    DeleteAllButCurrent,
    PruneAllLogs(bool),
    // Message(s) to communicate up to App
    OpenManualPrune,
    Exit,
    //Message(s) communicated back from App
    RefreshLogs(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFileOption {
    file_name: String,
    timestamp: String,
}

impl std::fmt::Display for LogFileOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.timestamp)
    }
}

pub struct LoggingView {
    curr_file_path: String,
    curr_raw_logs: String,
    log_slider_idx: usize,
    log_file_selected: Option<String>,
    log_selected_screen: Option<Screen>,
    pub log_items: Vec<LogItem>,
    pub prune_all: bool,
}

impl LoggingView {
    pub fn new(curr_log_path: &str) -> Self {
        let file_name = log_utils::get_filename(curr_log_path);
        Self {
            curr_file_path: String::from(curr_log_path),
            curr_raw_logs: String::new(),
            log_slider_idx: get_defaut_slider_idx(),
            log_file_selected: Some(file_name.clone()),
            log_selected_screen: Some(Screen::All),
            log_items: {
                let available_logs = log_utils::get_app_logs();
                let mut items: Vec<LogItem> = Vec::new();
                for (idx, file_name) in available_logs.iter().enumerate() {
                    items.push(LogItem { 
                        id: idx, 
                        file_name: file_name.clone(), 
                        timestamp: log_utils::get_timestamp(file_name),
                        checked: false 
                    });
                }
                items
            },
            prune_all: false
        }
    }

    pub fn update(&mut self, message: LoggingMessage) -> Task<LoggingMessage> {
        match message {
            LoggingMessage::LevelChanged(idx) => {
                self.log_slider_idx = idx;
                Task::none()
            }
            LoggingMessage::LogFileChanged(dt_str) => {
                self.log_file_selected = dt_str.as_ref().map(|selection| selection.file_name.clone());
                Task::none()
            }
            LoggingMessage::LogScreenChanged(screen) => {
                self.log_selected_screen = Some(screen);
                Task::none()
            }
            LoggingMessage::ToggleLogsToRemove(id, checked) => {
                let is_current = self.log_items.iter()
                    .find(|item| item.id == id)
                    .is_some_and(|item| self.is_current_file(&item.file_name));
                if is_current {
                    return Task::none();
                }

                if !checked {
                    self.prune_all = false;
                }

                if let Some(item) = self.log_items.iter_mut().find(|item| item.id == id) {
                    item.checked = checked;
                }
                self.sync_prune_all();
                Task::none()
            }
            LoggingMessage::DeleteAllButCurrent => {
                let current_file_name = log_utils::get_filename(&self.curr_file_path);
                self.log_items.retain_mut(|item| {
                    if item.file_name == current_file_name {
                        true
                    } else {
                        let _ = log_utils::delete_log(&item.file_name);
                        false
                    }
                });
                self.prune_all = false;
                self.normalize_selection();
                Task::none()
            }
            LoggingMessage::DeleteLogs => {
                let current_file_name = log_utils::get_filename(&self.curr_file_path);
                self.log_items.retain_mut(|item| {
                    if item.file_name != current_file_name && item.checked {
                        let _ = log_utils::delete_log(&item.file_name);
                        false
                    } else {
                        true
                    }
                });

                self.prune_all = false;
                self.normalize_selection();
                Task::none()
            }
            LoggingMessage::PruneAllLogs(toggle) => {
                let current_file_name = log_utils::get_filename(&self.curr_file_path);
                self.prune_all = toggle;
                for item in self.log_items.iter_mut() {
                    item.checked = item.file_name != current_file_name && toggle;
                }
                Task::none()
            }
            LoggingMessage::RefreshLogs(logs) => {
                self.curr_raw_logs = logs;
                self.refresh_log_items();
                Task::none()
            }
            LoggingMessage::OpenManualPrune => { Task::none() }
            LoggingMessage::Exit => { Task::none() }
        }
    }

    pub fn view(&self) -> Element<'_, LoggingMessage> {
        let complete_logs = if let Some(file_path) = &self.log_file_selected {
            if file_path == &self.curr_file_path {
                parse_logs(self.curr_raw_logs.clone())
            } else {
                let path_buf: PathBuf = [&get_log_path(), &file_path].iter().collect();
                let path_str  = path_buf.display().to_string();
                let raw_logs = get_log_data(&path_str);
                parse_logs(raw_logs)
            }
        } else {
            Vec::new()
        };

        let level_options = LogLevel::get_options();

        let logs_header = row![
            text("Timestamp").width(Length::Fixed(310.)),
            text("Level").width(Length::FillPortion(1)),
            text("Screen").width(Length::FillPortion(1)),
            text("Message").width(Length::FillPortion(3))
        ];
        let mut log_cols = iced::widget::column![];
        for (idx, log) in complete_logs.iter().enumerate() {
            let level_idx = level_options.iter().position(|r| r == &log.level).unwrap_or(0);
            if self.log_slider_idx <= level_idx && (self.log_selected_screen == Some(Screen::All) || self.log_selected_screen == Some(log.screen.clone().into())) {
                log_cols = log_cols.push(log_row(log.clone(), idx));
            }
        }
        let logs_display = Scrollable::new(log_cols)
            .width(Length::Fill)
            .height(Length::Fill);

        let log_files = self.log_items.iter().map(|item| LogFileOption {
            file_name: item.file_name.clone(),
            timestamp: item.timestamp.clone(),
        }).collect::<Vec<_>>();
        let selected_log = self.log_file_selected.as_ref().and_then(|file_name| {
            self.log_items.iter().find(|item| &item.file_name == file_name).map(|item| LogFileOption {
                file_name: item.file_name.clone(),
                timestamp: item.timestamp.clone(),
            })
        });

        column![
            container(cw::incremental_slider(level_options, self.log_slider_idx, 600., |idx| LoggingMessage::LevelChanged(idx)))
                .height(Length::Fixed(80.))
                .center_x(Length::Fill),
            row![
                container(
                    row![
                        text("Log File:"),
                        pick_list(
                            log_files,
                            selected_log,
                            |selection| LoggingMessage::LogFileChanged(Some(selection)),
                        ),
                    ].spacing(10)
                    .height(Length::Fixed(30.))
                ),
                container(
                    row![
                        text("Screen:"),
                        pick_list(
                            &Screen::OPTIONS[..],
                            self.log_selected_screen.clone(),
                            |screen| LoggingMessage::LogScreenChanged(screen),
                        )
                    ].spacing(5)
                    .height(Length::Fixed(30.))
                ).padding(6),
            ].spacing(20),
            logs_header,
            logs_display,
            row![
                Button::new(text("Prune Logs"))
                    .on_press(LoggingMessage::OpenManualPrune)
                    .padding(8),
                Button::new(text("Close"))
                    .on_press(LoggingMessage::Exit)
                    .padding(8),
            ]
            .spacing(10)
        ]
            .into()
    }

    fn normalize_selection(&mut self) {
        let selected_item = self.log_file_selected.as_ref()
            .and_then(|file_name| self.log_items.iter().find(|item| &item.file_name == file_name));
        let item = selected_item
            .or_else(|| self.log_items.iter().find(|item| self.is_current_file(&item.file_name)))
            .or_else(|| self.log_items.first());

        self.log_file_selected = item.map(|item| item.file_name.clone());
    }

    fn sync_prune_all(&mut self) {
        let current_file_name = log_utils::get_filename(&self.curr_file_path);
        let mut deletable = self.log_items.iter().filter(|item| item.file_name != current_file_name);
        self.prune_all = deletable.clone().next().is_some() && deletable.all(|item| item.checked);
    }

    pub(crate) fn is_current_file(&self, file_name: &str) -> bool {
        file_name == log_utils::get_filename(&self.curr_file_path)
    }

    fn refresh_log_items(&mut self) {
        let available_logs = log_utils::get_app_logs();
        let current_file_name = log_utils::get_filename(&self.curr_file_path);
        let previous_items = std::mem::take(&mut self.log_items);
        self.log_items = available_logs.into_iter().enumerate().map(|(id, file_name)| {
            let checked = previous_items.iter()
                .find(|item| item.file_name == file_name)
                .map(|item| item.checked)
                .unwrap_or(false);
            let is_current = file_name == current_file_name;
            LogItem {
                id,
                timestamp: log_utils::get_timestamp(&file_name),
                file_name,
                checked: checked && !is_current,
            }
        }).collect();
        self.normalize_selection();
        self.sync_prune_all();
    }
}

fn get_defaut_slider_idx() -> usize {
    // TODO: Call gss-core to retrieve default log level from settings.json under app
    LogLevel::get_level_idx("INFO".into())
}

pub fn log_row<M: Clone + 'static>(log: LogData, index: usize) -> Element<'static, M> {
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
        }).into()
}