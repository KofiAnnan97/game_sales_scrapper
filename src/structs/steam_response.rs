use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Game{
    #[serde(rename = "appid")]
    pub app_id: usize,
    pub name: String,
}

pub struct PriceOverview{
    pub currency: String,
    pub discount_percent: usize,
    pub initial: f64,
    pub final_price: f64,
}