use std::{path::PathBuf, sync::Arc};

use iced::window;

use crate::{
    app::enums::*,
    utils::log_utils::LogLevel,
    views::{
        logs::{LoggingMessage, Screen},
        preview::PreviewMessage,
        settings::Page,
    },
};
use types::internal::store::GameStore;

#[derive(Debug, Clone)]
pub enum MainMessage {
    MainWindowOpened(window::Id),
    TabSelected(Tab),
    OpenBaseView,
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
    LogEvent(Screen, LogLevel, String),
    RefreshLogsView,
    SetDefaultLogLevel(String),
    _UpdateLoggingSettings(usize, usize, usize),
}

#[derive(Debug, Clone)]
pub enum Message {
    CloseWindow(window::Id),
    Main(MainMessage),
    Preview(PreviewMessage),
    Logging(LoggingMessage),
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

impl From<LoggingMessage> for Message {
    fn from(msg: LoggingMessage) -> Self {
        Message::Logging(msg)
    }
}
