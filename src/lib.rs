pub mod alerting {
    pub mod email; 
}

pub mod stores {
    pub mod steam;
    pub mod gog;
    //pub mod humble_bundle;
}

pub mod file_ops {
    pub mod structs;
    pub mod csv;
    pub mod json;
    pub mod settings;
    pub mod thresholds;
}

pub use alerting::email;
pub use stores::{steam, gog}; //, humble_bundle};
pub use file_ops::{structs, csv, json, settings, thresholds};