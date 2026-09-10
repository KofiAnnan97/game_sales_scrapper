use crate::components::custom_widgets as cw;
use crate::log_utils::{LogData, parse_logs};
use crate::utils::log_utils;
use crate::utils::log_utils::{LogLevel, get_log_path};
use crate::views::sub_windows::LogItem;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Button, Scrollable, column, container, pick_list, row, text};
use iced::{Alignment, Background, Color, Element, Length, Task};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

const LOGS_PER_PAGE: usize = 20;

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
            Screen::Search => write!(f, "Search"),
            Screen::Thresholds => write!(f, "Thresholds"),
            Screen::Sales => write!(f, "Sales Preview"),
            Screen::Settings => write!(f, "Settings"),
            Screen::Logs => write!(f, "Logs"),
            Screen::Internal => write!(f, "Internal"),
            Screen::All => write!(f, "All"),
            Screen::None => write!(f, "-"),
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
        Screen::Internal,
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
    PageChanged(usize),
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
    RefreshLogs,
}

#[derive(Default)]
struct LogState {
    file_size: u64,
    entries: Vec<LogData>,
    pending_line: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFileOption {
    file_name: String,
    timestamp: String,
}

impl Display for LogFileOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.timestamp)
    }
}

pub struct LoggingView {
    curr_file_path: String,
    current_log: LogState,
    historical_logs: HashMap<String, LogState>,
    log_slider_idx: usize,
    log_file_selected: Option<String>,
    log_selected_screen: Option<Screen>,
    log_page: usize,
    pub log_items: Vec<LogItem>,
    pub prune_all: bool,
}

impl LoggingView {
    pub fn new(curr_log_path: &str) -> Self {
        let file_name = log_utils::get_filename(curr_log_path);
        Self {
            curr_file_path: String::from(curr_log_path),
            current_log: LogState::default(),
            historical_logs: HashMap::new(),
            log_slider_idx: get_defaut_slider_idx(),
            log_file_selected: Some(file_name.clone()),
            log_selected_screen: Some(Screen::All),
            log_page: 0,
            log_items: {
                let available_logs = log_utils::get_app_logs();
                let mut items: Vec<LogItem> = Vec::new();
                for (idx, file_name) in available_logs.iter().enumerate() {
                    items.push(LogItem {
                        id: idx,
                        file_name: file_name.clone(),
                        timestamp: log_utils::get_timestamp(file_name),
                        checked: false,
                    });
                }
                items
            },
            prune_all: false,
        }
    }

    pub fn update(&mut self, message: LoggingMessage) -> Task<LoggingMessage> {
        match message {
            LoggingMessage::LevelChanged(idx) => {
                self.log_slider_idx = idx;
                self.log_page = 0;
                Task::none()
            }
            LoggingMessage::PageChanged(page) => {
                self.log_page = page;
                Task::none()
            }
            LoggingMessage::LogFileChanged(dt_str) => {
                self.log_file_selected =
                    dt_str.as_ref().map(|selection| selection.file_name.clone());
                self.refresh_selected_historical_log();
                self.log_page = 0;
                Task::none()
            }
            LoggingMessage::LogScreenChanged(screen) => {
                self.log_selected_screen = Some(screen);
                self.log_page = 0;
                Task::none()
            }
            LoggingMessage::ToggleLogsToRemove(id, checked) => {
                let is_current = self
                    .log_items
                    .get(id)
                    .is_some_and(|item| self.is_current_file(&item.file_name));
                if is_current {
                    return Task::none();
                }

                if !checked {
                    self.prune_all = false;
                }

                if let Some(item) = self.log_items.get_mut(id) {
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
            LoggingMessage::RefreshLogs => {
                self.refresh_current_log();
                self.refresh_selected_historical_log();
                self.refresh_log_items();
                self.clamp_page();
                Task::none()
            }
            LoggingMessage::OpenManualPrune => Task::none(),
            LoggingMessage::Exit => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, LoggingMessage> {
        let level_options = LogLevel::get_options();
        let filtered_logs = self.filtered_logs(&level_options);

        let logs_header = row![
            text("Timestamp").width(Length::Fixed(310.)),
            text("Level").width(Length::FillPortion(1)),
            text("Screen").width(Length::FillPortion(1)),
            text("Message").width(Length::FillPortion(3))
        ];

        let total_log_count = filtered_logs.len();
        let page_count = total_log_count.div_ceil(LOGS_PER_PAGE);
        let page = self.log_page.min(page_count.saturating_sub(1));
        let page_start_idx = page * LOGS_PER_PAGE;
        let page_end_idx = (page_start_idx + LOGS_PER_PAGE).min(total_log_count);

        let mut log_cols = column![];
        for (idx, log) in filtered_logs[page_start_idx..page_end_idx]
            .iter()
            .enumerate()
        {
            log_cols = log_cols.push(log_row(log.clone(), page_start_idx + idx));
        }

        let logs_display = Scrollable::new(log_cols)
            .width(Length::Fill)
            .height(Length::Fill);

        let log_files = self
            .log_items
            .iter()
            .map(|item| LogFileOption {
                file_name: item.file_name.clone(),
                timestamp: item.timestamp.clone(),
            })
            .collect::<Vec<_>>();

        let selected_log = self.log_file_selected.as_ref().and_then(|file_name| {
            self.log_items
                .iter()
                .find(|item| &item.file_name == file_name)
                .map(|item| LogFileOption {
                    file_name: item.file_name.clone(),
                    timestamp: item.timestamp.clone(),
                })
        });

        let log_range = if total_log_count == 0 {
            "Showing 0-0 of 0 lines".to_string()
        } else {
            format!(
                "Showing {}-{} of {} lines",
                page_start_idx + 1,
                page_end_idx,
                total_log_count
            )
        };

        let mut prev_page_btn = Button::new(text("Previous")).padding(6);
        if page > 0 {
            prev_page_btn = prev_page_btn.on_press(LoggingMessage::PageChanged(page - 1));
        }

        let mut next_page_btn = Button::new(text("Next")).padding(6);
        if page + 1 < page_count {
            next_page_btn = next_page_btn.on_press(LoggingMessage::PageChanged(page + 1));
        }

        column![
            container(cw::incremental_slider(
                level_options,
                self.log_slider_idx,
                600.,
                LoggingMessage::LevelChanged
            ))
            .height(Length::Fixed(80.))
            .center_x(Length::Fill),
            row![
                container(
                    row![
                        text("Log File:"),
                        pick_list(log_files, selected_log, |selection| {
                            LoggingMessage::LogFileChanged(Some(selection))
                        },),
                    ]
                    .align_y(Vertical::Center)
                    .spacing(10)
                    .height(Length::Fixed(30.))
                ),
                container(
                    row![
                        text("Screen:"),
                        pick_list(
                            &Screen::OPTIONS[..],
                            self.log_selected_screen,
                            LoggingMessage::LogScreenChanged,
                        )
                    ]
                    .align_y(Vertical::Center)
                    .spacing(5)
                    .height(Length::Fixed(30.))
                ),
                container(
                    row![prev_page_btn, text(log_range), next_page_btn,]
                        .spacing(12)
                        .align_y(Alignment::Center)
                )
                .width(Length::Fill)
                .align_x(Horizontal::Right),
            ]
            .spacing(20),
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
        let selected_item = self.log_file_selected.as_ref().and_then(|file_name| {
            self.log_items
                .iter()
                .find(|item| &item.file_name == file_name)
        });
        let item = selected_item
            .or_else(|| {
                self.log_items
                    .iter()
                    .find(|item| self.is_current_file(&item.file_name))
            })
            .or_else(|| self.log_items.first());

        self.log_file_selected = item.map(|item| item.file_name.clone());
    }

    fn sync_prune_all(&mut self) {
        let current_file_name = log_utils::get_filename(&self.curr_file_path);
        let mut deletable = self
            .log_items
            .iter()
            .filter(|item| item.file_name != current_file_name);
        self.prune_all = deletable.clone().next().is_some() && deletable.all(|item| item.checked);
    }

    fn clamp_page(&mut self) {
        let level_options = LogLevel::get_options();
        let total_log_count = self.filtered_logs(&level_options).len();
        let page_count = total_log_count.div_ceil(LOGS_PER_PAGE);
        self.log_page = self.log_page.min(page_count.saturating_sub(1));
    }

    fn filtered_logs(&self, level_options: &[String]) -> Vec<LogData> {
        let displayed_logs = if let Some(file_path) = &self.log_file_selected {
            if self.is_current_file(file_path) {
                self.current_log.entries.clone()
            } else {
                self.historical_logs
                    .get(file_path)
                    .map(|log| log.entries.clone())
                    .unwrap_or_default()
            }
        } else {
            Vec::new()
        };

        displayed_logs
            .into_iter()
            .filter(|log| {
                let level_idx = level_options
                    .iter()
                    .position(|level| level == &log.level)
                    .unwrap_or(0);
                self.log_slider_idx <= level_idx
                    && (self.log_selected_screen == Some(Screen::All)
                        || self.log_selected_screen == Some(log.screen.clone().into()))
            })
            .collect()
    }

    pub(crate) fn is_current_file(&self, file_name: &str) -> bool {
        file_name == log_utils::get_filename(&self.curr_file_path)
    }

    fn refresh_log_items(&mut self) {
        let available_logs = log_utils::get_app_logs();
        let current_file_name = log_utils::get_filename(&self.curr_file_path);
        let previous_items = std::mem::take(&mut self.log_items);
        let previous_checked: HashMap<String, bool> = previous_items
            .into_iter()
            .map(|item| (item.file_name, item.checked))
            .collect();
        self.log_items = available_logs
            .into_iter()
            .enumerate()
            .map(|(id, file_name)| {
                let checked = previous_checked.get(&file_name).copied().unwrap_or(false);
                let is_current = file_name == current_file_name;
                LogItem {
                    id,
                    timestamp: log_utils::get_timestamp(&file_name),
                    file_name,
                    checked: checked && !is_current,
                }
            })
            .collect();
        self.normalize_selection();
        self.sync_prune_all();
    }

    fn refresh_current_log(&mut self) {
        refresh_log_file(&self.curr_file_path, &mut self.current_log);
    }

    fn refresh_selected_historical_log(&mut self) {
        let file_name = match self.log_file_selected.as_ref() {
            Some(file_name) if !self.is_current_file(file_name) => file_name.clone(),
            _ => return,
        };

        let path_buf: PathBuf = [&get_log_path(), &file_name].iter().collect();
        let log_state = self.historical_logs.entry(file_name).or_default();
        refresh_log_file(&path_buf.display().to_string(), log_state);
    }
}

fn refresh_log_file(file_path: &str, log_state: &mut LogState) {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            *log_state = LogState::default();
            return;
        }
    };

    let file_size = match file.metadata().map(|metadata| metadata.len()) {
        Ok(size) => size,
        Err(_) => return,
    };

    if file_size < log_state.file_size {
        *log_state = LogState::default();
    }

    if file_size == log_state.file_size {
        return;
    }

    if file.seek(SeekFrom::Start(log_state.file_size)).is_err() {
        return;
    }

    let mut appended = String::new();
    if file.read_to_string(&mut appended).is_err() {
        return;
    }

    let mut buffer = std::mem::take(&mut log_state.pending_line);
    buffer.push_str(&appended);

    let lines = buffer.split_inclusive('\n');

    for line in lines {
        if line.ends_with('\n') {
            log_state.entries.extend(parse_logs(line));
        } else {
            log_state.pending_line.push_str(line);
        }
    }

    log_state.file_size = file_size;
}

fn get_defaut_slider_idx() -> usize {
    // TODO: Call gss-core to retrieve default log level from settings.json under app
    LogLevel::get_level_idx("INFO".into())
}

pub fn log_row<M: Clone + 'static>(log: LogData, index: usize) -> Element<'static, M> {
    let background = if index.is_multiple_of(2) {
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
