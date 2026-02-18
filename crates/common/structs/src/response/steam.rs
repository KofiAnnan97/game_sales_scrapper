use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct App{
    #[serde(rename = "appid")]
    pub app_id: u32,
    pub name: String,
    pub last_modified: i64,
    pub price_change_number: i64,
}

pub struct PriceOverview{
    pub currency: String,
    pub discount_percent: u32,
    pub initial: f64,
    pub final_price: f64,
}