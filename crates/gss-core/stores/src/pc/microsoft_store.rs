use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{self, Value};
// use mockall::automock;
use std::result::Result;
use tokio::time::Duration;

use constants::stores::microsoft_store::*;
use errors::api::ApiError;
use types::internal::data::SaleInfo;
use types::response::microsoft_store::{GameInfo, ProductInfo};

// #[automock]
#[async_trait]
pub trait MicrosoftStoreApi {
    async fn search_game_by_title(&self, title: &str) -> Result<Vec<ProductInfo>, ApiError>;
    async fn get_game_data(&self, xbox_id: &str) -> Result<GameInfo, ApiError>;
    async fn get_price_using_search(&self, title: &str, xbox_id: &str) -> Option<SaleInfo>;
    async fn get_price_details(&self, xbox_id: &str) -> Option<SaleInfo>;
}

pub struct MSClient<'a> {
    http_client: &'a reqwest::Client,
}

impl<'a> MSClient<'a> {
    pub fn new(http_client: &'a reqwest::Client) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl<'a> MicrosoftStoreApi for MSClient<'a> {
    async fn search_game_by_title(&self, title: &str) -> Result<Vec<ProductInfo>, ApiError> {
        let query_string = [
            ("query", title),
            ("mediaType", "games"),
            ("age", "all"),
            ("price", "all"),
            ("category", "all"),
            ("subscription", "none"),
            ("gl", "US"),
            ("hl", "en-US"),
        ];
        let url = format!("{}{}", BASE_URL, SEARCH_ENDPOINT);
        let resp = self
            .http_client
            .get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .query(&query_string)
            .send()
            .await?
            .text()
            .await?;
        let body: Value = serde_json::from_str(&resp)?;
        let products = body
            .get("productsList")
            .ok_or(ApiError::Message(MISSING_PRODUCTS_MSG.into()))?;
        let game_list: Vec<ProductInfo> = Vec::deserialize(products)?;
        Ok(game_list)
    }

    async fn get_game_data(&self, xbox_id: &str) -> Result<GameInfo, ApiError> {
        let query_string = [("productId", xbox_id), ("gl", "US"), ("hl", "en-US")];
        let url = format!("{}{}", BASE_URL, PDP_ENDPOINT);
        let resp = self
            .http_client
            .get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .query(&query_string)
            .send()
            .await?
            .text()
            .await?;

        let game = serde_json::from_str::<GameInfo>(&resp)?;
        Ok(game)
    }

    async fn get_price_using_search(&self, title: &str, xbox_id: &str) -> Option<SaleInfo> {
        let search_list: Vec<ProductInfo> = self
            .search_game_by_title(title)
            .await
            .unwrap_or_else(|_e| Vec::new());
        for game in search_list {
            if game.product_id == xbox_id {
                let mut discount_str = game.price_info.badge_text.unwrap_or_default();
                discount_str = if !discount_str.is_empty() {
                    discount_str[1..discount_str.len() - 1].to_string()
                } else {
                    String::from("0")
                };
                return Some(SaleInfo {
                    icon_link: game.box_icon_url.clone(),
                    title: game.title.clone(),
                    original_price: game.price_info.msrp.unwrap_or(f64::MIN),
                    current_price: game.price_info.price.unwrap_or(f64::MAX),
                    discount_percentage: discount_str,
                    store_page_link: game.redirect_url.unwrap_or_default(),
                });
            }
        }
        None
    }

    async fn get_price_details(&self, xbox_id: &str) -> Option<SaleInfo> {
        match self.get_game_data(xbox_id).await {
            Ok(game) => {
                let mut discount_str = game.price_info.badge_text.unwrap_or_default();
                discount_str = if !discount_str.is_empty() {
                    discount_str[1..discount_str.len() - 1].to_string()
                } else {
                    String::from("0")
                };
                return Some(SaleInfo {
                    icon_link: game.box_icon_url.clone(),
                    title: game.title.clone(),
                    original_price: game.price_info.msrp.unwrap_or(f64::MIN),
                    current_price: game.price_info.price.unwrap_or(f64::MAX),
                    discount_percentage: discount_str,
                    store_page_link: game.redirect_url.unwrap_or_default(),
                });
            }
            Err(_) => None,
        }
    }
}

pub async fn search_game_by_title(
    title: &str,
    http_client: &reqwest::Client,
) -> Result<Vec<ProductInfo>, ApiError> {
    MSClient::new(http_client).search_game_by_title(title).await
}

pub async fn get_price_using_search(
    title: &str,
    xbox_id: &str,
    http_client: &reqwest::Client,
) -> Option<SaleInfo> {
    MSClient::new(http_client)
        .get_price_using_search(title, xbox_id)
        .await
}

pub async fn get_price_details(xbox_id: &str, http_client: &reqwest::Client) -> Option<SaleInfo> {
    MSClient::new(http_client).get_price_details(xbox_id).await
}
