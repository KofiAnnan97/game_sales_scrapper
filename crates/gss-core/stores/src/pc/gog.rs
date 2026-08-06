use serde_json::{Value};
use serde::Deserialize;
use std::{f64, format};
use async_trait::async_trait;
// use mockall::automock;
use tokio::time::{Duration};

use types::internal::data::{SaleInfo};
use types::response::gog::{Game, PriceOverview, GameInfo};
use constants::stores::gog::*;
use errors::api::ApiError;

// #[automock]
#[async_trait]
pub trait GogApi {
    async fn search_game_by_title(&self, title: &str) -> serde_json::Result<Vec<Game>>;
    async fn get_price_details(&self, title: &str) -> Option<PriceOverview>;
    async fn search_game_by_title_v2(&self, title: &str, limit: u32) -> Result<Vec<GameInfo>, ApiError>;
    async fn get_game_data(&self, title: &str) -> Result<GameInfo, ApiError>;
    async fn get_price_details_v2(&self, title: &str) -> Option<SaleInfo>;
}

pub struct GogClient {
    http_client: reqwest::Client,
}

impl GogClient {
    pub fn new() -> Self {
        Self { http_client: reqwest::Client::new() }
    }
    pub fn with_client(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl GogApi for GogClient {
    async fn search_game_by_title(&self, title: &str) -> serde_json::Result<Vec<Game>> {
        let media_type = "game";
        let limit :i32 = 30;
        let url = format!("{}{}?mediaType={}&search={}&limit={}", BASE_URL_V1, MEDIA_ENDPOINT_V1, media_type, title, limit);
        let resp = self.http_client.get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .send()
            .await
            .expect("Failed to get response")
            .text()
            .await
            .expect("Failed to get data");
        let body : Value = serde_json::from_str(&resp).expect("Could not convert to JSON");
        //println!("{:?}", body);
        let products = serde_json::to_string(&body["products"]).unwrap();
        let games_list : Vec<Game> = serde_json::from_str::<Vec<Game>>(&products)?;
        Ok(games_list)
    }

    async fn get_price_details(&self, title: &str) -> Option<PriceOverview> {
        let http_client = reqwest::Client::new();
        let media_type = "game";
        let limit_num : i32 = 30;
        let url = format!("{}{}?mediaType={}&search={}&limit={}", BASE_URL_V1, MEDIA_ENDPOINT_V1, media_type, title, limit_num);
        let resp = http_client.get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .send()
            .await
            .expect("Failed to get response")
            .text()
            .await
            .expect("Failed to get data");
        let body: Value = serde_json::from_str(&resp).expect("Could not convert to JSON");
        //println!("{:?}", body);
        if let Some(products) = body["products"].as_array() {
            for idx in 0..products.len(){
                let game_title = products[idx]["title"].to_string();
                if title.to_string() == game_title[1..game_title.len()-1].to_string(){
                    let price = serde_json::to_string(&products[idx]["price"]).unwrap();
                    let price_overview = serde_json::from_str::<PriceOverview>(&price).unwrap();
                    return Some(price_overview);
                }
            }
        }
        None
    }

    async fn search_game_by_title_v2(&self, title: &str, limit: u32) -> Result<Vec<GameInfo>, ApiError> {
        let like_title = format!("like:{}", title);
        let query_string = [
            ("query", like_title.as_str()),
            ("limit", &limit.to_string()),
            ("order", "desc:score"),
            ("productType", "in:game"),
            ("page", "1"),
            ("countryCode", "US"),
            ("locale", "en-US"),
            ("currencyCode", "USD"),
        ];
        let url = format!("{}{}", BASE_URL_V2, CATALOG_ENDPOINT_V2);
        let resp = self.http_client.get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .query(&query_string)
            .send()
            .await?
            .text()
            .await?;

        let body: serde_json::Value = serde_json::from_str(&resp)?;
        let products = body.get("products").ok_or(ApiError::Message(format!("{}", MISSING_PRODUCTS_MSG)))?;
        let games: Vec<GameInfo> = Vec::deserialize(products)?;
        Ok(games)        
    }

    async fn get_game_data(&self, title: &str) -> Result<GameInfo, ApiError>{
        match self.search_game_by_title_v2(title, SINGLE_SEARCH).await {
            Ok(products) => {
                if products.len() == 1 {
                    Ok(products[0].clone())
                } else {
                    Err(ApiError::Message(String::from("Search results did not return 1 game entry")))
                }
            },
            Err(e) => Err(e)
        }
    }

    async fn get_price_details_v2(&self, title: &str) -> Option<SaleInfo> {
        match self.get_game_data(title).await {
            Ok(data) => match data.price {
                Some(po) => {
                    let discount_str;
                    if let Some(discount) = po.discount {
                       discount_str = discount[1..discount.len()-1].to_string()
                    } else { 
                        let base_amount = po.base_money.amount.parse::<f64>().unwrap();
                        let final_amount = po.final_money.amount.parse::<f64>().unwrap();
                        discount_str = format!("{}", (100.0*(1.0-final_amount/base_amount)).round() as i64)
                    };
                    return Some(SaleInfo{
                        title: data.title,
                        original_price: po.base_money.amount.parse::<f64>().unwrap_or_else(|_| f64::MIN),
                        current_price: po.final_money.amount.parse::<f64>().unwrap_or_else(|_| f64::MAX), 
                        discount_percentage: discount_str,
                        icon_link: data.c_horizontal,
                        store_page_link: data.store_link,
                    });
                },
                None => None,
            },
            Err(_) => None
        }
    }
}

pub fn get_price_from_list(title:&str, games_list: Vec<Game>) -> Option<f64> {
    for game in games_list.iter(){
        if title == &game.title {
            let game_price : f64 = game.price.final_amount.parse::<f64>().unwrap();
            return Some(game_price);
        } 
    }
    None
}

// Version 1
pub async fn search_game_by_title(title: &str) -> serde_json::Result<Vec<Game>> {
   GogClient::new().search_game_by_title(title).await
}

pub async fn get_price_details(title: &str) -> Option<PriceOverview> {
    GogClient::new().get_price_details(title).await
}

// Version 2
pub async fn search_game_by_title_v2(title: &str, http_client: &reqwest::Client) -> std::result::Result<Vec<GameInfo>, ApiError>{
    GogClient::with_client(http_client.clone()).search_game_by_title_v2(title, SEARCH_LIMIT).await
}

pub async fn get_price_details_v2(title: &str, http_client: &reqwest::Client) -> Option<SaleInfo> {
    GogClient::with_client(http_client.clone()).get_price_details_v2(title).await
}