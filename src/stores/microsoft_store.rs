use serde_json::{Result, Value, Error};

use crate::structs::data::SaleInfo;
use crate::structs::microsoft_store_response::GameInfo;

static BASE_URL : &str = "https://apps.microsoft.com";
static SEARCH_ENDPOINT : &str = "/api/products/search";

pub async fn search_game_by_title(title: &str, http_client: &reqwest::Client) -> Result<Vec<GameInfo>> {
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
    let resp = http_client.get(url)
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
    let game_list = serde_json::from_str::<Vec<GameInfo>>(&products)?;
    Ok(game_list)
}

pub async fn get_price(title: &str, xbox_id :&str, http_client: &reqwest::Client) -> Option<SaleInfo> {
    let search_list = search_game_by_title(title, http_client)
        .await
        .unwrap_or_else(|_e|Vec::new());
    for game in search_list {
        if game.product_id == xbox_id {
            let mut discount_str = game.price_info.badge_text.unwrap_or_default();
            discount_str = if !discount_str.is_empty() {
                discount_str[1..discount_str.len()-1].to_string()
            }else{
                String::from("0")
            };
            return Ok::<SaleInfo, Error>(SaleInfo{
                icon_link: game.large_icon_url.clone(),
                title: game.title.clone(),
                original_price: format!("{}", game.price_info.msrp.unwrap_or_default()),
                current_price: format!("{}", game.price_info.price.unwrap_or_default()),
                discount_percentage: discount_str,
                store_page_link: game.redirect_url.unwrap_or_default(),
            }).ok();
        }
    }
    None
}