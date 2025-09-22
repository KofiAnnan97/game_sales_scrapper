use serde_json::{Result, Value, Error};
use serde::{Deserialize, Serialize};
use std::f64;
use std::collections::HashMap;

pub static VERSION: u32 = 2;

// Version 1

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    pub title: String,
    pub id: u64,
    pub price: PriceOverview,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PriceOverview {
    pub currency: String,
    pub amount: String,
    #[serde(rename="baseAmount")]
    pub base_amount: String,
    #[serde(rename="finalAmount")]
    pub final_amount: String,
    #[serde(rename="isDiscounted")]
    pub is_discounted: bool,
    #[serde(rename="discountPercentage")]
    pub discount_percentage: i32,
    #[serde(rename="discountDifference")]
    discount_diff: String,
    pub symbol: String,
    #[serde(rename="isFree")]
    pub is_free: bool,
    pub discount: i32,
    #[serde(rename="isBonusStoreCreditIncluded")]
    is_bonus_credit_included: bool,
    #[serde(rename="bonusStoreCreditAmount")]
    bonus_credit_amount: String
}

pub async fn search_game_by_title(title: &str) -> Result<Vec<Game>> {
    let http_client = reqwest::Client::new();
    let media_type = "game";
    let limit :i32 = 30;
    let url = format!("https://embed.gog.com/games/ajax/filtered?mediaType={}&search={}&limit={}", media_type, title, limit);
    //println!("{}", url);
    let resp = http_client.get(url)
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

pub fn get_price_from_list(title:&str, games_list: Vec<Game>) -> Option<f64> {
    for game in games_list.iter(){
        if title == &game.title {
            let game_price : f64 = game.price.final_amount.parse::<f64>().unwrap();
            return Ok::<f64, Error>(game_price).ok();
        } 
    }
    None
}

pub async fn get_price(title: &str) -> Option<PriceOverview> {
    let http_client = reqwest::Client::new();
    let media_type = "game";
    let limit_num : i32 = 30;
    let url = format!("https://embed.gog.com/games/ajax/filtered?mediaType={}&search={}&limit={}", media_type, title, limit_num);
    //println!("{}", url);
    let resp = http_client.get(url)
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
                return Ok::<PriceOverview, Error>(price_overview).ok();
            }
        }
    }
    None
}

// Version 2

static BASE_URL : &str = "https://catalog.gog.com";
static CATALOG_ENDPOINT : &str = "/v1/catalog";

#[derive(Deserialize, Serialize, Debug)]
pub struct GameInfo {
    pub id: String,
    pub title: String,
    pub price: Option<Price>,
    #[serde(rename="coverHorizontal", skip)]
    c_horizontal: String,
    #[serde(rename="coverVertical", skip)]
    c_vertical: String,
    #[serde(skip)]
    developers: Vec<String>,
    #[serde(skip)]
    editions: Vec<String>,
    #[serde(skip)]
    features: Vec<HashMap<String, String>>,
    #[serde(skip)]
    genres: Vec<HashMap<String, String>>,
    #[serde(rename="operatingSystems", skip)]
    os: Vec<String>,
    #[serde(rename="productState", skip)]
    product_state: String,
    #[serde(rename="productType", skip)]
    product_type: String,
    #[serde(skip)]
    publishers: Vec<String>,
    #[serde(skip)]
    ratings: Vec<HashMap<String, String>>,
    #[serde(rename="releaseDate", skip)]
    release_date: String,
    #[serde(rename="reviewsRating", skip)]
    reviews_rating: u32,
    #[serde(skip)]
    screenshots: Vec<String>,
    #[serde(skip)]
    slug: String,
    #[serde(rename="storeLink", skip)]
    store_link: String,
    #[serde(rename="storeReleaseDate", skip)]
    store_release_date: String,
    #[serde(skip)]
    tags: Vec<HashMap<String,String>>,
    #[serde(rename="userPreferredLanguage", skip)]
    user_pref_lang: UserPreferredLanguage,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UserPreferredLanguage{
    pub code: String,
    #[serde(rename="inAudio")]
    pub in_audio: bool,
    #[serde(rename="inText")]
    pub in_text: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Price {
    #[serde(rename="final")]
    pub final_price: String,
    #[serde(rename="base")]
    pub base_price: String,
    pub discount: Option<String>,
    #[serde(rename="finalMoney")]
    pub final_money: FinalMoney,
    #[serde(rename="baseMoney")]
    pub base_money: BaseMoney,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BaseMoney {
    pub amount: String,
    pub currency: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FinalMoney {
    pub amount: String,
    pub currency: String,
    pub discount: String,
}

pub async fn search_game_by_title_v2(title: &str, http_client: &reqwest::Client) -> Result<Vec<GameInfo>>{
    let limit = 48;
    let order = "desc:score";
    let product_type = "in:game";
    let page = 1;
    let country_code = "US";
    let locale = "en-US";
    let currency = "USD";
    let url = format!("{}{}?limit={}&query=like:{}&order={}&productType={}&page={}&countryCode={}&locale={}&currencyCode={}", 
                      BASE_URL, CATALOG_ENDPOINT, limit, title, order, product_type, page, country_code, locale, currency);
    let resp = http_client.get(url)
        .send()
        .await
        .expect("Failed to get response")
        .text()
        .await
        .expect("Failed to get data");
    let body : Value = serde_json::from_str(&resp).expect("Could not convert GOG search to JSON");
    let products = serde_json::to_string(&body["products"]).unwrap();
    let games_list : Vec<GameInfo> = serde_json::from_str::<Vec<GameInfo>>(&products)?;
    Ok(games_list)    
}

pub async fn get_price_v2(title: &str, http_client: &reqwest::Client) -> Option<Price> {
    let limit = 1;
    let order = "desc:score";
    let product_type = "in:game";
    let page = 1;
    let country_code = "US";
    let locale = "en-US";
    let currency = "USD";
    let url = format!("{}{}?limit={}&query=like:{}&order={}&productType={}&page={}&countryCode={}&locale={}&currencyCode={}", 
                      BASE_URL, CATALOG_ENDPOINT, limit, title, order, product_type, page, country_code, locale, currency);
    let resp = http_client.get(url)
        .send()
        .await
        .expect("Failed to get response")
        .text()
        .await
        .expect("Failed to get data");
    let body: Value = serde_json::from_str(&resp).expect("Could not convert to JSON");
    if let Some(products) = body["products"].as_array() {
        let price_data = serde_json::to_string(&products[0]["price"]).unwrap();
        let price = serde_json::from_str::<Price>(&price_data).unwrap();
        return Ok::<Price, Error>(price).ok();
    }
    None 
}