use serde_json::{Result, Value};
use async_trait::async_trait;
use mockall::automock;
use tokio::time::{Duration};

use structs::internal::data::SaleInfo;
use structs::response::microsoft_store::{ProductInfo, GameInfo};
use constants::stores::microsoft_store::*;

#[automock]
#[async_trait]
pub trait MicrosoftStoreApi {
    async fn search_game_by_title(&self, title: &str) -> Result<Vec<ProductInfo>>;
    async fn get_price_using_search(&self, title: &str, xbox_id: &str) -> Option<SaleInfo>;
    async fn get_price_details(&self, xbox_id: &str) -> Option<SaleInfo>;
}

pub struct MSClient {
    http_client: reqwest::Client,
}

impl MSClient {
    pub fn new() -> Self {
        Self { http_client: reqwest::Client::new() }
    }

    pub fn with_client(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl MicrosoftStoreApi for MSClient{
    async fn search_game_by_title(&self, title: &str) -> Result<Vec<ProductInfo>> {
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
        let resp = self.http_client.get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .query(&query_string)
            .send()
            .await
            .expect("Failed to get response")
            .text()
            .await
            .expect("Failed to get data");
        let body: Value = serde_json::from_str(&resp).expect("Could not convert Microsoft Store search to JSON");
        let products = serde_json::to_string(&body["productsList"]).unwrap();
        //println!("{:?}", products);
        let game_list = serde_json::from_str::<Vec<ProductInfo>>(&products)?;
        Ok(game_list)
    }

    async fn get_price_using_search(&self, title: &str, xbox_id: &str) -> Option<SaleInfo> {
            let search_list: Vec<ProductInfo> = self.search_game_by_title(title).await.unwrap_or_else(|_e| Vec::new());
            for game in search_list {
                if game.product_id == xbox_id {
                let mut discount_str = game.price_info.badge_text.unwrap_or_default();
                discount_str = if !discount_str.is_empty() {
                    discount_str[1..discount_str.len()-1].to_string()
                }else{
                    String::from("0")
                };
                return Some(SaleInfo{
                    icon_link: game.box_icon_url.clone(),
                    title: game.title.clone(),
                    original_price: format!("{}", game.price_info.msrp.unwrap_or_default()),
                    current_price: format!("{}", game.price_info.price.unwrap_or_default()),
                    discount_percentage: discount_str,
                    store_page_link: game.redirect_url.unwrap_or_default(),
                });
                }
            }
            None
    }

    async fn get_price_details(&self, xbox_id: &str) -> Option<SaleInfo> {
            let query_string = [
                ("productId", xbox_id),
                ("gl", "US"),
                ("hl", "en-US"),
            ];
            let url = format!("{}{}", BASE_URL, PDP_ENDPOINT);
            let resp = self.http_client.get(url)
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
                .query(&query_string)
                .send()
                .await
                .expect("Failed to get response")
                .text()
                .await
                .expect("Failed to get data");
            let body: Value = serde_json::from_str(&resp).expect("Could not convert Microsoft Store game data to JSON");
            if let Some(data_str) = body.as_object() {
                let game_str = serde_json::to_string(data_str).unwrap();
                //println!("{:?}", game_str);
                let game = serde_json::from_str::<GameInfo>(&game_str).unwrap();
                let mut discount_str = game.price_info.badge_text.unwrap_or_default();
                discount_str = if !discount_str.is_empty() {
                    discount_str[1..discount_str.len()-1].to_string()
                }else{
                    String::from("0")
                };
                return Some(SaleInfo{
                    icon_link: game.box_icon_url.clone(),
                    title: game.title.clone(),
                    original_price: format!("{}", game.price_info.msrp.unwrap_or_default()),
                    current_price: format!("{}", game.price_info.price.unwrap_or_default()),
                    discount_percentage: discount_str,
                    store_page_link: game.redirect_url.unwrap_or_default(),
                });
            }
            None
    }
}

pub async fn search_game_by_title(title: &str, http_client: &reqwest::Client) -> Result<Vec<ProductInfo>> {
    MSClient::with_client(http_client.clone()).search_game_by_title(title).await
}

pub async fn get_price_using_search(title: &str, xbox_id :&str, http_client: &reqwest::Client) -> Option<SaleInfo> {
    MSClient::with_client(http_client.clone()).get_price_using_search(title, xbox_id).await
}

pub async fn get_price_details(xbox_id: &str, http_client: &reqwest::Client) -> Option<SaleInfo> {
    MSClient::with_client(http_client.clone()).get_price_details(xbox_id).await
}