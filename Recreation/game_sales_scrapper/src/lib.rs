pub mod email {
    pub mod mailer; 
}

pub mod store_api {
    pub mod steam;
    pub mod gog;
    //pub mod humble_bundle;
}

pub mod data {
    pub mod csv;
    pub mod json;
    pub mod settings;
    pub mod thresholds;
}

pub use email::mailer;
pub use store_api::{steam, gog}; //, humble_bundle};
pub use data::{csv, json, settings, thresholds};