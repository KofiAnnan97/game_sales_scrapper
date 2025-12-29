use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct App{
    #[serde(rename = "appid")]
    pub app_id: usize,
    pub name: String,
    pub last_modified: i64,
    pub price_change_number: i64,
}

pub struct PriceOverview{
    pub currency: String,
    pub discount_percent: usize,
    pub initial: f64,
    pub final_price: f64,
}