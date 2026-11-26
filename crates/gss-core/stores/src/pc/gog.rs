use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::result::Result;
// use mockall::automock;
use tokio::time::Duration;

use constants::stores::gog::*;
use errors::api::ApiError;
use types::internal::data::SaleInfo;
use types::response::gog::{Game, GameInfo, PriceOverview};

// #[automock]
#[async_trait]
pub trait GogApi {
    async fn search_game_by_title(&self, title: &str) -> Result<Vec<Game>, ApiError>;
    async fn get_price_details(&self, title: &str) -> Result<PriceOverview, ApiError>;
    async fn search_game_by_title_v2(
        &self,
        title: &str,
        limit: u32,
    ) -> Result<Vec<GameInfo>, ApiError>;
    async fn get_game_data(&self, title: &str) -> Result<GameInfo, ApiError>;
    async fn get_price_details_v2(&self, title: &str) -> Result<SaleInfo, ApiError>;
}

pub struct GogClient<'a> {
    http_client: &'a reqwest::Client,
}

impl<'a> GogClient<'a> {
    pub fn new(http_client: &'a reqwest::Client) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl<'a> GogApi for GogClient<'a> {
    async fn search_game_by_title(&self, title: &str) -> Result<Vec<Game>, ApiError> {
        let media_type = "game";
        let limit: i32 = 30;
        let url = format!(
            "{}{}?mediaType={}&search={}&limit={}",
            BASE_URL_V1, MEDIA_ENDPOINT_V1, media_type, title, limit
        );
        let resp = self
            .http_client
            .get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .send()
            .await?
            .text()
            .await?;
        let body: Value = serde_json::from_str(&resp)?;
        //println!("{:?}", body);
        let products = serde_json::to_string(&body["products"])?;
        let games_list: Vec<Game> = serde_json::from_str::<Vec<Game>>(&products)?;
        Ok(games_list)
    }

    async fn get_price_details(&self, title: &str) -> Result<PriceOverview, ApiError> {
        let media_type = "game";
        let limit_num: i32 = 30;
        let url = format!(
            "{}{}?mediaType={}&search={}&limit={}",
            BASE_URL_V1, MEDIA_ENDPOINT_V1, media_type, title, limit_num
        );
        let resp = self
            .http_client
            .get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .send()
            .await?
            .text()
            .await?;
        let body: Value = serde_json::from_str(&resp)?;
        //println!("{:?}", body);
        let products = body["products"]
            .as_array()
            .ok_or(ApiError::Message(format!(
                "Could not find price details for {}",
                title
            )))?;

        for product in products {
            let game_title = product["title"].to_string();
            if *title == game_title[1..game_title.len() - 1] {
                let price = serde_json::to_string(&product["price"])?;
                let price_overview = serde_json::from_str::<PriceOverview>(&price)?;
                return Ok(price_overview);
            }
        }
        Err(ApiError::Message(format!(
            "Could not find price details for {}",
            title
        )))
    }

    async fn search_game_by_title_v2(
        &self,
        title: &str,
        limit: u32,
    ) -> Result<Vec<GameInfo>, ApiError> {
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
        let resp = self
            .http_client
            .get(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
            .query(&query_string)
            .send()
            .await?
            .text()
            .await?;

        let body: serde_json::Value = serde_json::from_str(&resp)?;
        let products = body
            .get("products")
            .ok_or(ApiError::Message(MISSING_PRODUCTS_MSG.into()))?;
        let games: Vec<GameInfo> = Vec::deserialize(products)?;
        Ok(games)
    }

    async fn get_game_data(&self, title: &str) -> Result<GameInfo, ApiError> {
        match self.search_game_by_title_v2(title, SINGLE_SEARCH).await {
            Ok(products) => {
                if products.len() == 1 {
                    Ok(products[0].clone())
                } else {
                    Err(ApiError::Message(String::from(
                        "Search results did not return 1 game entry",
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn get_price_details_v2(&self, title: &str) -> Result<SaleInfo, ApiError> {
        match self.get_game_data(title).await {
            Ok(data) => match data.price {
                Some(po) => {
                    let discount_str;
                    if let Some(discount) = po.discount {
                        discount_str = discount[1..discount.len() - 1].to_string()
                    } else {
                        let base_amount = po.base_money.amount.parse::<f64>().map_err(|e| {
                            ApiError::Message(format!("Base price could not be parsed. {}", e))
                        })?;
                        let final_amount = po.final_money.amount.parse::<f64>().map_err(|e| {
                            ApiError::Message(format!("Final price could not be parsed. {}", e))
                        })?;
                        discount_str = format!(
                            "{}",
                            (100.0 * (1.0 - final_amount / base_amount)).round() as i64
                        )
                    };
                    Ok(SaleInfo {
                        title: data.title,
                        original_price: po.base_money.amount.parse::<f64>().unwrap_or(f64::MIN),
                        current_price: po.final_money.amount.parse::<f64>().unwrap_or(f64::MAX),
                        discount_percentage: discount_str,
                        icon_link: data.c_horizontal,
                        store_page_link: data.store_link,
                    })
                }
                None => Err(ApiError::Message(format!(
                    "Could not retrieve price details for {}",
                    title
                ))),
            },
            Err(e) => Err(e),
        }
    }
}

pub fn get_price_from_list(title: &str, games_list: Vec<Game>) -> Result<f64, ApiError> {
    for game in games_list.iter() {
        if title == game.title {
            let game_price: f64 = game
                .price
                .final_amount
                .parse::<f64>()
                .map_err(|e| ApiError::Message(format!("{}", e)))?;
            return Ok(game_price);
        }
    }
    Err(ApiError::Message(format!(
        "Could not find price details for {}",
        title
    )))
}

// Version 1
pub async fn search_game_by_title(
    title: &str,
    http_client: &reqwest::Client,
) -> Result<Vec<Game>, ApiError> {
    GogClient::new(http_client)
        .search_game_by_title(title)
        .await
}

pub async fn get_price_details(
    title: &str,
    http_client: &reqwest::Client,
) -> Result<PriceOverview, ApiError> {
    GogClient::new(http_client).get_price_details(title).await
}

// Version 2
pub async fn search_game_by_title_v2(
    title: &str,
    http_client: &reqwest::Client,
) -> Result<Vec<GameInfo>, ApiError> {
    GogClient::new(http_client)
        .search_game_by_title_v2(title, SEARCH_LIMIT)
        .await
}

pub async fn get_price_details_v2(
    title: &str,
    http_client: &reqwest::Client,
) -> Result<SaleInfo, ApiError> {
    GogClient::new(http_client)
        .get_price_details_v2(title)
        .await
}
