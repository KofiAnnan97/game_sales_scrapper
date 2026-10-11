use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct App {
    #[serde(rename = "appid")]
    pub app_id: u32,
    pub name: String,
    pub last_modified: i64,
    pub price_change_number: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppDetails {
    pub success: bool,
    #[serde(rename = "data")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub app_data: Option<AppData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppData {
    #[serde(rename = "type")]
    product_type: String,
    pub name: String,
    pub steam_appid: u32,
    required_age: Age,
    is_free: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    controller_support: Option<String>,
    detailed_description: String,
    about_the_game: String,
    short_description: String,
    supported_languages: String,
    pub header_image: String,
    capsule_image: String,
    capsule_imagev5: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    website: Option<String>,
    pc_requirements: Requirements,
    mac_requirements: Requirements,
    linux_requirements: Requirements,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    legal_notice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub price_overview: Option<PriceOverview>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Age {
    Str(String),
    Int(u32),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Requirements {
    Vec(Vec<String>),
    Map(HashMap<String, String>),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppPrices {
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty", default)]
    pub prices: HashMap<String, ResponseData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseData {
    pub success: bool,
    pub data: PriceData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PriceData {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub price_overview: Option<PriceOverview>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PriceOverview {
    pub currency: String,
    pub discount_percent: u32,
    pub initial: f64,
    #[serde(rename = "final")]
    pub final_price: f64,
}
