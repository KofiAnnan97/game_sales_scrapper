pub mod email {
    pub mod mailer; 
}

pub mod store_api {
    pub mod steam;
    //pub mod gog;
}

pub use email::mailer;
pub use store_api::steam; //{steam, gog};