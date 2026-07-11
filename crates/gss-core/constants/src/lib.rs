pub mod cli {
    pub mod args;
}

pub mod operations {
    pub mod properties;
    pub mod settings;
    pub mod thresholds;
    pub mod logging;
}

pub mod stores {
    pub mod gog;
    pub mod steam;
    pub mod microsoft_store;
}