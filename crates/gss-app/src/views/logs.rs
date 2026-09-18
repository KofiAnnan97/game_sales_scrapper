use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Button, Scrollable, column, container, pick_list, row, stack, text};
use iced::{Alignment, Background, Color, Element, Length, Task};
use iced_aw::Spinner;

use files::general;

use crate::components::{custom_styles as cs, custom_widgets as cw};
use crate::log_utils::{LogData, parse_logs};
use crate::utils::log_utils;
use crate::utils::log_utils::{LogLevel, get_log_path};
use crate::views::sub_windows::LogItem;

const DEFAULT_LOGS_PER_PAGE: usize = 20;
const DEFAULT_MAX_CACHED_HISTORICAL_LOGS: usize = 5;
const DEFAULT_PAGES_PER_WINDOW: usize = 20;
const DEFAULT_ENTRIES_PER_WINDOW: usize = DEFAULT_LOGS_PER_PAGE * DEFAULT_PAGES_PER_WINDOW;

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
    LogWindowLoaded(u64, String, LogState),
    ShowLoading(u64),

    // Messages to communicate up to App
    OpenManualPrune,
    Exit,

    // Messages communicated back from App
    RefreshLogs,
    SetLogsPerPage(usize),
    SetPagesPerWindow(usize),
    SetEntriesPerWindow(usize, usize),
    SetMaxCachedHistoricalLogs(usize),
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
    displayed_log_file: Option<String>,
    log_selected_screen: Option<Screen>,
    log_page: usize,
    next_request_id: u64,
    active_request_id: u64,
    logs_loading: bool,
    show_loading: bool,
    pub log_items: Vec<LogItem>,
    pub prune_all: bool,
    // Log Cache Settings
    logs_per_page: usize,
    pages_per_window: usize,
    entries_per_window: usize,
    max_cached_historical_logs: usize,
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
            displayed_log_file: Some(file_name.clone()),
            log_selected_screen: Some(Screen::All),
            log_page: 0,
            next_request_id: 0,
            active_request_id: 0,
            logs_loading: true,
            show_loading: false,
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
            logs_per_page: get_page_length(),
            pages_per_window: get_window_size(),
            entries_per_window: get_window_entries_size(),
            max_cached_historical_logs: get_max_log_cache(),
        };
        view.refresh_filter_cache();
        view
    }

    pub fn update(&mut self, message: LoggingMessage) -> Task<LoggingMessage> {
        match message {
            LoggingMessage::LevelChanged(idx) => {
                if self.log_slider_idx == idx {
                    return Task::none();
                }

                self.log_slider_idx = idx;
                self.log_page = 0;
                self.schedule_selected_log_window(0, true)
            }
            LoggingMessage::PageChanged(page) => {
                let (window_start_page, total_pages) = self.selected_page_window();
                if window_start_page == 0 && page == self.pages_per_window
                    || page < window_start_page
                    || page > window_start_page + self.pages_per_window
                {
                    let new_start_page = page
                        .saturating_sub(self.pages_per_window / 2)
                        .min(total_pages.saturating_sub(self.pages_per_window));
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
                if self.log_selected_screen == Some(screen) {
                    return Task::none();
                }

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
                    self.historical_logs.insert(file_name.clone(), state);
                }
                self.displayed_log_file = Some(file_name);
                self.logs_loading = false;
                self.show_loading = false;
                self.refresh_filter_cache();
                self.clamp_page();
                Task::none()
            }
            LoggingMessage::ShowLoading(request_id) => {
                if request_id == self.active_request_id && self.logs_loading {
                    self.show_loading = true;
                }
                Task::none()
            }
            LoggingMessage::SetLogsPerPage(count) => {
                self.logs_per_page = count;
                Task::none()
            }
            LoggingMessage::SetPagesPerWindow(count) => {
                self.pages_per_window = count;
                Task::none()
            }
            LoggingMessage::SetEntriesPerWindow(logs_per_page, pages_per_window) => {
                self.entries_per_window = logs_per_page * pages_per_window;
                Task::none()
            }
            LoggingMessage::SetMaxCachedHistoricalLogs(count) => {
                self.max_cached_historical_logs = count;
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
        let page_count = total_log_count.div_ceil(self.logs_per_page);
        let page = self.log_page.min(page_count.saturating_sub(1));
        let (window_start_page, _) = self.selected_page_window();
        let page_start_idx = page.saturating_sub(window_start_page) * self.logs_per_page;

        let mut log_cols = column![];
        let selected_entries = self.selected_entries();
        for (idx, entry_index) in self
            .filtered_log_indices
            .iter()
            .skip(page_start_idx)
            .take(self.logs_per_page)
            .enumerate()
        {
            if let Some(log) = selected_entries.get(*entry_index) {
                log_cols = log_cols.push(log_row(log, page_start_idx + idx));
            }
        }

        let log_table: Scrollable<LoggingMessage> = Scrollable::new(log_cols)
            .width(Length::Fill)
            .height(Length::Fill);

        let logs_display: Element<'_, LoggingMessage> = if self.show_loading {
            stack![
                log_table,
                container(
                    column![
                        text("Loading Logs...").size(24).center(),
                        Spinner::new()
                            .width(150.0)
                            .height(150.0)
                            .circle_radius(10.0),
                    ]
                    .spacing(10)
                    .align_x(Horizontal::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_style| cs::darken_background(0.85))
            ]
            .into()
        } else {
            log_table.into()
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
                page * self.logs_per_page + 1,
                (page * self.logs_per_page + self.logs_per_page).min(total_log_count),
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
        let page_count = total_log_count.div_ceil(self.logs_per_page);
        self.log_page = self.log_page.min(page_count.saturating_sub(1));
    }

    fn selected_page_window(&self) -> (usize, usize) {
        if let Some(file_path) = &self.displayed_log_file {
            let state = if self.is_current_file(file_path) {
                Some(&self.current_log)
            } else {
                self.historical_logs.get(file_path)
            };
            if let Some(state) = state {
                return (
                    state.window_start_page,
                    state.total_matching_entries.div_ceil(self.logs_per_page),
                );
            }
        }
        (0, 0)
    }

    fn selected_total_matching_entries(&self) -> usize {
        if let Some(file_path) = &self.displayed_log_file {
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
        if let Some(file_path) = &self.displayed_log_file {
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
        self.filtered_log_indices = (0..self.selected_entries().len()).collect();
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
        if self.logs_loading && self.active_request_id != 0 && !force_refresh {
            return Task::none();
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
            self.show_loading = false;
            self.refresh_filter_cache();
            self.clamp_page();
            return Task::none();
        }

        self.logs_loading = true;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        self.active_request_id = self.next_request_id;
        let request_id = self.active_request_id;
        let minimum_level = self.log_slider_idx;
        let selected_screen = self.log_selected_screen;
        let path = path.display().to_string();

        let logs_per_page = self.logs_per_page;
        let entries_per_window = self.entries_per_window;
        Task::batch([
            Task::perform(
                async {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                },
                move |_| LoggingMessage::ShowLoading(request_id),
            ),
            Task::perform(
                tokio::task::spawn_blocking(move || {
                    load_log_window(
                        &path,
                        start_page,
                        minimum_level,
                        selected_screen,
                        logs_per_page,
                        entries_per_window,
                    )
                }),
                move |result| {
                    LoggingMessage::LogWindowLoaded(
                        request_id,
                        file_name,
                        result.unwrap_or_default(),
                    )
                },
            ),
        ])
    }

    fn cache_historical_log(&mut self, file_name: &str) {
        if self.historical_logs.contains_key(file_name) {
            return;
        }

        self.historical_log_order.push_back(file_name.to_string());

        while self.historical_log_order.len() > self.max_cached_historical_logs {
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
    logs_per_page: usize,
    entries_per_window: usize,
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

    let first_entry = if start_page > 0 {
        (start_page - 1) * logs_per_page
    } else {
        0
    };
    let last_entry = if first_entry == 0 {
        first_entry + entries_per_window
    } else {
        first_entry + entries_per_window + logs_per_page
    };
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
    LogLevel::get_level_idx("DEBUG".into())
}

fn get_page_length() -> usize {
    // TODO: retrieve value from settings json or use default
    DEFAULT_LOGS_PER_PAGE
}

// Assumes that the window size is an even number
fn get_window_size() -> usize {
    // TODO: retrieve value from settings json or use default
    DEFAULT_PAGES_PER_WINDOW
}

fn get_max_log_cache() -> usize {
    // TODO: retrieve value from settings json or use default
    DEFAULT_MAX_CACHED_HISTORICAL_LOGS
}

fn get_window_entries_size() -> usize {
    // TODO: retrieve value from settings json or use default
    DEFAULT_ENTRIES_PER_WINDOW
}

pub fn log_row<'a, M: Clone + 'a>(log: &'a LogData, index: usize) -> Element<'a, M> {
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
