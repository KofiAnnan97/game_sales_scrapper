use crate::components::custom_widgets as cw;
use crate::log_utils::{LogData, parse_logs};
use crate::utils::log_utils;
use crate::utils::log_utils::{LogLevel, get_log_path};
use crate::views::sub_windows::LogItem;
use files::general;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Button, Scrollable, column, container, pick_list, row, text};
use iced::{Alignment, Background, Color, Element, Length, Task};
use iced_aw::Spinner;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const LOGS_PER_PAGE: usize = 20;
const MAX_CACHED_HISTORICAL_LOGS: usize = 5;
const PAGES_PER_WINDOW: usize = 20;
const ENTRIES_PER_WINDOW: usize = LOGS_PER_PAGE * PAGES_PER_WINDOW;

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
    LogWindowLoaded(u64, String, LogState),
}

#[derive(Clone, Debug, Default)]
pub struct LogState {
    file_size: u64,
    entries: Vec<LogData>,
    pending_line: String,
    window_start_page: usize,
    total_matching_entries: usize,
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
    historical_log_order: VecDeque<String>,
    filtered_log_indices: Vec<usize>,
    log_slider_idx: usize,
    log_file_selected: Option<String>,
    log_selected_screen: Option<Screen>,
    log_page: usize,
    next_request_id: u64,
    active_request_id: u64,
    logs_loading: bool,
    pub log_items: Vec<LogItem>,
    pub prune_all: bool,
}

impl LoggingView {
    pub fn new(curr_log_path: &str) -> Self {
        let file_name = log_utils::get_filename(curr_log_path);
        let mut view = Self {
            curr_file_path: String::from(curr_log_path),
            current_log: LogState::default(),
            historical_logs: HashMap::new(),
            historical_log_order: VecDeque::new(),
            filtered_log_indices: Vec::new(),
            log_slider_idx: get_defaut_slider_idx(),
            log_file_selected: Some(file_name.clone()),
            log_selected_screen: Some(Screen::All),
            log_page: 0,
            next_request_id: 0,
            active_request_id: 0,
            logs_loading: true,
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
        };
        view.refresh_filter_cache();
        view
    }

    pub fn update(&mut self, message: LoggingMessage) -> Task<LoggingMessage> {
        match message {
            LoggingMessage::LevelChanged(idx) => {
                self.log_slider_idx = idx;
                self.log_page = 0;
                self.schedule_selected_log_window(0, true)
            }
            LoggingMessage::PageChanged(page) => {
                let (window_start_page, total_pages) = self.selected_page_window();
                if page < window_start_page || page >= window_start_page + PAGES_PER_WINDOW {
                    let new_start_page = page
                        .saturating_sub(PAGES_PER_WINDOW / 2)
                        .min(total_pages.saturating_sub(PAGES_PER_WINDOW));
                    self.clear_selected_log_window(new_start_page);
                    self.log_page = page;
                    return self.schedule_selected_log_window(new_start_page, true);
                }
                self.log_page = page;
                Task::none()
            }
            LoggingMessage::LogFileChanged(dt_str) => {
                self.log_file_selected =
                    dt_str.as_ref().map(|selection| selection.file_name.clone());
                self.log_page = 0;
                self.schedule_selected_log_window(0, true)
            }
            LoggingMessage::LogScreenChanged(screen) => {
                self.log_selected_screen = Some(screen);
                self.log_page = 0;
                self.schedule_selected_log_window(0, true)
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
                let mut deleted_files = Vec::new();
                self.log_items.retain_mut(|item| {
                    if item.file_name == current_file_name {
                        true
                    } else {
                        let _ = log_utils::delete_log(&item.file_name);
                        deleted_files.push(item.file_name.clone());
                        false
                    }
                });
                for file_name in deleted_files {
                    self.remove_historical_log_cache(&file_name);
                }
                self.prune_all = false;
                self.normalize_selection();
                self.schedule_selected_log_window(0, true)
            }
            LoggingMessage::DeleteLogs => {
                let current_file_name = log_utils::get_filename(&self.curr_file_path);
                let mut deleted_files = Vec::new();
                self.log_items.retain_mut(|item| {
                    if item.file_name != current_file_name && item.checked {
                        let _ = log_utils::delete_log(&item.file_name);
                        deleted_files.push(item.file_name.clone());
                        false
                    } else {
                        true
                    }
                });

                for file_name in deleted_files {
                    self.remove_historical_log_cache(&file_name);
                }
                self.prune_all = false;
                self.normalize_selection();
                self.schedule_selected_log_window(0, true)
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
                self.refresh_log_items();
                let (window_start_page, _) = self.selected_page_window();
                self.schedule_selected_log_window(window_start_page, false)
            }
            LoggingMessage::OpenManualPrune => Task::none(),
            LoggingMessage::Exit => Task::none(),
            LoggingMessage::LogWindowLoaded(request_id, file_name, state) => {
                if request_id != self.active_request_id {
                    return Task::none();
                }

                if self.log_file_selected.as_deref() != Some(file_name.as_str()) {
                    return Task::none();
                }

                if self.is_current_file(&file_name) {
                    self.current_log = state;
                } else {
                    self.historical_logs.insert(file_name, state);
                }
                self.logs_loading = false;
                self.refresh_filter_cache();
                self.clamp_page();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, LoggingMessage> {
        let level_options = LogLevel::get_options();

        let logs_header = row![
            text("Timestamp").width(Length::Fixed(310.)),
            text("Level").width(Length::FillPortion(1)),
            text("Screen").width(Length::FillPortion(1)),
            text("Message").width(Length::FillPortion(3))
        ];

        let total_log_count = self.selected_total_matching_entries();
        let page_count = total_log_count.div_ceil(LOGS_PER_PAGE);
        let page = self.log_page.min(page_count.saturating_sub(1));
        let (window_start_page, _) = self.selected_page_window();
        let page_start_idx = page.saturating_sub(window_start_page) * LOGS_PER_PAGE;

        let logs_display: Element<'_, LoggingMessage> = if self.logs_loading {
            container(
                column![
                    text("Loading logs...").size(24).center(),
                    Spinner::new()
                        .width(150.0)
                        .height(150.0)
                        .circle_radius(10.0)
                ]
                .spacing(10)
                .align_x(Horizontal::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            let mut log_cols = column![];
            let selected_entries = self.selected_entries();
            for (idx, entry_index) in self
                .filtered_log_indices
                .iter()
                .skip(page_start_idx)
                .take(LOGS_PER_PAGE)
                .enumerate()
            {
                if let Some(log) = selected_entries.get(*entry_index) {
                    log_cols = log_cols.push(log_row(log, page_start_idx + idx));
                }
            }

            Scrollable::new(log_cols)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

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
                page * LOGS_PER_PAGE + 1,
                (page * LOGS_PER_PAGE + LOGS_PER_PAGE).min(total_log_count),
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
        let total_log_count = self.selected_total_matching_entries();
        let page_count = total_log_count.div_ceil(LOGS_PER_PAGE);
        self.log_page = self.log_page.min(page_count.saturating_sub(1));
    }

    fn selected_page_window(&self) -> (usize, usize) {
        if let Some(file_path) = &self.log_file_selected {
            let state = if self.is_current_file(file_path) {
                Some(&self.current_log)
            } else {
                self.historical_logs.get(file_path)
            };
            if let Some(state) = state {
                return (
                    state.window_start_page,
                    state.total_matching_entries.div_ceil(LOGS_PER_PAGE),
                );
            }
        }
        (0, 0)
    }

    fn selected_total_matching_entries(&self) -> usize {
        if let Some(file_path) = &self.log_file_selected {
            let state = if self.is_current_file(file_path) {
                Some(&self.current_log)
            } else {
                self.historical_logs.get(file_path)
            };
            return state.map_or(0, |state| state.total_matching_entries);
        }
        0
    }

    fn selected_entries(&self) -> &[LogData] {
        if let Some(file_path) = &self.log_file_selected {
            if self.is_current_file(file_path) {
                &self.current_log.entries
            } else {
                self.historical_logs
                    .get(file_path)
                    .map_or(&[], |log| log.entries.as_slice())
            }
        } else {
            &[]
        }
    }

    fn refresh_filter_cache(&mut self) {
        self.invalidate_filter_cache();
        self.filtered_log_indices = (0..self.selected_entries().len()).collect();
    }

    fn invalidate_filter_cache(&mut self) {
        self.filtered_log_indices.clear();
    }

    fn clear_selected_log_window(&mut self, start_page: usize) {
        self.invalidate_filter_cache();
        if let Some(file_path) = self.log_file_selected.as_ref() {
            if self.is_current_file(file_path) {
                self.current_log.entries.clear();
                self.current_log.window_start_page = start_page;
            } else if let Some(log_state) = self.historical_logs.get_mut(file_path) {
                log_state.entries.clear();
                log_state.window_start_page = start_page;
            }
        }
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

    fn schedule_selected_log_window(
        &mut self,
        start_page: usize,
        force_refresh: bool,
    ) -> Task<LoggingMessage> {
        if force_refresh {
            self.invalidate_filter_cache();
        }

        let file_name = match self.log_file_selected.as_ref() {
            Some(file_name) => file_name.clone(),
            None => return Task::none(),
        };

        let path = if self.is_current_file(&file_name) {
            PathBuf::from(&self.curr_file_path)
        } else {
            self.cache_historical_log(&file_name);
            [&get_log_path(), &file_name].iter().collect()
        };

        let cached_file_size = if self.is_current_file(&file_name) {
            self.current_log.file_size
        } else {
            self.historical_logs
                .get(&file_name)
                .map_or(0, |state| state.file_size)
        };

        let file_size = general::get_file_size(&path);
        if !force_refresh && file_size == cached_file_size {
            self.logs_loading = false;
            return Task::none();
        }

        self.logs_loading = true;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        self.active_request_id = self.next_request_id;
        let request_id = self.active_request_id;
        let minimum_level = self.log_slider_idx;
        let selected_screen = self.log_selected_screen;
        let path = path.display().to_string();
        Task::perform(
            tokio::task::spawn_blocking(move || {
                load_log_window(&path, start_page, minimum_level, selected_screen)
            }),
            move |result| {
                LoggingMessage::LogWindowLoaded(request_id, file_name, result.unwrap_or_default())
            },
        )
    }

    fn cache_historical_log(&mut self, file_name: &str) {
        if self.historical_logs.contains_key(file_name) {
            return;
        }

        self.historical_log_order.push_back(file_name.to_string());

        while self.historical_log_order.len() > MAX_CACHED_HISTORICAL_LOGS {
            if let Some(oldest_file) = self.historical_log_order.pop_front() {
                self.historical_logs.remove(&oldest_file);
            }
        }
    }

    fn remove_historical_log_cache(&mut self, file_name: &str) {
        self.historical_logs.remove(file_name);
        self.historical_log_order
            .retain(|cached| cached != file_name);
    }
}

fn load_log_window(
    file_path: &str,
    start_page: usize,
    minimum_level: usize,
    selected_screen: Option<Screen>,
) -> LogState {
    let mut log_state = LogState::default();
    let file = match general::get_file(file_path) {
        Ok(file) => file,
        Err(_) => {
            return log_state;
        }
    };

    let file_size = match file.metadata().map(|metadata| metadata.len()) {
        Ok(size) => size,
        Err(_) => return log_state,
    };

    let first_entry = start_page * LOGS_PER_PAGE;
    let last_entry = first_entry + ENTRIES_PER_WINDOW;
    let mut matching_entries = Vec::new();
    let mut total_matching_entries = 0;
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        for log in parse_logs(&line) {
            let level_idx = LogLevel::get_level_idx(log.level.clone());
            let matches = minimum_level <= level_idx
                && (selected_screen == Some(Screen::All)
                    || selected_screen == Some(log.screen.clone().into()));
            if matches {
                if (first_entry..last_entry).contains(&total_matching_entries) {
                    matching_entries.push(log);
                }
                total_matching_entries += 1;
            }
        }
    }
    log_state.entries = matching_entries;
    log_state.pending_line.clear();
    log_state.file_size = file_size;
    log_state.window_start_page = start_page;
    log_state.total_matching_entries = total_matching_entries;
    log_state
}

fn get_defaut_slider_idx() -> usize {
    // TODO: Call gss-core to retrieve default log level from settings.json under app
    LogLevel::get_level_idx("INFO".into())
}

pub fn log_row<'a, M: Clone + 'static>(log: &'a LogData, index: usize) -> Element<'a, M> {
    let background = if index.is_multiple_of(2) {
        Color::from_rgb8(10, 11, 12)
    } else {
        Color::BLACK
    };

    container(
        row![
            text(log.timestamp.as_str()).width(Length::Fixed(300.0)),
            text(log.level.as_str()).width(Length::FillPortion(1)),
            text(log.screen.as_str()).width(Length::FillPortion(1)),
            text(log.message.as_str()).width(Length::FillPortion(3)),
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
