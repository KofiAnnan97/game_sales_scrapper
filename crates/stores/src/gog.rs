use serde_json::{Result, Value, Error};
use std::f64;

use structs::data::{SaleInfo};
use structs::gog::{Game, PriceOverview, GameInfo};

pub static VERSION: u32 = 2;

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

pub async fn get_price_details(title: &str) -> Option<PriceOverview> {
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

pub async fn search_game_by_title_v2(title: &str, http_client: &reqwest::Client) -> Result<Vec<GameInfo>>{
    let mut like_title = String::from("like:");
    like_title.push_str(title);
    let query_string = [
        ("query", like_title.as_str()),
        ("limit", "48"),
        ("order", "desc:score"),
        ("productType", "in:game"),
        ("page", "1"),
        ("countryCode", "US"),
        ("locale", "en-US"),
        ("currencyCode", "USD"),
    ];
    let url = format!("{}{}", BASE_URL, CATALOG_ENDPOINT);
    let resp = http_client.get(url)
        .query(&query_string)
        .send()
        .await
        .expect("Failed to get response")
        .text()
        .await
        .expect("Failed to get data");
    let body : Value = serde_json::from_str(&resp).expect("Could not convert  search to JSON");
    //println!("{:?}", body);
    let products = serde_json::to_string(&body["products"]).unwrap();
    let games_list : Vec<GameInfo> = serde_json::from_str::<Vec<GameInfo>>(&products)?;
    Ok(games_list)
}

pub async fn get_price_details_v2(title: &str, http_client: &reqwest::Client) -> Option<SaleInfo> {
    let mut like_title = String::from("like:");
    like_title.push_str(title);
    let query_string = [
        ("query", like_title.as_str()),
        ("limit", "1"),
        ("order", "desc:score"),
        ("productType", "in:game"),
        ("page", "1"),
        ("countryCode", "US"),
        ("locale", "en-US"),
        ("currencyCode", "USD"),
    ];
    let url = format!("{}{}", BASE_URL, CATALOG_ENDPOINT);
    let resp = http_client.get(url)
        .query(&query_string)
        .send()
        .await
        .expect("Failed to get response")
        .text()
        .await
        .expect("Failed to get data");
    let body: Value = serde_json::from_str(&resp).expect("Could not convert to JSON");
    if let Some(products) = body["products"].as_array() {
        let first_product = serde_json::to_string(&products[0]).unwrap();
        let data = serde_json::from_str::<GameInfo>(&first_product).unwrap();
        match data.price {
            Some(po) => {
                return Ok::<SaleInfo, Error>(SaleInfo{
                    title: data.title,
                    original_price: po.base_money.amount,
                    current_price: po.final_money.amount, 
                    discount_percentage: po.final_money.discount[0..po.final_money.discount.len()-3].to_string(),
                    icon_link: data.c_horizontal,
                    store_page_link: data.store_link,
                }).ok();
            },
            None => (),
        }
    }
    None 
}