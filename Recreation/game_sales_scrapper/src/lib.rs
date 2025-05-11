pub mod email {
    pub mod mailer; 
}

pub mod store_api {
    pub mod steam;
    pub mod gog;
}

pub mod data {
    pub mod json;
    pub mod settings;
    pub mod thresholds;
}

pub use email::mailer;
pub use store_api::{steam, gog};
pub use data::{json, settings, thresholds};