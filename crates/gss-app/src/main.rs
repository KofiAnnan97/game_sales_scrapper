use std::path::PathBuf;
use std::path::Path;
use std::sync::Arc;
use std::io;
use std::collections::{HashMap, HashSet};

use iced::widget::{
    Button, column, container, row, text, stack, center
};
use iced::{Element, Length, Padding, Subscription, Task, application, window, exit};
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

use tabs::{thresholds as thrshlds_view, search::SKIP_STORE_SELECTION, actions::ActionDisplayed};
use views::settings as sttngs_view;
use components::{custom_widgets as cw, custom_styles as cs};
use utils::actions_utils::{send_sales_email, update_cache};
use utils::search_utils::perform_store_search;
use utils::file_utils::open_file;
use utils::log_utils::{self, LogLevel};
use crate::views::logs as logs_view;
use cw::message_dialog;
use crate::views::preview::{PreviewMessage, PreviewView};
use crate::views::settings::{Page, alias_settings, store_selection};

const LOADING_FRAMES_SIZE : usize = 4;

const STATUS_ERR : &str = "ERROR";

fn main() -> iced::Result {
    let log_file = log_utils::new_log();  
    let log_file_clone = log_file.clone();  
    std::panic::set_hook(Box::new(move |panic_info| {
        let panic_msg = log_utils::fatal_message_builder(panic_info);
        general::append_to_file(&log_file_clone, &panic_msg);
    }));

    application(move || App::new(log_file.clone()), App::update, App::view)
        .title("Game Sales Scrapper")
        .subscription(App::subscription)
        .font(ICED_AW_FONT_BYTES)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tab {
    Search,
    Thresholds,
    Actions,
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
    TabSelected(Tab),
    // ViewSelected(View),
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
    AppClosing,
    HideDialog,
    //Settings Messages
    StoreSettingsExpanded(bool),
    PageSelected(Page),
    // Log Messages
    // LogsShown,
    UpdateLogFile,
    OpenLogsView,
    CloseLogsView
}

#[derive(Debug, Clone)]
pub(crate) enum Message{
    Main(MainMessage),
    Preview(PreviewMessage),
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
            Tab::Actions => "Actions",
        }
    }
}

struct App {
    tab: Tab,
    active_view: View,
    settings_view_open: bool,
    preview_view_open: bool,
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
    action_displayed: ActionDisplayed,
    show_dialog: bool,
    preview_view: PreviewView,
    // Settings variables
    settings_page: Page,
    store_settings_expanded: bool,
    // Logging variables
    status_message: String,
    message_details: String,
    log_batch: String,
    current_log_file: String,
    logs_view_open: bool,
}

impl App {
    fn new(log_file: String) -> Self {
        let mut app = Self {
            tab: Tab::Search,
            active_view: View::Base,
            settings_view_open: false,
            preview_view_open: false,
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
            action_displayed: ActionDisplayed::NoAction,
            show_dialog: false,
            preview_view: PreviewView::default(),
            // Settings variables
            settings_page: Page::General,
            store_settings_expanded: false,
            //Logging variables
            status_message: String::from("Ready"),
            message_details: String::new(),
            log_batch: String::new(),
            current_log_file: log_file,
            logs_view_open: false,
        };
        app.sync_threshold_edits();
        app
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(log_utils::new_log())
    }
}

impl App {
    fn subscription(&self) -> Subscription<Message> {
        let log_sub = time::every(Duration::from_secs(30)).map(|_| MainMessage::UpdateLogFile.into());
        let main_tick_sub = time::every(Duration::from_millis(400)).map(|_| MainMessage::Tick.into());
        let preview_tick_sub = time::every(Duration::from_millis(400))
            .map(|_| {Message::Preview(PreviewMessage::Tick)});
        let app_close_sub = window::close_events().map(|_| MainMessage::AppClosing.into());     
        Subscription::batch(vec![log_sub, main_tick_sub, preview_tick_sub, app_close_sub])
    }

    fn update_main(&mut self, message: MainMessage) -> Task<Message>{
        match message {
            MainMessage::TabSelected(tab) => {
                self.tab = tab;
                let status_str = format!("Showing {}", tab.label());
                self.log_batch.push_str(log_utils::message_builder(&status_str, LogLevel::DEBUG).as_str());
                Task::none()
            }
            // MainMessage::ViewSelected(view) => {
            //     self.active_view = view;
            //     let status_str = format!("Switched to {:?}", view);
            //     self.log_batch.push_str(log_utils::message_builder(&status_str, LogLevel::DEBUG).as_str());
            //     Task::none()
            // }
            MainMessage::ToggleStore(store, enabled) => {
                if enabled {
                    if !self.selected_stores.contains(&store) {
                        self.selected_stores.push(store.clone());
                    }
                } else {
                    self.selected_stores.retain(|id| id != &store);
                }
                settings::update_selected_stores(self.selected_stores.clone());
                let status_str = String::from("Updated selected stores");
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::none()
            }
            MainMessage::ToggleAliasEnabled(enabled) => {
                self.alias_enabled = enabled;
                settings::update_alias_state(if enabled { 1 } else { 0 });
                let status_str = format!("Alias enabled: {}", self.alias_enabled);
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::none()
            }
            MainMessage::ToggleAliasReuse(enabled) => {
                self.alias_reuse_enabled = enabled;
                settings::update_alias_reuse_state(if enabled { 1 } else { 0 });
                let status_str = format!("Alias reuse enabled: {}", self.alias_reuse_enabled);
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::none()
            }
            MainMessage::OpenSettings => {
                self.settings_view_open = true;
                self.active_view = View::Settings;
                self.settings_page = Page::General;
                self.log_batch.push_str(&log_utils::message_builder("Opened Settings view", LogLevel::INFO));
                Task::none()
            }
            MainMessage::CloseSettings => {
                self.settings_view_open = false;
                self.active_view = View::Base;
                self.show_dialog = false;
                self.log_batch.push_str(&log_utils::message_builder("Closed Settings view", LogLevel::INFO));
                Task::none()
            }
            MainMessage::OpenSalesPreview => {
                self.active_view = View::Preview;
                self.preview_view_open = true;
                self.log_batch.push_str(&log_utils::message_builder("Open Sales view", LogLevel::INFO));
                Task::done(Message::Preview(PreviewMessage::ResetToSales))
            }
            MainMessage::CloseSalesPreview => {
                self.active_view = View::Base;
                self.preview_view_open = false;
                self.log_batch.push_str(&log_utils::message_builder("Closed Sales view", LogLevel::INFO));
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
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
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
                                    self.log_batch.push_str(&log_utils::message_builder(&log_msg, LogLevel::ERROR));
                                } else {
                                    log_msg = format!("CSV data: {:?}", &self.bulk_simple_threshs);
                                    self.log_batch.push_str(&log_utils::message_builder(&log_msg, LogLevel::INFO));
                                }
                            } else {
                                log_msg = format!("{} is empty", path.display().to_string());
                                self.log_batch.push_str(&log_utils::message_builder(&log_msg, LogLevel::DEBUG));
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
                    let status_str = String::from("No games loaded from CSV.");
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                    return Task::none();
                }

                if let Some(game) = self.current_bulk_game() {
                    self.search_query = game.name.clone();
                    self.add_price = game.price.to_string();
                    self.start_game_search(self.search_query.clone())
                } else {
                    let status_str = String::from("No games loaded from CSV.");
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
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
                Task::none()
            }
            MainMessage::StoreSearchCompleted(game_store, result) => {
                if let Some(entry) = self.search_results_by_store.iter_mut().find(|(id, _)| id == &game_store) {
                    match result {
                        Ok(list) => {
                            entry.1 = list;
                            let status_str = format!("Search complete for {}", game_store.get_name());
                            self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                        }
                        Err(err) => {
                            let status_str = format!("Search failed for {}: {}", game_store.get_name(), err);
                            self.log_batch.push_str(log_utils::message_builder(&status_str, LogLevel::ERROR).as_str());
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
                self.log_batch.push_str(&log_utils::message_builder("Reset search results", LogLevel::INFO));
                Task::none()
            }
            MainMessage::NextStore => {
                if self.current_store_search_idx < self.search_results_by_store.len() - 1 {
                    self.current_store_search_idx += 1;
                    let (game_store, results) = &self.search_results_by_store[self.current_store_search_idx];
                    if results.is_empty() {
                        let query = self.search_query.clone();
                        let game_store_clone = game_store.clone();
                        let status_str = format!("Searching {}...", game_store.get_name());
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                        return Task::perform(perform_store_search(query, game_store_clone), move |result| {
                            MainMessage::StoreSearchCompleted(game_store_clone, result).into()
                        });
                    }
                } else {
                    let status_str = String::from("Reached last store.");
                    self.status_message = status_str.clone();
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                }
                Task::none()
            }
            MainMessage::PreviousStore => {
                if self.current_store_search_idx > 0 {
                    self.current_store_search_idx -= 1;
                    let (store_id, _) = &self.search_results_by_store[self.current_store_search_idx];
                    let status_str = format!("Viewing results from {}", store_id);
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                }
                Task::none()
            }
            MainMessage::AddThreshold => {
                let price = match self.add_price.trim().parse::<f64>() {
                    Ok(price) => price,
                    Err(_) => {
                        let status_str = String::from("Invalid desired price. Enter a decimal value.");
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
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
                    let status_str = String::from("No selected storefront titles were added. Choose a result before adding a threshold.");
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
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
                            let status_str = format!("Added threshold for '{}'. Moving onto the next game {}.", &self.search_query, next_game.name);
                            self.search_query = next_game.name.clone();
                            self.add_price = next_game.price.to_string();
                            self.add_alias.clear();
                            
                            self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                            return self.start_game_search(self.search_query.clone());
                        }

                        self.bulk_search_used = false;
                        let status_str = String::from("Finished processing games from CSV file.");
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
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
                    self.log_batch.push_str(&log_utils::message_builder("Saving general settings", LogLevel::INFO));
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
                    self.log_batch.push_str(&log_utils::message_builder("Saving email settings", LogLevel::INFO));
                } else if self.settings_page == Page::Stores(GameStore::STEAM) {
                    if self.reveal_sensitive_data && !self.steam_api_key.is_empty() {
                        properties::set_steam_api_key(self.steam_api_key.clone());
                    }
                    self.log_batch.push_str(&log_utils::message_builder("Saving steam settings", LogLevel::INFO));
                }

                self.show_dialog = true;
                Task::none()
            }
            MainMessage::SendEmailResult(result) => {
                let level;
                let details;
                let (level, details) = match result {
                    Ok(success) => {
                        self.log_batch.push_str(&log_utils::message_builder(&success, LogLevel::INFO));
                        level = "INFO".into();
                        details = "Email request has been successfully sent.".into();
                        (level, details)
                    }
                    Err(err) => {
                        self.log_batch.push_str(&log_utils::message_builder(&err, LogLevel::ERROR));
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
                self.action_displayed = ActionDisplayed::UpdateCache;
                self.is_caching_in_progress = true;
                let status_str = String::from("Updating cache...");
                self.status_message = status_str.clone();
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::perform(update_cache(), |result| MainMessage::UpdateCacheResult(result).into())
            }
            MainMessage::UpdateCacheResult(result) => {
                self.is_caching_in_progress = false;
                match result {
                    Ok(output) => {
                        self.log_batch.push_str(&log_utils::message_builder(&output, LogLevel::INFO));
                        let status_str = String::from("Cache update complete");
                        self.status_message = status_str.clone();
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                    }
                    Err(err) => {
                        self.log_batch.push_str(&log_utils::message_builder(&err, LogLevel::ERROR));
                        let err_msg = format!("Cache update failed, {:?}", err);
                        self.status_message = STATUS_ERR.into();
                        self.message_details = "An issue occurred trying to cache game titles. Please check your internet connection or try again later.".into();
                        self.show_dialog = true;
                        self.log_batch.push_str(&log_utils::message_builder(&err_msg, LogLevel::ERROR));
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
                let status_str = format!("Threshold row updated.");
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::none()
            }
            MainMessage::RemoveThresholdRow(idx) => {
                if let Some(title) = self.thresholds.get(idx).map(|threshold| threshold.title.clone()) {
                    let _ = thresholds::remove(&title);
                    self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                    self.sync_threshold_edits();
                    let status_str = format!("Removed threshold {}.", title);
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                }
                Task::none()
            }
            MainMessage::Refresh => {
                self.refresh_state();
                let status_str = String::from("Refreshed state");
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::DEBUG));
                Task::none()
            }
            // MainMessage::LogsShown => {
            //     self.action_displayed = ActionDisplayed::Logs;
            //     Task::none()
            // }
            MainMessage::UpdateLogFile => {
                if !self.log_batch.is_empty() && !self.current_log_file.is_empty() {
                    general::append_to_file(&self.current_log_file, &self.log_batch);
                    self.log_batch.clear();
                }   
                Task::none()
            }
            MainMessage::AppClosing => {
                if !self.log_batch.is_empty() {
                    self.log_batch.push_str(&log_utils::message_builder("Application successfully exited.", LogLevel::INFO));
                    general::append_to_file(&self.current_log_file, &self.log_batch);
                    self.log_batch.clear();
                }
                exit()
            }
            MainMessage::HideDialog => { 
                self.show_dialog = false; 
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
                self.log_batch.push_str(&log_utils::message_builder("Open Logs view", LogLevel::INFO));
                Task::none()
            }
            MainMessage::CloseLogsView => {
                self.active_view = View::Base;
                self.logs_view_open = false;
                self.log_batch.push_str(&log_utils::message_builder("Closed Logs view", LogLevel::INFO));
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
                let status_str = String::from("Sending email...");
                self.status_message = status_str.clone();
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::perform(send_sales_email(), |result| MainMessage::SendEmailResult(result).into())
            }
            PreviewMessage::OpenEmailSettings => {
                self.settings_view_open = true;
                self.active_view = View::Settings;
                self.settings_page = Page::Email;
                Task::none()
            }
            PreviewMessage::SendLogEvent => {
                self.log_batch.push_str(&self.preview_view.log_msg);
                Task::none()
            }
            preview_message => {
                self.preview_view
                    .update(preview_message)
                    .map(Message::Preview)
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Main(message) => self.update_main(message),
            Message::Preview(message) => self.update_preview(message),
        }
    }

    fn view(&self) -> Element<'_, Message> {       
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
            .push(
                Tab::Actions,
                TabLabel::Text(String::from("Actions")),
                tabs::actions::view_actions(self),
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
        let logs_window: Element<'_, Message> = logs_view::view(self).into();

        let right_pane = match self.active_view {
            View::Base => {
                if self.show_dialog && self.tab == Tab::Actions  {
                    stack![
                        base_view,
                        cs::backdrop(MainMessage::HideDialog.into()),
                        center(message_dialog(
                            &self.status_message,
                            &self.message_details,
                            MainMessage::HideDialog.into()
                        ))
                    ]
                    .into()
                } else {
                    base_view.into()
                }
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
            let status_str = String::from("Please enter a search query.");
            self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
            return Task::none();
        }

        self.search_query = query;
        self.search_results_by_store.clear();
        self.current_store_search_idx = 0;

        for store_id in self.selected_stores.iter() {
            self.search_results_by_store.push((store_id.clone(), Vec::new()));
        }

        if self.search_results_by_store.is_empty() {
            let status_str = String::from("No stores to search.");
            self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
            return Task::none();
        }

        self.pending_searches = self.search_results_by_store.len();
        self.is_search_in_progress = true;
        self.search_loading_frame = 0;

        let status_str = format!("Searching {} stores concurrently for '{}'...",self.search_results_by_store.len(),self.search_query);
        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));

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

#[cfg(test)]
mod tests {
    use super::*;

    mod app {
        use super::*;

        #[test]
        fn bulk_search_advances_to_next_game() {
            let mut app = App::new(String::new());
            app.bulk_simple_threshs = vec![
                SimpleGameThreshold { name: "Alpha".into(), price: 10.0 },
                SimpleGameThreshold { name: "Beta".into(), price: 20.0 },
            ];
            app.bulk_search_index = 0;

            let next_game = app.next_bulk_game();

            assert_eq!(next_game.map(|game| game.name), Some("Beta".into()));
        }

        #[test]
        fn bulk_search_return_none_when_done() {
            let mut app = App::new(String::new());
            app.bulk_simple_threshs = vec![
                SimpleGameThreshold { name: "Alpha".into(), price: 10.0 },
            ];
            app.bulk_search_index = 1;

            let next_game = app.next_bulk_game();

            assert!(next_game.is_none());
        }

        #[test]
        fn bulk_search_returns_current_game() {
            let mut app = App::new(String::new());
            app.bulk_simple_threshs = vec![
                SimpleGameThreshold { name: String::from("Alpha"), price: 10.0 },
                SimpleGameThreshold { name: String::from("Beta"), price: 20.0 },
            ];
            app.bulk_search_index = 1;

            let current_game = app.current_bulk_game();

            assert_eq!(current_game.map(|game| game.name), Some("Beta".to_string()));
        }

        #[test]
        fn insert_threshold() {
            let mut thresholds_list = Vec::new();

            let threshold_added = App::insert_threshold(
                &mut thresholds_list,
                "Example Game",
                "Example Alias",
                9.99,
                12345,
                0,
                "".to_string(),
            );

            assert!(threshold_added);
            assert_eq!(thresholds_list.len(), 1);
            assert_eq!(thresholds_list[0].title, "Example Game");
            assert_eq!(thresholds_list[0].alias, "Example Alias");
            assert_eq!(thresholds_list[0].desired_price, 9.99);
            assert_eq!(thresholds_list[0].steam_id, 12345);
        }

        #[test]
        fn update_existing_threshold() {
            let mut thresholds_list = vec![
                GameThreshold {
                    title: "Example Game".into(),
                    alias: "Old Alias".into(),
                    steam_id: 100,
                    gog_id: 0,
                    microsoft_store_id: String::new(),
                    currency: String::from("USD"),
                    desired_price: 19.99,
                }
            ];

            let threshold_added = App::insert_threshold(
                &mut thresholds_list,
                "Example Game",
                "New Alias",
                8.99,
                200,
                10,
                "MS123".to_string(),
            );

            assert!(!threshold_added);
            assert_eq!(thresholds_list.len(), 1);
            assert_eq!(thresholds_list[0].alias, "New Alias");
            assert_eq!(thresholds_list[0].desired_price, 8.99);
            assert_eq!(thresholds_list[0].steam_id, 200);
            assert_eq!(thresholds_list[0].gog_id, 10);
            assert_eq!(thresholds_list[0].microsoft_store_id, "MS123");
        }

        #[test]
        fn thresholds_sync_to_alias_and_price_edits() {
            let mut app = App::new(String::new());
            app.thresholds = vec![
                GameThreshold {
                    title: "Example Game".into(),
                    alias: "Alias1".into(),
                    steam_id: 0,
                    gog_id: 0,
                    microsoft_store_id: String::new(),
                    currency: String::from("USD"),
                    desired_price: 5.5,
                },
                GameThreshold {
                    title: "Example 2".into(),
                    alias: String::new(),
                    steam_id: 0,
                    gog_id: 0,
                    microsoft_store_id: String::new(),
                    currency: String::from("USD"),
                    desired_price: 12.0,
                },
            ];

            app.sync_threshold_edits();

            assert_eq!(app.threshold_alias_edits, vec!["Alias1", ""]);
            assert_eq!(app.threshold_price_edits, vec!["5.5", "12"]);
        }

        #[test]
        fn sort_thresholds_column_order_cycle() {
            let mut app = App::new(String::new());
            app.threshold_sort_column = None;
            app.threshold_sort_order = SortOrder::Original;

            let _ = app.update(MainMessage::SortThresholds(SortColumn::Title).into());
            assert_eq!(app.threshold_sort_column, Some(SortColumn::Title));
            assert_eq!(app.threshold_sort_order, SortOrder::Ascending);

            let _ = app.update(MainMessage::SortThresholds(SortColumn::Title).into());
            assert_eq!(app.threshold_sort_order, SortOrder::Descending);

            let _ = app.update(MainMessage::SortThresholds(SortColumn::Title).into());
            assert_eq!(app.threshold_sort_column, None);
            assert_eq!(app.threshold_sort_order, SortOrder::Original);
        }

        #[test]
        fn log_start_search_with_no_stores() {
            let mut app = App::new(String::new());
            app.selected_stores.clear();
            app.search_query = "Example".into();

            let _ = app.update(MainMessage::StartSearch.into());

            assert!(!app.is_search_in_progress);
            assert!(app.log_batch.contains("No stores to search."));
            assert!(app.search_results_by_store.is_empty());
        }

        #[test]
        fn logs_start_search_with_no_query() {
            let mut app = App::new(String::new());
            app.search_query.clear();

            let _ = app.update(MainMessage::StartSearch.into());

            assert!(!app.is_search_in_progress);
            assert!(app.log_batch.contains("Please enter a search query."));
        }

        #[test]
        fn update_cache_error_shows_dialog() {
            let mut app = App::new(String::new());
            let _ = app.update(MainMessage::UpdateCacheResult(Err("cache update failed".to_string())).into());
            assert!(app.show_dialog);
            assert_eq!(app.message_details, "An issue occurred trying to cache game titles. Please check your internet connection or try again later.");
        }

        #[test]
        fn check_price_error_shows_dialog() {
            let mut app = App::new(String::new());
            let _ = app.update(PreviewMessage::GetSalesUpdated(Err("Could not find sales data".to_string())).into());
            assert!(app.preview_view.show_dialog);
            assert_eq!(app.preview_view.message_details, "An issue occurred while looking for game sales. Please check your internet connection or try again later.");
        }
    }
}
