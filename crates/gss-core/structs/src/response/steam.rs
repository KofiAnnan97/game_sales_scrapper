use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct App{
    #[serde(rename = "appid")]
    pub app_id: u32,
    pub name: String,
    pub last_modified: i64,
    pub price_change_number: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppDetails{
    pub success: bool,
    #[serde(rename = "data")]
    pub app_data: Option<AppData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppData{
    #[serde(rename = "type")]
    product_type: String,
    pub name: String,
    pub steam_appid: u64,
    required_age: Age,
    is_free: bool,
    controller_support: Option<String>,
    detailed_description: String,
    about_the_game: String,
    short_description: String, 
    supported_languages: String,
    pub header_image: String,
    capsule_image: String,
    capsule_imagev5: String,
    website: Option<String>,
    pc_requirements: Requirements,
    mac_requirements: Requirements,
    linux_requirements: Requirements,
    legal_notice: Option<String>,
    pub price_overview: Option<PriceOverview>
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Age{
   Str(String),
   Int(u32)
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Requirements{
    Vec(Vec<String>),
    Map(HashMap<String, String>)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PriceOverview{
    pub currency: String,
    pub discount_percent: u32,
    pub initial: f64,
    #[serde(rename = "final")]
    pub final_price: f64,
}