use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tab {
    Search,
    Thresholds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Base,
    Settings,
    Preview,
    Logs,
}

#[derive(Debug, Clone)]
pub enum StoreSearchResult {
    Steam { title: String, steam_id: u32 },
    Gog { title: String, gog_id: u32 },
    Microsoft { title: String, ms_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Title,
    Alias,
    SteamId,
    GogId,
    MicrosoftId,
    DesiredPrice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Original,
    Ascending,
    Descending,
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    #[allow(unused)]
    IoError(io::ErrorKind),
}

impl StoreSearchResult {
    pub fn title(&self) -> &str {
        match self {
            StoreSearchResult::Steam { title, .. } => title,
            StoreSearchResult::Gog { title, .. } => title,
            StoreSearchResult::Microsoft { title, .. } => title,
        }
    }

    pub fn ids(&self) -> (u32, u32, String) {
        match self {
            StoreSearchResult::Steam { steam_id, .. } => (*steam_id, 0, String::new()),
            StoreSearchResult::Gog { gog_id, .. } => (0, *gog_id, String::new()),
            StoreSearchResult::Microsoft { ms_id, .. } => (0, 0, ms_id.clone()),
        }
    }
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Search => "Search",
            Tab::Thresholds => "Thresholds",
        }
    }
}
