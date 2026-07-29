use iced::widget::{
    Button, Checkbox, column, container, row, text, stack, center
};
use iced::{Element, Length, Padding, Subscription, Task, clipboard, application, window, exit};
use iced::time::{self, Duration};
use iced_aw::menu::{self, Menu};
use iced_aw::{ICED_AW_FONT_BYTES, menu_bar, menu_items, TabLabel, tabs::{Tabs, TabBarPosition}};

use std::path::PathBuf;
use std::path::Path;
use std::sync::Arc;
use std::io;
use std::collections::{HashMap, HashSet};

// Common internal libraries
use file_types::{csv, general};
use file_ops::{settings, thresholds};
use properties;
use structs::internal::data::{GameThreshold, SimpleGameThreshold};

// App specific modules
mod views;
mod components;
mod utils;

use views::{thresholds as thrshlds_view, settings as sttngs_view};
use views::search::SKIP_STORE_SELECTION;
use views::actions::ActionDisplayed;
use components::{custom_widgets, custom_styles};
use utils::actions_utils::{send_sales_email, update_cache};
use utils::search_utils::perform_store_search;
use utils::pricing_utils::{check_prices_for_display, SaleInfoWithHandler};
use utils::file_utils::open_file;
use utils::log_utils::{self, LogLevel};

use crate::components::custom_widgets::message_dialog;

const LOADING_FRAMES_SIZE: usize = 4;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Search,
    Thresholds,
    Actions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Base,
    Settings,
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
enum Message {
    TabSelected(Tab),
    ViewSelected(View),
    OpenMoreSettings,
    CloseSettings,
    ToggleStore(String, bool),
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
    StoreSearchCompleted(String, Result<Vec<StoreSearchResult>, String>),
    SearchReset,
    NextStore,
    PreviousStore,
    AddThreshold,
    ThresholdAliasChanged(usize, String),
    ThresholdPriceChanged(usize, String),
    UpdateThresholdRow(usize),
    RemoveThresholdRow(usize),
    SortThresholds(SortColumn),
    CheckPrices,
    CheckPricesResult(Result<HashMap<String, Vec<SaleInfoWithHandler>>, String>),
    SendEmail,
    SendEmailResult(Result<String, String>),
    UpdateCache,
    UpdateCacheResult(Result<String, String>),
    CopyLinkToClipboard(String, String),
    ResetCopyMessage,
    LogsShown,
    UpdateLogFile,
    Tick,
    Refresh,
    AppClosing,
    HideDialog,
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
    available_stores: Vec<String>,
    selected_stores: Vec<String>,
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
    search_results_by_store: Vec<(String, Vec<StoreSearchResult>)>,
    current_store_search_idx: usize,
    selected_results_by_store: HashMap<String, Option<usize>>,
    is_search_in_progress: bool,
    is_caching_in_progress: bool,
    is_price_check_in_progress: bool,
    pending_searches: usize,
    search_loading_frame: usize,
    caching_loading_frame: usize,
    price_check_loading_frame: usize,
    selected_file: Option<PathBuf>,
    bulk_simple_threshs: Vec<SimpleGameThreshold>,
    bulk_search_index: usize,
    bulk_search_used: bool,
    thresholds: Vec<GameThreshold>,
    threshold_alias_edits: Vec<String>,
    threshold_price_edits: Vec<String>,
    threshold_sort_column: Option<SortColumn>,
    threshold_sort_order: SortOrder,
    status_message: String,
    log_batch: String,
    current_log_file: String,
    action_displayed: ActionDisplayed,
    sales_info_by_store: HashMap<String, Vec<SaleInfoWithHandler>>,
    show_dialog: bool,
    copied_link: Option<String>,
}

impl App {
    fn new(log_file: String) -> Self {
        let mut app = Self {
            tab: Tab::Search,
            active_view: View::Base,
            settings_view_open: false,
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
            is_price_check_in_progress: false,
            pending_searches: 0,
            search_loading_frame: 0,
            caching_loading_frame: 0,
            price_check_loading_frame: 0,
            selected_file: None,
            bulk_simple_threshs: Vec::new(),
            bulk_search_index: 0,
            bulk_search_used: false,
            thresholds: thresholds::load_thresholds().unwrap_or_default(),
            threshold_alias_edits: Vec::new(),
            threshold_price_edits: Vec::new(),
            threshold_sort_column: None,
            threshold_sort_order: SortOrder::Original,
            status_message: String::from("Ready"),
            log_batch: String::new(),
            current_log_file: log_file,
            action_displayed: ActionDisplayed::NoAction,
            sales_info_by_store: {
                let sibs: HashMap<String, Vec<SaleInfoWithHandler>> = settings::get_available_stores()
                .iter()
                    .map(|store_name| (store_name.clone(), Vec::new()))
                    .collect();
                sibs
            },
            show_dialog: false,
            copied_link: None,
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
        let log_subscription = time::every(Duration::from_secs(30)).map(|_| Message::UpdateLogFile);
        let tick_subscription = time::every(Duration::from_millis(400)).map(|_| Message::Tick);
        let app_close_subscription = window::close_events().map(|_| Message::AppClosing);     
        Subscription::batch(vec![log_subscription, tick_subscription, app_close_subscription])
    }
    
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.tab = tab;
                let status_str = format!("Showing {}", tab.label());
                self.log_batch.push_str(log_utils::message_builder(&status_str, LogLevel::DEBUG).as_str());
                Task::none()
            }
            Message::ViewSelected(view) => {
                self.active_view = view;
                let status_str = format!("Switched to {:?}", view);
                self.log_batch.push_str(log_utils::message_builder(&status_str, LogLevel::DEBUG).as_str());
                Task::none()
            }
            Message::ToggleStore(store, enabled) => {
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
            Message::ToggleAliasEnabled(enabled) => {
                self.alias_enabled = enabled;
                settings::update_alias_state(if enabled { 1 } else { 0 });
                let status_str = format!("Alias enabled: {}", self.alias_enabled);
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::none()
            }
            Message::ToggleAliasReuse(enabled) => {
                self.alias_reuse_enabled = enabled;
                settings::update_alias_reuse_state(if enabled { 1 } else { 0 });
                let status_str = format!("Alias reuse enabled: {}", self.alias_reuse_enabled);
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::none()
            }
            Message::OpenMoreSettings => {
                self.settings_view_open = true;
                self.active_view = View::Settings;
                let status_str = String::from("Opened more settings view");
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::none()
            }
            Message::CloseSettings => {
                self.settings_view_open = false;
                self.active_view = View::Base;
                self.show_dialog = false;
                let status_str = String::from("Closed settings tab");
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::none()
            }
            Message::SortThresholds(column) => {
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
            Message::ProjectPathChanged(value) => { self.project_path = value; Task::none() }
            Message::TestPathChanged(value) => { self.test_path = value; Task::none() }
            Message::SteamApiKeyChanged(value) => { self.steam_api_key = value; Task::none() }
            Message::RecipientEmailChanged(value) => { self.recipient_email = value; Task::none() }
            Message::SmtpHostChanged(value) => { self.smtp_host = value; Task::none() }
            Message::SmtpPortChanged(value) => { self.smtp_port = value; Task::none() }
            Message::SmtpEmailChanged(value) => { self.smtp_email = value; Task::none() }
            Message::SmtpUserChanged(value) => { self.smtp_user = value; Task::none() }
            Message::SmtpPasswordChanged(value) => { self.smtp_password = value; Task::none() }
            Message::ToggleSensitiveData(reveal) => { self.set_reveal_sensitive_data(reveal); Task::none() }
            Message::ToggleTestMode(enabled) => { self.test_mode = enabled; Task::none() }
            Message::SearchQueryChanged(value) => { self.search_query = value; Task::none() }
            Message::StartSearch => {
                self.bulk_search_used = false;
                self.start_game_search(self.search_query.clone())
            }
            Message::SelectAllStores => {
                self.selected_stores = self.available_stores.clone();
                settings::update_selected_stores(self.selected_stores.clone());
                Task::none()
            }
            Message::SelectNoStores => {
                self.selected_stores.clear();
                settings::update_selected_stores(Vec::new());
                Task::none()
            }
            Message::OpenCsv => {
                if self.is_search_in_progress {
                    Task::none()
                } else {
                    self.is_search_in_progress = true;
                    window::oldest()
                        .and_then(|id| window::run(id, open_file))
                        .then(Task::future)
                        .map(Message::CsvOpened)
                }
            }
            Message::CsvOpened(result) => {
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
            Message::CopyLinkToClipboard(id, url) => {
                self.copied_link = Some(id);

                Task::batch([
                    clipboard::write(url),
                    Task::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    },
                    |_| Message::ResetCopyMessage,
                )
                ])
            }
            Message::ResetCopyMessage => {
                self.copied_link = None;
                Task::none()
            }
            Message::Tick => {
                if self.is_search_in_progress {
                    self.search_loading_frame = (self.search_loading_frame + 1) % LOADING_FRAMES_SIZE;
                }
                if self.is_caching_in_progress {
                    self.caching_loading_frame = (self.search_loading_frame + 1) % LOADING_FRAMES_SIZE;
                }
                if self.is_price_check_in_progress {
                    self.price_check_loading_frame = (self.price_check_loading_frame + 1) % LOADING_FRAMES_SIZE;
                }
                Task::none()
            }
            Message::ExecuteBulkInsert => {
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
            Message::SearchResultSelected(selected) => {
                if let Some((store_id, _)) = self.search_results_by_store.get(self.current_store_search_idx) {
                    if selected == SKIP_STORE_SELECTION {
                        self.selected_results_by_store.insert(store_id.clone(), None);
                    } else {
                        self.selected_results_by_store.insert(store_id.clone(), Some(selected));
                    }
                }
                Task::none()
            }
            Message::StoreSearchCompleted(store_id, result) => {
                if let Some(entry) = self.search_results_by_store.iter_mut().find(|(id, _)| id == &store_id) {
                    match result {
                        Ok(list) => {
                            entry.1 = list;
                            let status_str = format!("Search complete for {}", store_id);
                            self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                        }
                        Err(err) => {
                            let status_str = format!("Search failed for {}: {}", store_id, err);
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
            Message::SearchReset => {
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
            Message::NextStore => {
                if self.current_store_search_idx < self.search_results_by_store.len() - 1 {
                    self.current_store_search_idx += 1;
                    let (store_id, results) = &self.search_results_by_store[self.current_store_search_idx];
                    if results.is_empty() {
                        let query = self.search_query.clone();
                        let store_id_clone = store_id.clone();
                        let status_str = format!("Searching {}...", store_id);
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                        return Task::perform(perform_store_search(query, store_id_clone.clone()), move |result| {
                            Message::StoreSearchCompleted(store_id_clone, result)
                        });
                    }
                } else {
                    let status_str = String::from("Reached last store.");
                    self.status_message = status_str.clone();
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                }
                Task::none()
            }
            Message::PreviousStore => {
                if self.current_store_search_idx > 0 {
                    self.current_store_search_idx -= 1;
                    let (store_id, _) = &self.search_results_by_store[self.current_store_search_idx];
                    let status_str = format!("Viewing results from {}", store_id);
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                }
                Task::none()
            }
            Message::AddThreshold => {
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
            Message::SaveSettings => {
                if !self.project_path.is_empty() && Path::new(&self.project_path).is_dir() {
                    properties::set_project_path(&self.project_path);
                }
                if !self.test_path.is_empty() && Path::new(&self.test_path).is_dir() {
                    properties::set_test_path(&self.test_path);
                }
                if self.reveal_sensitive_data && !self.steam_api_key.is_empty() {
                    properties::set_steam_api_key(self.steam_api_key.clone());
                }
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
                properties::set_test_mode(self.test_mode);
                let status_str = String::from("Saved settings");
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                self.show_dialog = true;
                Task::none()
            }
            Message::CheckPrices => {
                self.search_results_by_store.clear();
                self.action_displayed = ActionDisplayed::CheckPrices;
                self.is_price_check_in_progress = true;
                self.price_check_loading_frame = 0;
                let status_str = String::from("Fetching sales info...");
                self.status_message = status_str.clone();
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::perform(check_prices_for_display(), Message::CheckPricesResult)
            }
            Message::CheckPricesResult(result) => {
                self.is_price_check_in_progress = false;
                match result {
                    Ok(map) => {
                        self.sales_info_by_store = map;
                        let status_str = String::from("Price info fetched for sales.");
                        self.status_message = status_str.clone();
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                    }
                    Err(err) => {
                        self.log_batch= err.clone();
                        let status_str = String::from("Failed to fetch sales info");
                        self.status_message = status_str.clone();
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                    }
                }
                Task::none()
            }
            Message::SendEmail => {
                self.action_displayed = ActionDisplayed::TestEmail;
                let status_str = String::from("Sending email...");
                self.status_message = status_str.clone();
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::perform(send_sales_email(), Message::SendEmailResult)
            }
            Message::SendEmailResult(result) => {
                match result {
                    Ok(output) => {
                        self.log_batch.push_str(&log_utils::message_builder(&output, LogLevel::INFO));
                        let status_str = String::from("Email request complete");
                        self.status_message = status_str.clone();
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                    }
                    Err(err) => {
                        self.log_batch.push_str(&log_utils::message_builder(&err, LogLevel::ERROR));
                        let status_str = String::from("Email failed");
                        self.status_message = status_str.clone();
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::ERROR));
                    }
                }
                Task::none()
            }
            Message::UpdateCache => {
                self.action_displayed = ActionDisplayed::UpdateCache;
                self.is_caching_in_progress = true;
                self.price_check_loading_frame = 0;
                let status_str = String::from("Updating cache...");
                self.status_message = status_str.clone();
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                Task::perform(update_cache(), Message::UpdateCacheResult)
            }
            Message::UpdateCacheResult(result) => {
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
                        let status_str = String::from("Cache update failed");
                        self.status_message = status_str.clone();
                        self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::ERROR));
                    }
                }
                Task::none()
            }
            Message::ThresholdAliasChanged(idx, value) => {
                if idx == usize::MAX {
                    self.add_alias = value;
                } else if let Some(slot) = self.threshold_alias_edits.get_mut(idx) {
                    *slot = value;
                }
                Task::none()
            }
            Message::ThresholdPriceChanged(idx, value) => {
                if idx == usize::MAX {
                    self.add_price = value;
                } else if let Some(slot) = self.threshold_price_edits.get_mut(idx) {
                    *slot = value;
                }
                Task::none()
            }
            Message::UpdateThresholdRow(idx) => {
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
            Message::RemoveThresholdRow(idx) => {
                if let Some(title) = self.thresholds.get(idx).map(|threshold| threshold.title.clone()) {
                    let _ = thresholds::remove(&title);
                    self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                    self.sync_threshold_edits();
                    let status_str = format!("Removed threshold {}.", title);
                    self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::INFO));
                }
                Task::none()
            }
            Message::Refresh => {
                self.refresh_state();
                let status_str = String::from("Refreshed state");
                self.log_batch.push_str(&log_utils::message_builder(&status_str, LogLevel::DEBUG));
                Task::none()
            }
            Message::LogsShown => {
                self.action_displayed = ActionDisplayed::Logs;
                Task::none()
            }
            Message::UpdateLogFile => {
                if !self.log_batch.is_empty() && !self.current_log_file.is_empty() {
                    general::append_to_file(&self.current_log_file, &self.log_batch);
                    self.log_batch.clear();
                }   
                Task::none()
            }
            Message::AppClosing => {
                if !self.log_batch.is_empty() {
                    self.log_batch.push_str(&log_utils::message_builder("Application successfully exited.", LogLevel::INFO));
                    general::append_to_file(&self.current_log_file, &self.log_batch);
                    self.log_batch.clear();
                }
                exit()
            }
            Message::HideDialog => { self.show_dialog = false; Task::none() }
        }
    }

    fn view(&self) -> Element<'_, Message> {       
        let store_menu_items = self.available_stores.iter().fold(column![], |column, store_id| {
            let label = settings::get_proper_store_name(store_id).unwrap_or_else(|| store_id.clone());
            column.push(
                Checkbox::new(self.selected_stores.contains(store_id))
                    .label(label)
                    .on_toggle(move |enabled| Message::ToggleStore(store_id.clone(), enabled))
                    .width(Length::Fill),
            )
        });

        let store_selection_menu = Menu::new(menu_items!(
            (container(store_menu_items).width(Length::Fill)),
            (row![
                Button::new(text("Select None"))
                    .on_press(Message::SelectNoStores)
                    .padding(4),
                Button::new(text("Select All"))
                    .on_press(Message::SelectAllStores)
                    .padding(4),
            ])
        ))
        .width(280.0);

        let alias_options_menu = Menu::new(menu_items!(
            (
                Checkbox::new(self.alias_enabled)
                    .label("Enable aliases")
                    .on_toggle(Message::ToggleAliasEnabled)
                    .width(Length::Fill)
            ),
            (
                Checkbox::new(self.alias_reuse_enabled)
                    .label("Enable alias reuse")
                    .on_toggle(Message::ToggleAliasReuse)
                    .width(Length::Fill)
            ),
        ))
        .width(280.0);
        // .close_on_item_click(true);

        let settings_menu = Menu::new(menu_items!(
            (custom_widgets::submenu_button("Selected Stores"), store_selection_menu),
            (custom_widgets::submenu_button("Alias Options"), alias_options_menu),
            (custom_widgets::menu_text_button("More Settings...", Message::OpenMoreSettings)),
        ))
        .width(320.0);

        // let customize_menu = Menu::new(menu_items!(
        //     (text("Font size")),
        //     (text("Themes..."))
        // )).width(320.0);
        // let actions_menu = Menu::new(menu_items!(
        //     (text("Schedule sub menu")),
        //     (text("SMTP Email (in sub menu)")),
        //     (text("Update cache")),
        //     (text("Check Logs"))
        // )).width(320.0);

        let menu_bar = menu_bar!(
            // (container(text("Customize")), customize_menu),
            (container(text("Settings")), settings_menu),
            // (container(text("Actions")), actions_menu)
        )
        .spacing(5.0)
        .padding(Padding::new(4.0))
        .draw_path(menu::DrawPath::Backdrop)
        .close_on_background_click_global(true);

        let base_view = Tabs::new(Message::TabSelected)
            .push(
                Tab::Search,
                TabLabel::Text(String::from("Search")),
                views::search::search_tab(self),
            )
            .push(
                Tab::Thresholds,
                TabLabel::Text(String::from("Thresholds")),
                self.view_thresholds(),
            )
            .push(
                Tab::Actions,
                TabLabel::Text(String::from("Actions")),
                views::actions::view_actions(self),
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
                if self.active_view == View::Settings {
                    bar = bar.push(
                        row![
                            container(text("Settings")).padding(8),
                            Button::new(text("×"))
                                .on_press(Message::CloseSettings)
                                .padding(8),
                        ]
                        .spacing(4)
                        .align_y(iced::Alignment::Center)
                    );
                } else {
                    bar = bar.push(Button::new(text("Settings")).on_press(Message::ViewSelected(View::Settings)).padding(8));
                }
            }

            bar.spacing(10).padding(10)
        };

        let settings_window: Element<'_, Message> = if self.show_dialog {
            stack![
                sttngs_view::settings_window(self),
                custom_styles::backdrop(Message::HideDialog),
                center(message_dialog(
                    "Info",
                    "Settings were saved successfully.",
                    Message::CloseSettings
                ))
            ]
            .into()
        } else {
            sttngs_view::settings_window(self).into()
        };

        let right_pane = match self.active_view {
            View::Base => base_view.into(),
            View::Settings => settings_window,
        };

        let content = column![
            top_row,
            tab_bar,
            container(right_pane)
                .width(Length::Fill)
                .padding(10),
        ]
        .spacing(10);

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
                Message::StoreSearchCompleted(store_id, result)
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

            let _ = app.update(Message::SortThresholds(SortColumn::Title));
            assert_eq!(app.threshold_sort_column, Some(SortColumn::Title));
            assert_eq!(app.threshold_sort_order, SortOrder::Ascending);

            let _ = app.update(Message::SortThresholds(SortColumn::Title));
            assert_eq!(app.threshold_sort_order, SortOrder::Descending);

            let _ = app.update(Message::SortThresholds(SortColumn::Title));
            assert_eq!(app.threshold_sort_column, None);
            assert_eq!(app.threshold_sort_order, SortOrder::Original);
        }

        #[test]
        fn log_start_search_with_no_stores() {
            let mut app = App::new(String::new());
            app.selected_stores.clear();
            app.search_query = "Example".into();

            let _ = app.update(Message::StartSearch);

            assert!(!app.is_search_in_progress);
            assert!(app.log_batch.contains("No stores to search."));
            assert!(app.search_results_by_store.is_empty());
        }

        #[test]
        fn logs_start_search_with_no_query() {
            let mut app = App::new(String::new());
            app.search_query.clear();

            let _ = app.update(Message::StartSearch);

            assert!(!app.is_search_in_progress);
            assert!(app.log_batch.contains("Please enter a search query."));
        }
    }
}
