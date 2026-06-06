use iced::widget::{
    Button, Checkbox, column, container, row, text,
};
use iced::{Element, Length, Task, application, Padding};
use iced_aw::menu::{self, Menu};
use iced_aw::{ICED_AW_FONT_BYTES, menu_bar, menu_items, TabLabel, tabs::{Tabs, TabBarPosition}};

use reqwest::Client;
use std::path::Path;
use std::collections::{HashMap, HashSet};

// Common internal libraries
use file_ops::{settings, thresholds};
use properties;
use structs::internal::data::{GameThreshold};

// App specific modules
mod views;
mod components;
mod utils;

use views::{thresholds as thrshlds_view, settings as sttngs_view};
use views::search::SKIP_STORE_SELECTION;
use components::custom_buttons;
use utils::actions_utils::{send_sales_email, update_cache};
use utils::search_utils::perform_store_search;
use utils::pricing_utils::check_prices;

fn main() -> iced::Result {
    application(App::default, App::update, App::view)
        .title("Game Sales Scrapper")
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
    SaveSettings,
    SearchQueryChanged(String),
    StartSearch,
    SearchResultSelected(usize),
    StoreSearchCompleted(String, Result<Vec<StoreSearchResult>, String>),
    NextStore,
    PreviousStore,
    AddThreshold,
    ThresholdAliasChanged(usize, String),
    ThresholdPriceChanged(usize, String),
    UpdateThresholdRow(usize),
    RemoveThresholdRow(usize),
    SortThresholds(SortColumn),
    CheckPrices,
    CheckPricesResult(Result<String, String>),
    SendEmail,
    SendEmailResult(Result<String, String>),
    UpdateCache,
    UpdateCacheResult(Result<String, String>),
    Refresh,
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
    thresholds: Vec<GameThreshold>,
    threshold_alias_edits: Vec<String>,
    threshold_price_edits: Vec<String>,
    threshold_sort_column: Option<SortColumn>,
    threshold_sort_order: SortOrder,
    status_message: String,
    log: String,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            tab: Tab::Search,
            active_view: View::Base,
            settings_view_open: false,
            available_stores: settings::get_available_stores(),
            selected_stores: settings::get_selected_stores(),
            alias_enabled: settings::get_alias_state(),
            alias_reuse_enabled: settings::get_alias_reuse_state(),
            steam_api_key: properties::get_steam_api_key(),
            recipient_email: properties::get_recipient(),
            smtp_host: properties::get_smtp_host(),
            smtp_port: properties::get_smtp_port().to_string(),
            smtp_email: properties::get_smtp_email(),
            smtp_user: properties::get_smtp_user(),
            smtp_password: properties::get_smtp_pwd(),
            project_path: properties::get_project_path(),
            test_path: properties::get_test_path(),
            test_mode: properties::is_testing_enabled(),
            search_query: String::new(),
            add_alias: String::new(),
            add_price: String::new(),
            search_results_by_store: Vec::new(),
            current_store_search_idx: 0,
            selected_results_by_store: HashMap::new(),
            thresholds: thresholds::load_thresholds().unwrap_or_default(),
            threshold_alias_edits: Vec::new(),
            threshold_price_edits: Vec::new(),
            threshold_sort_column: None,
            threshold_sort_order: SortOrder::Original,
            status_message: String::from("Ready"),
            log: String::new(),
        };
        app.sync_threshold_edits();
        app
    }
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.tab = tab;
                self.status_message = format!("Showing {}", tab.label());
                Task::none()
            }
            Message::ViewSelected(view) => {
                self.active_view = view;
                self.status_message = format!("Switched to {:?}", view);
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
                self.status_message = String::from("Updated selected stores");
                Task::none()
            }
            Message::ToggleAliasEnabled(enabled) => {
                self.alias_enabled = enabled;
                settings::update_alias_state(if enabled { 1 } else { 0 });
                self.status_message = String::from("Alias state updated");
                Task::none()
            }
            Message::ToggleAliasReuse(enabled) => {
                self.alias_reuse_enabled = enabled;
                settings::update_alias_reuse_state(if enabled { 1 } else { 0 });
                self.status_message = String::from("Alias reuse state updated");
                Task::none()
            }
            Message::OpenMoreSettings => {
                self.settings_view_open = true;
                self.active_view = View::Settings;
                self.status_message = String::from("Opened more settings view");
                Task::none()
            }
            Message::CloseSettings => {
                self.settings_view_open = false;
                self.active_view = View::Base;
                self.status_message = String::from("Closed settings tab");
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

                self.status_message = match self.threshold_sort_order {
                    SortOrder::Original => format!("Threshold column '{}' is in original order", col_name),
                    SortOrder::Ascending => format!("Threshold column '{}' is in ascending order", col_name),
                    SortOrder::Descending => format!("Threshold column '{}' is in descending order", col_name),
                };
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
            Message::ToggleTestMode(enabled) => {
                self.test_mode = enabled;
                Task::none()
            }
            Message::SearchQueryChanged(value) => {
                self.search_query = value;
                Task::none()
            }
            Message::StartSearch => {
                if self.search_query.trim().is_empty() {
                    self.status_message = String::from("Please enter a search query.");
                    return Task::none();
                }

                let stores_to_search = self.selected_stores.clone();
                self.search_results_by_store.clear();
                self.current_store_search_idx = 0;
                
                for store_id in stores_to_search.iter() {
                    self.search_results_by_store.push((store_id.clone(), Vec::new()));
                }
                
                if self.search_results_by_store.is_empty() {
                    self.status_message = String::from("No stores to search.");
                    return Task::none();
                }
                
                self.status_message = format!("Searching {} stores concurrently...", self.search_results_by_store.len());
                let query = self.search_query.clone();
                
                // Search all selected stores concurrently
                let tasks: Vec<Task<Message>> = self.search_results_by_store.iter().map(|(store_id, _)| {
                    let query = query.clone();
                    let store_id = store_id.clone();
                    Task::perform(perform_store_search(query, store_id.clone()), move |result| {
                        Message::StoreSearchCompleted(store_id, result)
                    })
                }).collect();
                
                Task::batch(tasks)
            }
            Message::SearchResultSelected(selected) => {
                if let Some((store_id, _)) = self.search_results_by_store.get(self.current_store_search_idx) {
                    if selected == SKIP_STORE_SELECTION {
                        self.selected_results_by_store.insert(store_id.clone(), None);
                    } else {
                        self.selected_results_by_store.insert(store_id.clone(), Some(selected));
                    }
                    // self.status_message = String::from("Game selected.");
                }
                Task::none()
            }
            Message::StoreSearchCompleted(store_id, result) => {
                if let Some(entry) = self.search_results_by_store.iter_mut().find(|(id, _)| id == &store_id) {
                    match result {
                        Ok(list) => {
                            entry.1 = list;
                            self.status_message = format!("Search complete for {}", store_id);
                        }
                        Err(err) => {
                            self.status_message = format!("Search failed for {}: {}", store_id, err);
                        }
                    }
                }
                Task::none()
            }
            Message::NextStore => {
                if self.current_store_search_idx < self.search_results_by_store.len() - 1 {
                    self.current_store_search_idx += 1;
                    let (store_id, results) = &self.search_results_by_store[self.current_store_search_idx];
                    if results.is_empty() {
                        let query = self.search_query.clone();
                        let store_id_clone = store_id.clone();
                        self.status_message = format!("Searching {}...", store_id);
                        return Task::perform(perform_store_search(query, store_id_clone.clone()), move |result| {
                            Message::StoreSearchCompleted(store_id_clone, result)
                        });
                    }
                } else {
                    self.status_message = String::from("Reached last store.");
                }
                Task::none()
            }
            Message::PreviousStore => {
                if self.current_store_search_idx > 0 {
                    self.current_store_search_idx -= 1;
                    let (store_id, _) = &self.search_results_by_store[self.current_store_search_idx];
                    self.status_message = format!("Viewing results from {}", store_id);
                }
                Task::none()
            }
            Message::AddThreshold => {
                let price = match self.add_price.trim().parse::<f64>() {
                    Ok(price) => price,
                    Err(_) => {
                        self.status_message = String::from("Invalid desired price. Enter a decimal value.");
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
                    self.status_message = String::from("No selected storefront titles were added. Choose a result before adding a threshold.");
                } else {
                    thresholds::update_thresholds(thresholds_list);
                    for title in added_titles {
                        thresholds::update_threshold_alias(title, &alias);
                    }         
                    self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                    self.sync_threshold_edits();
                    self.search_results_by_store.clear();
                    self.selected_results_by_store.clear();
                    self.current_store_search_idx = 0;
                    self.tab = Tab::Thresholds;
                    self.status_message = format!("Added {} threshold(s) from storefront results.", added_count);
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
                if !self.steam_api_key.is_empty() {
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
                        self.smtp_password.clone(),
                    );
                }
                properties::set_test_mode(self.test_mode);
                self.status_message = String::from("Saved settings");
                Task::none()
            }
            Message::CheckPrices => {
                self.status_message = String::from("Checking prices...");
                Task::perform(check_prices(false), Message::CheckPricesResult)
            }
            Message::CheckPricesResult(result) => {
                match result {
                    Ok(output) => {
                        self.log = output.clone();
                        self.status_message = String::from("Price check complete");
                    }
                    Err(err) => {
                        self.log = err.clone();
                        self.status_message = String::from("Price check failed");
                    }
                }
                self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                Task::none()
            }
            Message::SendEmail => {
                self.status_message = String::from("Sending email...");
                Task::perform(send_sales_email(), Message::SendEmailResult)
            }
            Message::SendEmailResult(result) => {
                match result {
                    Ok(output) => {
                        self.log = output.clone();
                        self.status_message = String::from("Email request complete");
                    }
                    Err(err) => {
                        self.log = err.clone();
                        self.status_message = String::from("Email failed");
                    }
                }
                Task::none()
            }
            Message::UpdateCache => {
                self.status_message = String::from("Updating cache...");
                Task::perform(update_cache(), Message::UpdateCacheResult)
            }
            Message::UpdateCacheResult(result) => {
                match result {
                    Ok(output) => {
                        self.log = output.clone();
                        self.status_message = String::from("Cache update complete");
                    }
                    Err(err) => {
                        self.log = err.clone();
                        self.status_message = String::from("Cache update failed");
                    }
                }
                Task::none()
            }
            Message::ThresholdAliasChanged(idx, value) => {
                if idx == usize::MAX {
                    // alias input from the Add Threshold search form
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
                        let _ = thresholds::update_price(&threshold.title, price);
                    }
                }
                self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                self.sync_threshold_edits();
                self.status_message = String::from("Threshold row updated.");
                Task::none()
            }
            Message::RemoveThresholdRow(idx) => {
                if let Some(title) = self.thresholds.get(idx).map(|threshold| threshold.title.clone()) {
                    let _ = thresholds::remove(&title);
                    self.thresholds = thresholds::load_thresholds().unwrap_or_default();
                    self.sync_threshold_edits();
                    self.status_message = format!("Removed threshold {}.", title);
                }
                Task::none()
            }
            Message::Refresh => {
                self.refresh_state();
                self.status_message = String::from("Refreshed state");
                Task::none()
            }
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
        ))
        .width(280.0)
        .close_on_item_click(true);

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
            (custom_buttons::submenu_button("Selected Stores"), store_selection_menu),
            (custom_buttons::submenu_button("Alias Options"), alias_options_menu),
            (custom_buttons::menu_text_button("More Settings...", Message::OpenMoreSettings)),
        ))
        .width(320.0)
        .close_on_item_click(true);

        let help_menu = Menu::new(menu_items!(
            (custom_buttons::menu_text_button("About", Message::OpenMoreSettings)),
        ))
        .width(Length::Shrink);

        let menu_bar = menu_bar!(
            (container(text("Settings")), settings_menu),
            (container(text("Help")), help_menu),
        )
        .width(Length::Fill)
        .spacing(5.0)
        .padding(Padding::new(4.0))
        .draw_path(menu::DrawPath::Backdrop)
        .close_on_item_click_global(true)
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

        // Top-left menu bar that sits at the very top of the application window
        let top_row = row![
            menu_bar,
        ]
        .width(Length::Fill)
        .spacing(0);

        let tab_bar = {
            let mut bar = row![];

            // if self.active_tab == Tab::Base {
            //     bar = bar.push(container(text("Base")).padding(8));
            // } else {
            //     bar = bar.push(Button::new(text("Base")).on_press(Message::ViewSelected(Tab::Base)).padding(8));
            // }

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
                        .align_y(iced::Alignment::Center),
                    );
                } else {
                    bar = bar.push(Button::new(text("Settings")).on_press(Message::ViewSelected(View::Settings)).padding(8));
                }
            }

            bar.spacing(10).padding(10)
        };

        let right_pane = match self.active_view {
            View::Base => base_view.into(),
            View::Settings => sttngs_view::settings_window(self),
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
        self.steam_api_key = properties::get_steam_api_key();
        self.recipient_email = properties::get_recipient();
        self.smtp_host = properties::get_smtp_host();
        self.smtp_port = properties::get_smtp_port().to_string();
        self.smtp_email = properties::get_smtp_email();
        self.smtp_user = properties::get_smtp_user();
        self.smtp_password = properties::get_smtp_pwd();
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

    fn view_thresholds(&self) -> Element<'_, Message> {
        thrshlds_view::thresholds_tab(self)
    }
}