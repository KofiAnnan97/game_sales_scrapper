pub mod alerting {
    pub mod email; 
}

pub mod stores {
    pub mod steam;
    pub mod gog;
    pub mod microsoft_store;
}

pub mod file_ops {
    pub mod structs;
    pub mod csv;
    pub mod json;
    pub mod settings;
    pub mod thresholds;
}

pub use alerting::email;
pub use stores::{steam, gog, microsoft_store};
pub use file_ops::{structs, csv, json, settings, thresholds};