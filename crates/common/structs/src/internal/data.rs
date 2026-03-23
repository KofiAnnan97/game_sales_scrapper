use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct SaleInfo{
    pub icon_link: String,
    pub title: String,
    pub original_price: String,
    pub current_price: String,
    pub discount_percentage: String,
    pub store_page_link: String,
}

#[derive(Debug)]
pub struct SimpleGameThreshold {
    pub name: String,
    pub price: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameThreshold{
    pub title: String,
    pub alias: String,
    pub steam_id: u32,
    pub gog_id: u32,
    pub microsoft_store_id: String,
    pub currency: String,
    pub desired_price: f64,
}