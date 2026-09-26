use std::collections::HashMap;
use std::path::PathBuf;

use iced::window;
use types::internal::data::{GameThreshold, SimpleGameThreshold};
use types::internal::store::GameStore;

use crate::app::enums::*;
use crate::utils::log_utils::{Logger, new_log};
use crate::views::logs::LoggingView;
use crate::views::preview::PreviewView;
use crate::views::settings::Page;

pub struct App {
    // General
    pub main_window: Option<window::Id>,
    pub tab: Tab,
    pub active_view: View,
    pub logger: Logger,
    pub view_bar: Vec<View>,
    // Settings
    pub available_stores: Vec<GameStore>,
    pub selected_stores: Vec<GameStore>,
    pub alias_enabled: bool,
    pub alias_reuse_enabled: bool,
    pub reveal_sensitive_data: bool,
    pub steam_api_key: String,
    pub recipient_email: String,
    pub smtp_host: String,
    pub smtp_port: String,
    pub smtp_email: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub project_path: String,
    pub test_path: String,
    pub test_mode: bool,
    pub auto_advance_enabled: bool,
    pub default_log_level: String,
    //Search
    pub search_query: String,
    pub add_alias: String,
    pub add_price: String,
    pub search_results_by_store: Vec<(GameStore, Vec<StoreSearchResult>)>,
    pub current_store_search_idx: usize,
    pub selected_results_by_store: HashMap<GameStore, Option<usize>>,
    pub is_search_in_progress: bool,
    pub is_caching_in_progress: bool,
    pub pending_searches: usize,
    pub search_loading_frame: usize,
    pub caching_loading_frame: usize,
    pub selected_file: Option<PathBuf>,
    pub bulk_simple_threshs: Vec<SimpleGameThreshold>,
    pub bulk_search_index: usize,
    pub bulk_search_used: bool,
    pub thresholds: Vec<GameThreshold>,
    pub threshold_alias_edits: Vec<String>,
    pub threshold_price_edits: Vec<String>,
    pub threshold_sort_column: Option<SortColumn>,
    pub threshold_sort_order: SortOrder,
    pub show_dialog: bool,
    pub preview_view: PreviewView,
    pub logging_view: LoggingView,
    // Settings variables
    pub settings_page: Page,
    pub store_settings_expanded: bool,
    // Logging variables
    pub status_message: String,
    pub message_details: String,
    pub manual_prune_window: Option<window::Id>,
    pub manual_prune_open: bool,
}

impl Default for App {
    fn default() -> App {
        Self::new(new_log()).0
    }
}
