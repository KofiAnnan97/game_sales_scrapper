use std::path::PathBuf;
use std::path::Path;
use std::sync::Arc;
use std::io;
use std::collections::{HashMap, HashSet};

use iced::widget::{Button, column, container, row, text};
use iced::{Element, Length, Padding, Subscription, Task, window, exit, daemon};
use iced::time::{self, Duration};
use iced_aw::menu::{self, Menu};
use iced_aw::{ICED_AW_FONT_BYTES, menu_bar, menu_items, TabLabel, tabs::{Tabs, TabBarPosition}};

// Common internal libraries
use files::{csv, general};
use file_ops::{settings, thresholds};
use properties;
use types::internal::data::{GameThreshold, SimpleGameThreshold};
use types::internal::store::GameStore;

// App specific modules
mod views;
mod tabs;
mod components;
mod utils;

use tabs::{thresholds as thrshlds_view, search::SKIP_STORE_SELECTION};
use views::settings as sttngs_view;
use components::{custom_widgets as cw};
use utils::actions_utils::{send_sales_email, update_cache};
use utils::search_utils::perform_store_search;
use utils::file_utils::open_file;
use utils::log_utils::{self, LogLevel};
use crate::views::logs as logs_view;
use crate::views::sub_windows::LogItem;
use crate::utils::log_utils::Logger;
use crate::views::logs::{Screen}; //LogPane, LoggingMessage, LoggingView, Screen};
use crate::views::preview::{PreviewMessage, PreviewView};
use crate::views::settings::{Page, alias_settings, store_selection};
use crate::views::sub_windows::manual_prune;

const LOADING_FRAMES_SIZE : usize = 4;

const STATUS_ERR : &str = "ERROR";

fn main() -> iced::Result {
    let log_file = log_utils::new_log();
    let log_file_clone = log_file.clone();  
    std::panic::set_hook(Box::new(move |panic_info| {
        let panic_msg = log_utils::fatal_message_builder(panic_info);
        general::append_to_file(&log_file_clone, &panic_msg);
    }));

    daemon(move || App::new(log_file.clone()), App::update, App::view)
        .title("Game Sales Scrapper")
        .subscription(App::subscription)
        .font(ICED_AW_FONT_BYTES)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tab {
    Search,
    Thresholds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Base,
    Settings,
    Preview,
    Logs,
}

#[derive(Debug, Clone)]
enum StoreSearchResult {
    Steam { title: String, steam_id: u32 },
    Gog { title: String, gog_id: u32 },
    Microsoft { title: String, ms_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Title,
    Alias,
    SteamId,
    GogId,
    MicrosoftId,
    DesiredPrice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortOrder {
    Original,
    Ascending,
    Descending,
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

#[derive(Debug, Clone)]
pub(crate) enum MainMessage {
    MainWindowOpened(window::Id),
    TabSelected(Tab),
    OpenSettings,
    CloseSettings,
    OpenSalesPreview,
    CloseSalesPreview,
    ToggleStore(GameStore, bool),
    ToggleAliasEnabled(bool),
    ToggleAliasReuse(bool),
    ProjectPathChanged(String),
    TestPathChanged(String),
    SteamApiKeyChanged(String),
    RecipientEmailChanged(String),
    SmtpHostChanged(String),
    SmtpPortChanged(String),
    SmtpEmailChanged(String),
    SmtpUserChanged(String),
    SmtpPasswordChanged(String),
    ToggleTestMode(bool),
    ToggleSensitiveData(bool),
    SaveSettings,
    SearchQueryChanged(String),
    StartSearch,
    SelectAllStores,
    SelectNoStores,
    SearchResultSelected(usize),
    OpenCsv,
    CsvOpened(Result<(PathBuf, Arc<String>), Error>),
    ExecuteBulkInsert,
    StoreSearchCompleted(GameStore, Result<Vec<StoreSearchResult>, String>),
    SearchReset,
    NextStore,
    PreviousStore,
    AddThreshold,
    ThresholdAliasChanged(usize, String),
    ThresholdPriceChanged(usize, String),
    UpdateThresholdRow(usize),
    RemoveThresholdRow(usize),
    SortThresholds(SortColumn),
    SendEmailResult(Result<String, String>),
    UpdateCache,
    UpdateCacheResult(Result<String, String>),
    Tick,
    Refresh,
    HideDialog,
    //Settings Messages
    StoreSettingsExpanded(bool),
    PageSelected(Page),
    // Log Messages
    UpdateLogFile,
    OpenLogsView,
    CloseLogsView,
    LevelChanged(usize),
    LogFileChanged(Option<String>),
    LogScreenChanged(Screen),
    LogEvent(Screen, LogLevel, String),
    OpenManualPrune,
    ToggleLogsToRemove(usize, bool),
    DeleteLogs,
    PruneAllLogs(bool),
}

#[derive(Debug, Clone)]
pub(crate) enum Message{
    CloseWindow(window::Id),
    Main(MainMessage),
    Preview(PreviewMessage),
    // Logging(LoggingMessage)
}

impl From<MainMessage> for Message {
    fn from(msg: MainMessage) -> Self {
        Message::Main(msg)
    }
}

impl From<PreviewMessage> for Message {
    fn from(msg: PreviewMessage) -> Self {
        Message::Preview(msg)
    }
}

// impl From<LoggingMessage> for Message {
//     fn from(msg: LoggingMessage) -> Self { Message::Logging(msg) }
// }

impl StoreSearchResult {
    fn title(&self) -> &str {
        match self {
            StoreSearchResult::Steam { title, .. } => title,
            StoreSearchResult::Gog { title, .. } => title,
            StoreSearchResult::Microsoft { title, .. } => title,
        }
    }

    fn ids(&self) -> (u32, u32, String) {
        match self {
            StoreSearchResult::Steam { steam_id, .. } => (*steam_id, 0, String::new()),
            StoreSearchResult::Gog { gog_id, .. } => (0, *gog_id, String::new()),
            StoreSearchResult::Microsoft { ms_id, .. } => (0, 0, ms_id.clone()),
        }
    }
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Search => "Search",
            Tab::Thresholds => "Thresholds",
        }
    }
}

struct App {
    main_window: Option<window::Id>,
    tab: Tab,
    active_view: View,
    settings_view_open: bool,
    preview_view_open: bool,
    logger: Logger,
    // Settings
    available_stores: Vec<GameStore>,
    selected_stores: Vec<GameStore>,
    alias_enabled: bool,
    alias_reuse_enabled: bool,
    reveal_sensitive_data: bool,
    steam_api_key: String,
    recipient_email: String,
    smtp_host: String,
    smtp_port: String,
    smtp_email: String,
    smtp_user: String,
    smtp_password: String,
    project_path: String,
    test_path: String,
    test_mode: bool,
    auto_advance_enabled: bool,
    //Search
    search_query: String,
    add_alias: String,
    add_price: String,
    search_results_by_store: Vec<(GameStore, Vec<StoreSearchResult>)>,
    current_store_search_idx: usize,
    selected_results_by_store: HashMap<GameStore, Option<usize>>,
    is_search_in_progress: bool,
    is_caching_in_progress: bool,
    pending_searches: usize,
    search_loading_frame: usize,
    caching_loading_frame: usize,
    selected_file: Option<PathBuf>,
    bulk_simple_threshs: Vec<SimpleGameThreshold>,
    bulk_search_index: usize,
    bulk_search_used: bool,
    thresholds: Vec<GameThreshold>,
    threshold_alias_edits: Vec<String>,
    threshold_price_edits: Vec<String>,
    threshold_sort_column: Option<SortColumn>,
    threshold_sort_order: SortOrder,
    show_dialog: bool,
    preview_view: PreviewView,
    //logging_view: LoggingView,
    // Settings variables
    settings_page: Page,
    store_settings_expanded: bool,
    // Logging variables
    status_message: String,
    message_details: String,
    logs_view_open: bool,
    log_slider_idx: usize,
    log_file_selected: Option<String>,
    log_selected_screen: Option<Screen>,
    manual_prune_window: Option<window::Id>,
    manual_prune_open: bool,
    log_items: Vec<LogItem>,
    prune_all: bool,
}

impl App {
    fn new(log_file: String) -> (Self, Task<Message>) {
        let (id, task) = window::open(window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            position: window::Position::Centered,
            resizable: true,
            ..Default::default()
        });
        
        let mut app = Self {
            main_window: Some(id),
            tab: Tab::Search,
            active_view: View::Base,
            settings_view_open: false,
            preview_view_open: false,
            logger: Logger::new(&log_file),
            // Settings
            available_stores: settings::get_available_stores(),
            selected_stores: settings::get_selected_stores(),
            alias_enabled: settings::get_alias_state(),
            alias_reuse_enabled: settings::get_alias_reuse_state(),
            reveal_sensitive_data: false,
            steam_api_key: properties::get_steam_api_key(true),
            recipient_email: properties::get_recipient(),
            smtp_host: properties::get_smtp_host(),
            smtp_port: properties::get_smtp_port().to_string(),
            smtp_email: properties::get_smtp_email(),
            smtp_user: properties::get_smtp_user(),
            smtp_password: properties::get_smtp_pwd(true),
            project_path: properties::get_project_path(),
            test_path: properties::get_test_path(),
            test_mode: properties::is_testing_enabled(),
            // TODO: Add auto advance to settings under app
            auto_advance_enabled: false,
            // Search
            search_query: String::new(),
            add_alias: String::new(),
            add_price: String::new(),
            search_results_by_store: Vec::new(),
            current_store_search_idx: 0,
            selected_results_by_store: HashMap::new(),
            is_search_in_progress: false,
            is_caching_in_progress: false,
            pending_searches: 0,
            search_loading_frame: 0,
            caching_loading_frame: 0,
            selected_file: None,
            bulk_simple_threshs: Vec::new(),
            bulk_search_index: 0,
            bulk_search_used: false,
            thresholds: thresholds::load_thresholds().unwrap_or_default(),
            threshold_alias_edits: Vec::new(),
            threshold_price_edits: Vec::new(),
            threshold_sort_column: None,
            threshold_sort_order: SortOrder::Original,
            show_dialog: false,
            preview_view: PreviewView::default(),
            //logging_view: LoggingView::new(&log_file),
            // Settings view variables
            settings_page: Page::General,
            store_settings_expanded: false,
            //Logging view variables
            status_message: String::from("Ready"),
            message_details: String::new(),
            logs_view_open: false,
            log_slider_idx: 0,
            log_file_selected: Some(log_utils::get_filename(&log_file)),
            log_selected_screen: Some(Screen::All),
            manual_prune_window: None,
            manual_prune_open: false,
            log_items: {
                let avaialable_logs = log_utils::get_app_logs();
                let mut items: Vec<LogItem> = Vec::new();
                for (idx, file_name) in avaialable_logs.iter().enumerate() {
                    if idx != 0 {
                        items.push(LogItem { id: idx, name: file_name.clone(), checked: false });
                    }
                }
                items
            },
            prune_all: false
        };
        
        app.sync_threshold_edits();
        (
            app,
            task.map(|id| MainMessage::MainWindowOpened(id).into()),
        )
    }
}

impl Default for App {
    fn default() -> App {
        Self::new(log_utils::new_log()).0
    }
}

impl App {
    fn subscription(&self) -> Subscription<Message> {
        let log_sub = time::every(Duration::from_secs(15)).map(|_| MainMessage::UpdateLogFile.into());
        let main_tick_sub = time::every(Duration::from_millis(400)).map(|_| MainMessage::Tick.into());
        let preview_tick_sub = time::every(Duration::from_millis(400))
            .map(|_| {Message::Preview(PreviewMessage::Tick)});
        let close_sub = iced::event::listen_with(|event, _, window_id| {
            match event {
                iced::Event::Window(
                    iced::window::Event::Closed
                ) => {
                    Some(Message::CloseWindow(window_id))
                }
                _ => None,
            }
        });
        
        Subscription::batch(vec![log_sub, main_tick_sub, preview_tick_sub, close_sub])
    }

    fn update_main(&mut self, message: MainMessage) -> Task<Message>{
        match message {
            MainMessage::MainWindowOpened(id) => {
                self.main_window = Some(id);
                Task::none()
            }
            MainMessage::TabSelected(tab) => {
                self.tab = tab;
                let screen = match self.tab {
                    Tab::Search => Screen::Search,
                    Tab::Thresholds => Screen::Thresholds,
                };
                self.logger.debug(screen, &format!("Showing {}", tab.label()));
                Task::none()
            }
            MainMessage::ToggleStore(store, enabled) => {
                if enabled {
                    if !self.selected_stores.contains(&store) {
                        self.selected_stores.push(store.clone());
                    }
                } else {
                    self.selected_stores.retain(|id| id != &store);
                }
                settings::update_selected_stores(self.selected_stores.clone());
               self.logger.info(Screen::Search, "Updated selected stores");
                Task::none()
            }
            MainMessage::ToggleAliasEnabled(enabled) => {
                self.alias_enabled = enabled;
                settings::update_alias_state(if enabled { 1 } else { 0 });
                self.logger.info(Screen::Search, &format!("Alias enabled: {}", self.alias_enabled));
                Task::none()
            }
            MainMessage::ToggleAliasReuse(enabled) => {
                self.alias_reuse_enabled = enabled;
                settings::update_alias_reuse_state(if enabled { 1 } else { 0 });
                self.logger.info(Screen::Settings, &format!("Alias reuse enabled: {}", self.alias_reuse_enabled));
                Task::none()
            }
            MainMessage::OpenSettings => {
                self.settings_view_open = true;
                self.active_view = View::Settings;
                self.settings_page = Page::General;
                self.logger.info(Screen::Settings, "Switching to Settings view");
                Task::none()
            }
            MainMessage::CloseSettings => {
                self.settings_view_open = false;
                self.active_view = View::Base;
                self.show_dialog = false;
                self.logger.info(Screen::Settings, "Closed settings view");
                Task::none()
            }
            MainMessage::OpenSalesPreview => {
                self.active_view = View::Preview;
                self.preview_view_open = true;
                self.logger.info(Screen::Sales, "Open sales view");
                Task::done(Message::Preview(PreviewMessage::ResetToSales))
            }
            MainMessage::CloseSalesPreview => {
                self.active_view = View::Base;
                self.preview_view_open = false;
                self.logger.info(Screen::Sales, "Close sales view");
                Task::none()
            }
            MainMessage::SortThresholds(column) => {
                if self.threshold_sort_column == Some(column) {
                    self.threshold_sort_order = match self.threshold_sort_order {
                        SortOrder::Original => SortOrder::Ascending,
                        SortOrder::Ascending => SortOrder::Descending,
                        SortOrder::Descending => SortOrder::Original,
                    };
                    if self.threshold_sort_order == SortOrder::Original {
                        self.threshold_sort_column = None;
                    }
                } else {
                    self.threshold_sort_column = Some(column);
                    self.threshold_sort_order = SortOrder::Ascending;
                }
                
                let col_name = match column {
                    SortColumn::Title => "Title",
                    SortColumn::Alias => "Alias",
                    SortColumn::SteamId => "Steam ID",
                    SortColumn::GogId => "GOG ID",
                    SortColumn::MicrosoftId => "Microsoft Store ID",
                    SortColumn::DesiredPrice => "Price",
                };

                let status_str = match self.threshold_sort_order {
                    SortOrder::Original => format!("Threshold column '{}' is in original order", col_name),
                    SortOrder::Ascending => format!("Threshold column '{}' is in ascending order", col_name),
                    SortOrder::Descending => format!("Threshold column '{}' is in descending order", col_name),
                };
                self.logger.info(Screen::Search, &status_str);
                Task::none()
            }
            MainMessage::ProjectPathChanged(value) => { self.project_path = value; Task::none() }
            MainMessage::TestPathChanged(value) => { self.test_path = value; Task::none() }
            MainMessage::SteamApiKeyChanged(value) => { self.steam_api_key = value; Task::none() }
            MainMessage::RecipientEmailChanged(value) => { self.recipient_email = value; Task::none() }
            MainMessage::SmtpHostChanged(value) => { self.smtp_host = value; Task::none() }
            MainMessage::SmtpPortChanged(value) => { self.smtp_port = value; Task::none() }
            MainMessage::SmtpEmailChanged(value) => { self.smtp_email = value; Task::none() }
            MainMessage::SmtpUserChanged(value) => { self.smtp_user = value; Task::none() }
            MainMessage::SmtpPasswordChanged(value) => { self.smtp_password = value; Task::none() }
            MainMessage::ToggleSensitiveData(reveal) => { self.set_reveal_sensitive_data(reveal); Task::none() }
            MainMessage::ToggleTestMode(enabled) => { self.test_mode = enabled; Task::none() }
            MainMessage::SearchQueryChanged(value) => { self.search_query = value; Task::none() }
            MainMessage::StartSearch => {
                self.bulk_search_used = false;
                self.start_game_search(self.search_query.clone())
            }
            MainMessage::SelectAllStores => {
                self.selected_stores = self.available_stores.clone();
                settings::update_selected_stores(self.selected_stores.clone());
                Task::none()
            }
            MainMessage::SelectNoStores => {
                self.selected_stores.clear();
                settings::clear_selected_stores();
                Task::none()
            }
            MainMessage::OpenCsv => {
                if self.is_search_in_progress {
                    Task::none()
                } else {
                    self.is_search_in_progress = true;
                    window::oldest()
                        .and_then(|id| window::run(id, open_file))
                        .then(Task::future)
                        .map(|file| Message::Main(MainMessage::CsvOpened(file)))
                }
            }
            MainMessage::CsvOpened(result) => {
                self.is_search_in_progress = false;
                if let Ok((path, contents)) = result {
                    self.selected_file = Some(path);
                    match &self.selected_file {
                        Some(path) => {
                            let data = Arc::try_unwrap(contents).unwrap_or_default();
                            let log_msg;
                            if !data.is_empty() {
                                self.bulk_simple_threshs = csv::parse_game_prices_from_str(&data).unwrap_or_default();
                                self.bulk_search_index = 0;
                                self.bulk_search_used = !self.bulk_simple_threshs.is_empty();
                                if self.bulk_simple_threshs.is_empty() {
                                    log_msg = format!("Could not convert {:?} to the Simple Game Threshold format. Check that csv file is properly formatted.", path.display().to_string());
                                    self.logger.error(Screen::Search, &log_msg);
                                } else {
                                    log_msg = format!("CSV data: {:?}", &self.bulk_simple_threshs);
                                    self.logger.info(Screen::Search, &log_msg);
                                }
                            } else {
                                log_msg = format!("{} is empty", path.display().to_string());
                                self.logger.debug(Screen::Search, &log_msg);
                            }
                        }, 
                        None => ()
                    };
                }
                Task::none()
            }
            MainMessage::Tick => {
                if self.is_search_in_progress {
                    self.search_loading_frame = (self.search_loading_frame + 1) % LOADING_FRAMES_SIZE;
                }
                if self.is_caching_in_progress {
                    self.caching_loading_frame = (self.search_loading_frame + 1) % LOADING_FRAMES_SIZE;
                }
                Task::none()
            }
            MainMessage::ExecuteBulkInsert => {
                self.bulk_search_used = true;
                self.bulk_search_index = 0;

                if self.bulk_simple_threshs.is_empty() {
                    self.logger.warn(Screen::Search, "No games loaded from CSV.");
                    return Task::none();
                }

                if let Some(game) = self.current_bulk_game() {
                    self.search_query = game.name.clone();
                    self.add_price = game.price.to_string();
                    self.start_game_search(self.search_query.clone())
                } else {
                    self.logger.warn(Screen::Search, "No games loaded from CSV.");
                    Task::none()
                }
            }
            MainMessage::SearchResultSelected(selected) => {
                if let Some((game_store, _)) = self.search_results_by_store.get(self.current_store_search_idx) {
                    if selected == SKIP_STORE_SELECTION {
                        self.selected_results_by_store.insert(game_store.clone(), None);
                    } else {
                        self.selected_results_by_store.insert(game_store.clone(), Some(selected));
                    }
                }
                if self.auto_advance_enabled {
                    return Task::done(MainMessage::NextStore.into());
                }
                Task::none()
            }
            MainMessage::StoreSearchCompleted(game_store, result) => {
                if let Some(entry) = self.search_results_by_store.iter_mut().find(|(id, _)| id == &game_store) {
                    match result {
                        Ok(list) => {
                            entry.1 = list;
                            self.logger.info(Screen::Search, &format!("Search completed for {:?}", game_store));
                        }
                        Err(err) => {
                            self.logger.error(Screen::Search, &format!("Search failed for {}: {:?}", game_store.get_name(), err));
                        }
                    }
                }
                if self.pending_searches > 0 {
                    self.pending_searches -= 1;
                }
                if self.pending_searches == 0 {
                    self.is_search_in_progress = false;
                    self.search_loading_frame = 0;
                }
                Task::none()
            }
            MainMessage::SearchReset => {
                self.search_query.clear();
                self.add_price.clear();
                self.add_alias.clear();
                self.search_results_by_store.clear();
                self.bulk_search_used = false;
                self.bulk_search_index = 0;
                self.bulk_simple_threshs.clear();
                self.logger.info(Screen::Search, "Reset search results");
                Task::none()
            }
            MainMessage::NextStore => {
                if self.current_store_search_idx < self.search_results_by_store.len() - 1 {
                    self.current_store_search_idx += 1;
                    let (game_store, results) = &self.search_results_by_store[self.current_store_search_idx];
                    if results.is_empty() {
                        let query = self.search_query.clone();
                        self.logger.info(Screen::Search, &format!("Search {}", game_store.get_name()));
                        let game_store_clone = game_store.clone();
                        return Task::perform(perform_store_search(query, game_store_clone), move |result| {
                            MainMessage::StoreSearchCompleted(game_store_clone, result).into()
                        });
                    }
                } else {
                    self.logger.info(Screen::Search, "Reached last store.");
                }
                Task::none()
            }
            MainMessage::PreviousStore => {
                if self.current_store_search_idx > 0 {
                    self.current_store_search_idx -= 1;
                    let (store_id, _) = &self.search_results_by_store[self.current_store_search_idx];
                    self.logger.info(Screen::Search, &&format!("Viewing results from {}", store_id));
                }
                Task::none()
            }
            MainMessage::AddThreshold => {
                let price = match self.add_price.trim().parse::<f64>() {
                    Ok(price) => price,
                    Err(_) => {
                        self.logger.error(Screen::Search, "Invalid desired price. Must be a decimal or integer value");
                        return Task::none();
                    }
                };

                let alias = self.add_alias.trim().to_string();
                let mut thresholds_list = thresholds::load_thresholds().unwrap_or_default();
                let mut added_titles = HashSet::new();
                let mut added_count = 0;

                for (store_id, results) in self.search_results_by_store.iter() {
                    if let Some(selected_idx) = self.selected_results_by_store.get(store_id).and_then(|opt| *opt) {
                        if let Some(result) = results.get(selected_idx) {
                            let title = result.title();
                            let (steam_id, gog_id, ms_id) = result.ids();

                            if added_titles.insert(title.to_string()) {
                                Self::insert_threshold(
                                    &mut thresholds_list,
                                    title,
                                    &alias,
                                    price,
                                    steam_id,
                                    gog_id,
                                    ms_id,
                                );
                                added_count += 1;
                            } else {
                                if let Some(existing) = thresholds_list.iter_mut().find(|existing| existing.title == title) {
                                    if steam_id != 0 {
                                        existing.steam_id = steam_id;
                                    }
                                    if gog_id != 0 {
                                        existing.gog_id = gog_id;
                                    }
                                    if !ms_id.is_empty() {
                                        existing.microsoft_store_id = ms_id.clone();
                                    }
                                }
                            }
                        }
                    }
                }

                if added_count == 0 {
                    self.logger.info(Screen::Search, "No selected storefront titles were added. Choose a result before adding a threshold.");
                } else {
                    thresholds::update_thresholds(thresholds_list);
                    added_titles.iter()
                        .for_each(|title| { thresholds::update_threshold_alias(title.to_owned(), &alias); });
                    self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                    self.sync_threshold_edits();
                    self.search_results_by_store.clear();
                    self.selected_results_by_store.clear();
                    self.current_store_search_idx = 0;

                    if self.bulk_search_used {
                        if let Some(next_game) = self.next_bulk_game() {
                            self.search_query = next_game.name.clone();
                            self.add_price = next_game.price.to_string();
                            self.add_alias.clear();
                            
                            self.logger.info(
                                Screen::Search,
                                &format!("Added threshold for '{}'. Moving onto the next game {}.", &self.search_query, next_game.name)
                            );
                            return self.start_game_search(self.search_query.clone());
                        }
                        self.bulk_search_used = false;
                        self.logger.info(Screen::Search, "Finished processing games from CSV file.")
                    }
                }
                self.add_alias.clear();
                self.add_price.clear();

                Task::none()
            }
            MainMessage::SaveSettings => {
                if self.settings_page == Page::General {
                    if !self.project_path.is_empty() && Path::new(&self.project_path).is_dir() {
                        properties::set_project_path(&self.project_path);
                    }
                    if !self.test_path.is_empty() && Path::new(&self.test_path).is_dir() {
                        properties::set_test_path(&self.test_path);
                    }
                    properties::set_test_mode(self.test_mode);
                    self.logger.info(Screen::Settings, "Saving general settings");
                } else if self.settings_page == Page::Email {  
                    if !self.recipient_email.is_empty() {
                        properties::set_recipient(&self.recipient_email);
                    }
                    let smtp_port = self.smtp_port.parse::<u16>().unwrap_or(0);
                    if smtp_port != 0 || !self.smtp_host.is_empty() || !self.smtp_email.is_empty() || !self.smtp_user.is_empty() || !self.smtp_password.is_empty() {
                        properties::set_stmp_vars(
                            self.smtp_host.clone(),
                            smtp_port,
                            self.smtp_email.clone(),
                            self.smtp_user.clone(),
                            if self.reveal_sensitive_data { self.smtp_password.clone() } else { String::new() },
                        );
                    }
                    self.logger.info(Screen::Settings, "Saving email settings");
                } else if self.settings_page == Page::Stores(GameStore::STEAM) {
                    if self.reveal_sensitive_data && !self.steam_api_key.is_empty() {
                        properties::set_steam_api_key(self.steam_api_key.clone());
                    }
                    self.logger.info(Screen::Settings, "Saving Steam settings");
                }

                self.show_dialog = true;
                Task::none()
            }
            MainMessage::SendEmailResult(result) => {
                let level;
                let details;
                let (level, details) = match result {
                    Ok(success) => {
                        self.logger.info(Screen::Sales, &success);
                        level = "INFO".into();
                        details = "Email request has been successfully sent.".into();
                        (level, details)
                    }
                    Err(err) => {
                        self.logger.error(Screen::Sales, &err);
                        level = STATUS_ERR.into();
                        details = "An issue occured trying send an email. Please check your email settings or connection.".into();
                        (level, details)
                    }
                };
                if self.active_view == View::Preview {
                    Task::done(PreviewMessage::GetEmailResult(level, details).into())
                } else {
                    self.show_dialog = true;
                    self.status_message = level;
                    self.message_details = details;
                    Task::none()
                }
            }
            MainMessage::UpdateCache => {
                self.show_dialog = false;
                self.is_caching_in_progress = true;
                self.logger.info(Screen::Settings, "Updating cache ...");
                Task::perform(update_cache(), |result| MainMessage::UpdateCacheResult(result).into())
            }
            MainMessage::UpdateCacheResult(result) => {
                self.is_caching_in_progress = false;
                match result {
                    Ok(output) => {
                        self.logger.info(Screen::Settings, &output);
                        self.logger.info(Screen::Settings, "Cache update complete");
                    }
                    Err(err) => {
                        self.logger.error(Screen::Settings, &format!("Cache update failed, {:?}", err));
                        self.status_message = STATUS_ERR.into();
                        self.message_details = "An issue occurred trying to cache game titles. Please check your internet connection or try again later.".into();
                        self.show_dialog = true;
                    }
                }
                Task::none()
            }
            MainMessage::ThresholdAliasChanged(idx, value) => {
                if idx == usize::MAX {
                    self.add_alias = value;
                } else if let Some(slot) = self.threshold_alias_edits.get_mut(idx) {
                    *slot = value;
                }
                Task::none()
            }
            MainMessage::ThresholdPriceChanged(idx, value) => {
                if idx == usize::MAX {
                    self.add_price = value;
                } else if let Some(slot) = self.threshold_price_edits.get_mut(idx) {
                    *slot = value;
                }
                Task::none()
            }
            MainMessage::UpdateThresholdRow(idx) => {
                if let Some(threshold) = self.thresholds.get(idx) {
                    let alias = self.threshold_alias_edits.get(idx).cloned().unwrap_or_default();
                    let price_str = self.threshold_price_edits.get(idx).cloned().unwrap_or_default();
                    if alias != threshold.alias {
                        thresholds::update_threshold_alias(threshold.title.clone(), &alias);
                    }
                    if let Ok(price) = price_str.trim().parse::<f64>() {
                        if threshold.alias.is_empty() {
                            let _ = thresholds::update_price(&threshold.title, price);
                        } else {
                            let _ = thresholds::update_price(&threshold.alias, price);
                        } 
                    }
                }
                self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                self.sync_threshold_edits();
                self.logger.info(Screen::Thresholds, "Threshold row updated");
                Task::none()
            }
            MainMessage::RemoveThresholdRow(idx) => {
                if let Some(title) = self.thresholds.get(idx).map(|threshold| threshold.title.clone()) {
                    let _ = thresholds::remove(&title);
                    self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                    self.sync_threshold_edits();
                    self.logger.info(Screen::Thresholds, & format!("Removed threshold {}.", title));
                }
                Task::none()
            }
            MainMessage::Refresh => {
                self.refresh_state();
                self.logger.debug(Screen::None, "Refreshed state");
                Task::none()
            }
            MainMessage::UpdateLogFile => {
               self.logger.flush();
                Task::none()
            }
            MainMessage::HideDialog => { 
                self.show_dialog = false;                 self.is_caching_in_progress = true;

                if self.status_message.eq_ignore_ascii_case(STATUS_ERR) {
                    self.message_details.clear();
                    self.status_message.clear();
                }
                Task::none() 
            }
            MainMessage::StoreSettingsExpanded(is_expanded) => {
                self.store_settings_expanded = is_expanded;
                Task::none()
            }
            MainMessage::PageSelected(selected) => {
                if self.active_view != View::Settings {
                    self.settings_view_open = true;
                    self.active_view = View::Settings;
                }
                self.settings_page = selected;
                Task::none()
            }
            MainMessage::OpenLogsView => {
                self.active_view = View::Logs;
                self.logs_view_open = true;
                self.logger.debug(Screen::Logs, "Open logs view");
                Task::none()
            }
            MainMessage::CloseLogsView => {
                self.active_view = View::Base;
                self.logs_view_open = false;
                self.logger.debug(Screen::Logs, "Closed Logs view");
                Task::none()
            }
            MainMessage::LevelChanged(idx) => {
                self.log_slider_idx = idx;
                Task::none()
            }
            MainMessage::LogFileChanged(dt_str) => {
                self.log_file_selected = dt_str;
                Task::none()
            }
            MainMessage::LogScreenChanged(screen) => {
                self.log_selected_screen = Some(screen);
                Task::none()
            }
            MainMessage::LogEvent(screen, level, msg) => {
                self.logger.log(screen, level, &msg);
                Task::none()
            }
            MainMessage::OpenManualPrune => {
                if self.manual_prune_open {
                    return Task::none();
                }

                self.manual_prune_open = true;

                let (id, task) = window::open(window::Settings {
                    size: iced::Size::new(600.0, 800.0),
                    resizable: true,
                    ..Default::default()
                });

                self.manual_prune_window = Some(id);

                return task.map(|_| MainMessage::LogEvent(Screen::Logs, LogLevel::DEBUG, "Opening Manual Prune Window".into()).into());
            }
            MainMessage::ToggleLogsToRemove(id, checked) => {
                if !checked {
                    self.prune_all = false;
                }

                if let Some(item) = self.log_items.iter_mut().find(|item| item.id == id) {
                    item.checked = checked;
                }
                Task::none()
            }
            MainMessage::DeleteLogs => {
                self.log_items.retain_mut(|item| {
                    if item.checked {
                        let _ = log_utils::delete_log(&item.name);
                        false
                    } else {
                        true
                    }
                });
                Task::done(Message::CloseWindow(self.manual_prune_window.unwrap()))
            }
            MainMessage::PruneAllLogs(toggle) => {
                if toggle {
                    self.prune_all = true;
                    for item in self.log_items.iter_mut() {
                        item.checked = true;
                    }
                }
                Task::none()
            }
        }
    }
    
    fn update_preview(&mut self, message: PreviewMessage) -> Task<Message> {
        match message {
            PreviewMessage::Exit => {
                self.active_view = View::Base;
                self.preview_view_open = false;
                Task::none()
            }

            PreviewMessage::SendEmail => {
                self.logger.info(Screen::Sales, "Attempting to send email.");
                Task::perform(send_sales_email(), |result| MainMessage::SendEmailResult(result).into())
            }
            PreviewMessage::OpenEmailSettings => {
                self.settings_view_open = true;
                self.active_view = View::Settings;
                self.settings_page = Page::Email;
                Task::none()
            }
            PreviewMessage::SendLogEvent(level, msg) => {
                self.logger.log(Screen::Sales, level, &msg);
                Task::none()
            }
            preview_message => {
                self.preview_view.update(preview_message).map(Message::Preview)
            }
        }
    }
    
    // fn update_logging(&mut self, message: LoggingMessage) -> Task<Message> {
    //     match message {
    //         LoggingMessage::Exit => {
    //             self.active_view = View::Base;
    //             self.logs_view_open = false;
    //             self.logger.debug(Screen::Logs, "Closed Logs view");
    //             Task::none()
    //         }
    //         logging_message => {
    //             //self.logging_view.update(logging_message).map(Message::Logging)
    //             Task::none()
    //         }
    //     }
    // }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseWindow(id) => {
                if Some(id) == self.main_window {
                    self.logger.info(Screen::None, "Application successfully exited.");
                    self.logger.flush();
                    exit()
                } else if Some(id) == self.manual_prune_window {
                    self.manual_prune_open = false;
                    self.manual_prune_window = None;
                   window::close(id) 
                } else {
                    Task::none()
                }
            }
            Message::Main(message) => self.update_main(message),
            Message::Preview(message) => self.update_preview(message),
            // Message::Logging(message) => self.update_logging(message),
            // _ => Task::none()
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.manual_prune_window {
            return manual_prune(&self);
        }

        let quick_file_menu = Menu::new(menu_items!(
            (text("Selected Stores").size(20)),
            (store_selection(self)),
            (row![
                Button::new(text("Select None"))
                    .on_press(MainMessage::SelectNoStores.into())
                    .padding(4),
                Button::new(text("Select All"))
                    .on_press(MainMessage::SelectAllStores.into())
                    .padding(4),
            ].spacing(10)),
            (text("Alias Settings").size(20)),
            (alias_settings(self)),
        ))
        .width(280.0);

        // let customize_menu = Menu::new(menu_items!(
        //     (text("Font size")),
        //     (text("Themes..."))
        // )).width(320.0);

        let file_menu = Menu::new(menu_items!(
            // (cw::submenu_button("Customize"), customize_menu),
            (cw::submenu_button("Quick Settings..."), quick_file_menu),
            (cw::menu_text_button("Settings", MainMessage::OpenSettings.into())),
        ))
        .width(320.0);
        
        let actions_menu = Menu::new(menu_items!(
            (cw::menu_text_button("Preview Sales", MainMessage::OpenSalesPreview.into())),
            // (text!("Edit Alert Schedule")),
            // (text("Update cache")),
            (cw::menu_text_button("Logs", MainMessage::OpenLogsView.into())),
        ))
        .width(320.0)
        .close_on_item_click(true);

        let menu_bar = menu_bar!(
            (container(text("File")), file_menu),
            (container(text("Actions")), actions_menu)
        )
        .spacing(5.0)
        .padding(Padding::new(4.0))
        .draw_path(menu::DrawPath::Backdrop)
        .close_on_background_click_global(true);

        let base_view = Tabs::new(|tab| { Message::Main(MainMessage::TabSelected(tab)) })
            .push(
                Tab::Search,
                TabLabel::Text(String::from("Search")),
                tabs::search::search_tab(self),
            )
            .push(
                Tab::Thresholds,
                TabLabel::Text(String::from("Thresholds")),
                self.view_thresholds(),
            )
            .set_active_tab(&self.tab)
            .tab_bar_position(TabBarPosition::Top)
            .width(Length::Fill);

        let top_row = row![
            menu_bar,
        ]
        .width(Length::Fill)
        .spacing(0);

        let tab_bar = {
            let mut bar = row![];

            if self.settings_view_open {
                bar = bar.push(
                    cw::closable_window_button(
                        "Settings",
                        MainMessage::OpenSettings.into(),
                        MainMessage::CloseSettings.into(),
                        self.active_view == View::Settings
                    )
                );
            }
            if self.preview_view_open {
                bar = bar.push(
                    cw::closable_window_button(
                        "Preview",
                        MainMessage::OpenSalesPreview.into(),
                        MainMessage::CloseSalesPreview.into(),
                        self.active_view == View::Preview
                    )
                );
            }
            if self.logs_view_open {
                bar = bar.push(
                    cw::closable_window_button(
                        "Logs",
                        MainMessage::OpenLogsView.into(),
                        MainMessage::CloseLogsView.into(),
                        self.active_view == View::Logs
                    )
                );
            }
            bar.padding(4)
        };

        let settings_window: Element<'_, Message> = sttngs_view::view(self).into();
        let preview_window: Element<'_, Message> = self.preview_view.view().map(Message::Preview);
        let logs_window: Element<'_, Message> = logs_view::view(self).into(); //self.logging_view.view().map(Message::Logging); //logs_view::view(self).into();

        let right_pane = match self.active_view {
            View::Base => {
                // if self.show_dialog && self.tab == {
                //     stack![
                //         base_view,
                //         cs::backdrop(MainMessage::HideDialog.into()),
                //         center(message_dialog(
                //             &self.status_message,
                //             &self.message_details,
                //             MainMessage::HideDialog.into()
                //         ))
                //     ]
                //     .into()
                // } else {
                    base_view.into()
                // }
            }
            View::Settings => settings_window,
            View::Preview => preview_window,
            View::Logs => logs_window,
        };

        let content = column![
            top_row,
            tab_bar,
            container(right_pane)
                .width(Length::Fill)
                .padding(5),
        ]
        .spacing(5);

        content.into()
    }

    fn refresh_state(&mut self) {
        self.thresholds = thresholds::load_thresholds().unwrap_or_default();
        self.sync_threshold_edits();
        self.available_stores = settings::get_available_stores();
        self.selected_stores = settings::get_selected_stores();
        self.alias_enabled = settings::get_alias_state();
        self.alias_reuse_enabled = settings::get_alias_reuse_state();
        self.steam_api_key = properties::get_steam_api_key(!self.reveal_sensitive_data);
        self.recipient_email = properties::get_recipient();
        self.smtp_host = properties::get_smtp_host();
        self.smtp_port = properties::get_smtp_port().to_string();
        self.smtp_email = properties::get_smtp_email();
        self.smtp_user = properties::get_smtp_user();
        self.smtp_password = properties::get_smtp_pwd(!self.reveal_sensitive_data);
        self.project_path = properties::get_project_path();
        self.test_path = properties::get_test_path();
        self.test_mode = properties::is_testing_enabled();
    }

    fn sync_threshold_edits(&mut self) {
        self.threshold_alias_edits = self.thresholds.iter().map(|threshold| threshold.alias.clone()).collect();
        self.threshold_price_edits = self.thresholds.iter().map(|threshold| threshold.desired_price.to_string()).collect();
    }

    fn insert_threshold(
        thresholds_list: &mut Vec<GameThreshold>,
        title: &str,
        alias: &str,
        price: f64,
        steam_id: u32,
        gog_id: u32,
        ms_id: String,
    ) -> bool {
        if let Some(existing) = thresholds_list.iter_mut().find(|existing| existing.title == title) {
            existing.alias = alias.to_string();
            existing.desired_price = price;
            existing.steam_id = steam_id;
            existing.gog_id = gog_id;
            existing.microsoft_store_id = ms_id;
            false
        } else {
            thresholds_list.push(GameThreshold {
                title: title.to_string(),
                alias: alias.to_string(),
                steam_id,
                gog_id,
                microsoft_store_id: ms_id,
                currency: String::from("USD"),
                desired_price: price,
            });
            true
        }
    }
    
    fn set_reveal_sensitive_data(&mut self, reveal: bool) {
        self.reveal_sensitive_data = reveal;
        self.refresh_state();
    }

    fn view_thresholds(&self) -> Element<'_, Message> {
        thrshlds_view::thresholds_tab(self)
    }

    fn start_game_search(&mut self, query: String) -> Task<Message> {
        if query.trim().is_empty() {
           self.logger.info(Screen::Search, "Search query was empty so no game could be found.");
            return Task::none();
        }

        self.search_query = query;
        self.search_results_by_store.clear();
        self.current_store_search_idx = 0;

        for store_id in self.selected_stores.iter() {
            self.search_results_by_store.push((store_id.clone(), Vec::new()));
        }

        if self.search_results_by_store.is_empty() {
            self.logger.info(Screen::Search, "No stores to search");
            return Task::none();
        }

        self.pending_searches = self.search_results_by_store.len();
        self.is_search_in_progress = true;
        self.search_loading_frame = 0;

        self.logger.info(
            Screen::Search,
            &format!("Searching {} stores concurrently for '{}'...",self.search_results_by_store.len(),self.search_query)
        );

        let query = self.search_query.clone();
        let tasks: Vec<Task<Message>> = self.search_results_by_store.iter().map(|(store_id, _)| {
            let query = query.clone();
            let store_id = store_id.clone();
            Task::perform(perform_store_search(query, store_id.clone()), move |result| {
                MainMessage::StoreSearchCompleted(store_id, result).into()
            })
        }).collect();

        Task::batch(tasks)
    }

    fn current_bulk_game(&self) -> Option<SimpleGameThreshold> {
        self.bulk_simple_threshs.get(self.bulk_search_index).cloned()
    }

    fn next_bulk_game(&mut self) -> Option<SimpleGameThreshold> {
        self.bulk_search_index += 1;
        self.bulk_simple_threshs.get(self.bulk_search_index).cloned()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     mod app {
//         use super::*;
//
//         #[test]
//         fn bulk_search_advances_to_next_game() {
//             let mut app = App::new(String::new());
//             app.bulk_simple_threshs = vec![
//                 SimpleGameThreshold { name: "Alpha".into(), price: 10.0 },
//                 SimpleGameThreshold { name: "Beta".into(), price: 20.0 },
//             ];
//             app.bulk_search_index = 0;
//
//             let next_game = app.next_bulk_game();
//
//             assert_eq!(next_game.map(|game| game.name), Some("Beta".into()));
//         }
//
//         #[test]
//         fn bulk_search_return_none_when_done() {
//             let mut app = App::new(String::new());
//             app.bulk_simple_threshs = vec![
//                 SimpleGameThreshold { name: "Alpha".into(), price: 10.0 },
//             ];
//             app.bulk_search_index = 1;
//
//             let next_game = app.next_bulk_game();
//
//             assert!(next_game.is_none());
//         }
//
//         #[test]
//         fn bulk_search_returns_current_game() {
//             let mut app = App::new(String::new());
//             app.bulk_simple_threshs = vec![
//                 SimpleGameThreshold { name: String::from("Alpha"), price: 10.0 },
//                 SimpleGameThreshold { name: String::from("Beta"), price: 20.0 },
//             ];
//             app.bulk_search_index = 1;
//
//             let current_game = app.current_bulk_game();
//
//             assert_eq!(current_game.map(|game| game.name), Some("Beta".to_string()));
//         }
//
//         #[test]
//         fn insert_threshold() {
//             let mut thresholds_list = Vec::new();
//
//             let threshold_added = App::insert_threshold(
//                 &mut thresholds_list,
//                 "Example Game",
//                 "Example Alias",
//                 9.99,
//                 12345,
//                 0,
//                 "".to_string(),
//             );
//
//             assert!(threshold_added);
//             assert_eq!(thresholds_list.len(), 1);
//             assert_eq!(thresholds_list[0].title, "Example Game");
//             assert_eq!(thresholds_list[0].alias, "Example Alias");
//             assert_eq!(thresholds_list[0].desired_price, 9.99);
//             assert_eq!(thresholds_list[0].steam_id, 12345);
//         }
//
//         #[test]
//         fn update_existing_threshold() {
//             let mut thresholds_list = vec![
//                 GameThreshold {
//                     title: "Example Game".into(),
//                     alias: "Old Alias".into(),
//                     steam_id: 100,
//                     gog_id: 0,
//                     microsoft_store_id: String::new(),
//                     currency: String::from("USD"),
//                     desired_price: 19.99,
//                 }
//             ];
//
//             let threshold_added = App::insert_threshold(
//                 &mut thresholds_list,
//                 "Example Game",
//                 "New Alias",
//                 8.99,
//                 200,
//                 10,
//                 "MS123".to_string(),
//             );
//
//             assert!(!threshold_added);
//             assert_eq!(thresholds_list.len(), 1);
//             assert_eq!(thresholds_list[0].alias, "New Alias");
//             assert_eq!(thresholds_list[0].desired_price, 8.99);
//             assert_eq!(thresholds_list[0].steam_id, 200);
//             assert_eq!(thresholds_list[0].gog_id, 10);
//             assert_eq!(thresholds_list[0].microsoft_store_id, "MS123");
//         }
//
//         #[test]
//         fn thresholds_sync_to_alias_and_price_edits() {
//             let mut app = App::new(String::new());
//             app.thresholds = vec![
//                 GameThreshold {
//                     title: "Example Game".into(),
//                     alias: "Alias1".into(),
//                     steam_id: 0,
//                     gog_id: 0,
//                     microsoft_store_id: String::new(),
//                     currency: String::from("USD"),
//                     desired_price: 5.5,
//                 },
//                 GameThreshold {
//                     title: "Example 2".into(),
//                     alias: String::new(),
//                     steam_id: 0,
//                     gog_id: 0,
//                     microsoft_store_id: String::new(),
//                     currency: String::from("USD"),
//                     desired_price: 12.0,
//                 },
//             ];
//
//             app.sync_threshold_edits();
//
//             assert_eq!(app.threshold_alias_edits, vec!["Alias1", ""]);
//             assert_eq!(app.threshold_price_edits, vec!["5.5", "12"]);
//         }
//
//         #[test]
//         fn sort_thresholds_column_order_cycle() {
//             let mut app = App::new(String::new());
//             app.threshold_sort_column = None;
//             app.threshold_sort_order = SortOrder::Original;
//
//             let _ = app.update(MainMessage::SortThresholds(SortColumn::Title).into());
//             assert_eq!(app.threshold_sort_column, Some(SortColumn::Title));
//             assert_eq!(app.threshold_sort_order, SortOrder::Ascending);
//
//             let _ = app.update(MainMessage::SortThresholds(SortColumn::Title).into());
//             assert_eq!(app.threshold_sort_order, SortOrder::Descending);
//
//             let _ = app.update(MainMessage::SortThresholds(SortColumn::Title).into());
//             assert_eq!(app.threshold_sort_column, None);
//             assert_eq!(app.threshold_sort_order, SortOrder::Original);
//         }
//
//         #[test]
//         fn log_start_search_with_no_stores() {
//             let mut app = App::new(String::new());
//             app.selected_stores.clear();
//             app.search_query = "Example".into();
//
//             let _ = app.update(MainMessage::StartSearch.into());
//
//             assert!(!app.is_search_in_progress);
//             assert!(app.log_batch.contains("No stores to search."));
//             assert!(app.search_results_by_store.is_empty());
//         }
//
//         #[test]
//         fn logs_start_search_with_no_query() {
//             let mut app = App::new(String::new());
//             app.search_query.clear();
//
//             let _ = app.update(MainMessage::StartSearch.into());
//
//             assert!(!app.is_search_in_progress);
//             assert!(app.log_batch.contains("Please enter a search query."));
//         }
//
//         #[test]
//         fn update_cache_error_shows_dialog() {
//             let mut app = App::new(String::new());
//             let _ = app.update(MainMessage::UpdateCacheResult(Err("cache update failed".to_string())).into());
//             assert!(app.show_dialog);
//             assert_eq!(app.message_details, "An issue occurred trying to cache game titles. Please check your internet connection or try again later.");
//         }
//
//         #[test]
//         fn check_price_error_shows_dialog() {
//             let mut app = App::new(String::new());
//             let _ = app.update(PreviewMessage::GetSalesUpdated(Err("Could not find sales data".to_string())).into());
//             assert!(app.preview_view.show_dialog);
//             assert_eq!(app.preview_view.message_details, "An issue occurred while looking for game sales. Please check your internet connection or try again later.");
//         }
//     }
// }
