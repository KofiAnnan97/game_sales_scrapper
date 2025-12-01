use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value, Error};

use crate::file_ops::structs::{SaleInfo};

static BASE_URL : &str = "https://apps.microsoft.com";
static SEARCH_ENDPOINT : &str = "/api/products/search";

#[derive(Deserialize, Serialize, Debug)]
pub struct GameInfo {
    #[serde(rename = "productId")]
    pub product_id: String,
    pub title: String,
    description: String,
    categories: Vec<String>,
    #[serde(rename = "publisherName")]
    publisher_name: Option<String>,
    images: Vec<Images>,
    #[serde(rename = "averageRating")]
    average_rating: f64,
    price: f64,
    #[serde(rename = "displayPrice")]
    display_price: String,
    #[serde(rename = "productFamilyName")]
    product_family_name: String,
    #[serde(rename = "packageFamilyNames")]
    package_family_names: Vec<String>,
    #[serde(rename = "isGamingAppOnly")]
    is_gaming_app_only: bool,
    installer: HashMap<String,String>,
    #[serde(rename = "skusSummary")]
    pub skus_summary: Vec<Option<SkusSummary>>,
    #[serde(rename = "releaseDateUtc")]
    release_date_utc: Option<String>,
    previews: Vec<Preview>,
    #[serde(rename = "priceInfo")]
    pub price_info: PriceInfo,
    #[serde(rename = "typeTag")]
    type_tag: String,
    #[serde(rename = "ratingCountFormatted")]
    rating_count_formatted: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: String,
    #[serde(rename = "posterArtUrl")]
    poster_art_url: String,
    #[serde(rename = "boxArtUrl")]
    pub large_icon_url: String,
    #[serde(rename = "iconUrlBackground")]
    icon_url_background: String,
    screenshots: Vec<String>,
    #[serde(rename = "encodedTitle")]
    encoded_title: String,
    #[serde(rename = "isApplication")]
    is_application: bool,
    #[serde(rename = "isGame")]
    is_game: bool,
    #[serde(rename = "isTvSeries")]
    is_tv_series: bool,
    #[serde(rename = "isMovie")]
    is_movie: bool,
    #[serde(rename = "isMoviesOrTVs")]
    is_movies_or_tvs: bool,
    #[serde(rename = "isPwa")]
    is_pwa: bool,
    #[serde(rename = "isCoreGame")]
    is_core_game: bool,
    #[serde(rename = "isAllowed")]
    is_allowed: bool,
    #[serde(rename = "isBrowsable")]
    is_browsable: bool,
    #[serde(rename = "isPurchaseEnabled")]
    is_purchase_enabled: bool,
    #[serde(rename = "isAd")]
    is_ad: bool,
    #[serde(rename = "isSparkProduct")]
    is_spark_product: bool,
    #[serde(rename = "isAndroid")]
    is_android: bool,
    #[serde(rename = "redirectUrl")]
    pub redirect_url: Option<String>,
    #[serde(rename = "isHardware")]
    is_hardware: bool,
    #[serde(rename = "isSubscription")]
    is_subscription: bool,
    #[serde(rename = "isTencent")]
    is_tencent: bool,
    #[serde(rename = "isTencentMini")]
    is_tencent_mini: bool,
    #[serde(rename = "pageTitleLocalization")]
    page_title_localization: String,
    #[serde(rename = "disableDownload")]
    disable_download: bool,
    #[serde(rename = "cardActions")]
    card_actions: Vec<String>,
    #[serde(rename = "productRatings")]
    product_ratings: Vec<ProductRating>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Images {
    #[serde(rename = "imageType")]
    image_type: Option<String>,
    #[serde(rename = "backgroundColor")]
    background_color: Option<String>,
    #[serde(rename = "foregroundColor")]
    foreground_color: Option<String>,
    caption: Option<String>,
    #[serde(rename = "imagePositionInfo")]
    image_position_info: Option<String>,
    url: String,
    height: i32,
    width: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SkusSummary {
    msrp: Option<f64>,
    #[serde(rename = "displayMSRP")]
    display_msrp: Option<String>,
    #[serde(rename = "salePrices")]
    sale_prices: Option<Vec<SalePrice>>,
    included_with: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SalePrice {
    conditions: Option<Conditions>,
    price: f64,
    #[serde(rename = "displayPrice")]
    display_price: String,
    #[serde(rename = "badgeId")]
    badge_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Conditions {
    #[serde(rename = "type")]
    condition_type: String,
    id: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Preview {
    #[serde(rename = "$type")]
    preview_type: String,
    #[serde(rename = "imageType")]
    image_type: Option<String>,
    #[serde(rename = "backgroundColor")]
    background_color: Option<String>,
    #[serde(rename = "foregroundColor")]
    foreground_color: Option<String>,
    caption: Option<String>,
    #[serde(rename = "imagePositionInfo")]
    image_position_info: Option<String>,
    url: String,
    height: i32,
    width: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PriceInfo {
    msrp: Option<f64>,
    price: Option<f64>,
    #[serde(rename = "badgeText")]
    badge_text: Option<String>,
    #[serde(rename = "forceToDisplayPrice")]
    force_to_display_price: bool,
    #[serde(rename = "narratorText")]
    narrator_text: String,
    ownership: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProductRating {
    #[serde(rename = "ratingSystem")]
    rating_system: String,
    #[serde(rename = "ratingSystemShortName")]
    rating_system_short_name: String,
    #[serde(rename = "ratingSystemId")]
    rating_system_id: String,
    #[serde(rename = "ratingSystemUrl")]
    rating_system_url: String,
    #[serde(rename = "ratingValue")]
    rating_value: String,
    #[serde(rename = "ratingId")]
    rating_id: String,
    #[serde(rename = "ratingValueLogoUrl")]
    rating_value_logo_url: String,
    #[serde(rename = "ratingAge")]
    rating_age: i32,
    #[serde(rename = "restrictMetadata")]
    restrict_metadata: bool,
    #[serde(rename = "restrictPurchase")]
    restrict_purchase: bool,
    #[serde(rename = "ratingDescriptors")]
    rating_descriptors: Option<Vec<String>>,
    #[serde(rename = "ratingDescriptorLogoUrls")]
    rating_descriptor_logo_urls: Option<Vec<String>>,
    #[serde(rename = "ratingDisclaimers")]
    rating_disclaimers: Vec<String>,
    #[serde(rename = "interactiveElements")]
    interactive_elements: Vec<String>,
    #[serde(rename = "longName")]
    long_name: String,
    #[serde(rename = "shortName")]
    short_name: String,
    description: String,
    #[serde(rename = "hasInAppPurchases")]
    has_in_app_purchases: bool,
}

pub async fn search_game_by_title(title: &str, http_client: &reqwest::Client) -> Result<Vec<GameInfo>> {
    let country_code = "US";
    let locale = "en-US";
    let media_type = "games";
    let querystring = [
        ("query", title),
        ("mediaType", media_type),
        ("age", "all"),
        ("price", "all"),
        ("category", "all"),
        ("subscription", "none"),
        ("gl", country_code),
        ("hl", locale),
    ];
    let url = format!("{}{}", BASE_URL, SEARCH_ENDPOINT);
    let resp = http_client.get(url)
        .query(&querystring)
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