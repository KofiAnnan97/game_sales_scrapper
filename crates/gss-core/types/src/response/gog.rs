use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/*-----------*
| VERSION 1 |
*-----------*/

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
    #[serde(rename = "baseAmount")]
    pub base_amount: String,
    #[serde(rename = "finalAmount")]
    pub final_amount: String,
    #[serde(rename = "isDiscounted")]
    pub is_discounted: bool,
    #[serde(rename = "discountPercentage")]
    pub discount_percentage: i32,
    #[serde(rename = "discountDifference")]
    pub discount_diff: String,
    pub symbol: String,
    #[serde(rename = "isFree")]
    pub is_free: bool,
    pub discount: i32,
    #[serde(rename = "isBonusStoreCreditIncluded")]
    pub is_bonus_credit_included: bool,
    #[serde(rename = "bonusStoreCreditAmount")]
    pub bonus_credit_amount: String,
}

/*-----------*
| VERSION 2 |
*-----------*/

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GameInfo {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub price: Option<Price>,
    #[serde(rename = "coverHorizontal")]
    pub c_horizontal: String,
    #[serde(rename = "storeLink")]
    pub store_link: String,
    #[serde(rename = "coverVertical")]
    c_vertical: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    developers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    editions: Vec<Editions>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    features: Vec<HashMap<String, String>>,
    #[serde(rename = "galaxyBackgroundImage")]
    galaxy_background_image: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    genres: Vec<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    logo: Option<String>,
    #[serde(
        rename = "operatingSystems",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    os: Vec<String>,
    #[serde(rename = "productState")]
    product_state: String,
    #[serde(rename = "productType")]
    product_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    publishers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    ratings: Vec<HashMap<String, String>>,
    #[serde(rename = "releaseDate")]
    release_date: String,
    #[serde(rename = "reviewsCount")]
    reviews_count: u32,
    #[serde(rename = "reviewsRating")]
    reviews_rating: u32,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    screenshots: Vec<String>,
    slug: String,
    #[serde(
        rename = "storeReleaseDate",
        skip_serializing_if = "Option::is_none",
        default
    )]
    store_release_date: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    tags: Vec<HashMap<String, String>>,
    #[serde(rename = "userPreferredLanguage")]
    user_pref_lang: UserPreferredLanguage,
}

pub struct GameInfoBuilder {
    pub data: GameInfo,
}

#[allow(clippy::new_ret_no_self)]
impl GameInfoBuilder {
    pub fn new(
        id_str: String,
        game_title: String,
        price_info: Price,
        icon_link: String,
        store_page_link: String,
    ) -> GameInfo {
        GameInfo {
            id: id_str,
            title: game_title,
            price: Option::from(price_info),
            c_horizontal: icon_link,
            store_link: store_page_link,
            c_vertical: "".to_string(),
            developers: vec![],
            editions: vec![],
            features: vec![],
            galaxy_background_image: None,
            genres: vec![],
            logo: None,
            os: vec![],
            product_state: "".to_string(),
            product_type: "".to_string(),
            publishers: vec![],
            ratings: vec![],
            release_date: "".to_string(),
            reviews_count: 0,
            reviews_rating: 0,
            screenshots: vec![],
            slug: "".to_string(),
            store_release_date: None,
            tags: vec![],
            user_pref_lang: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct UserPreferredLanguage {
    pub code: String,
    #[serde(rename = "inAudio")]
    pub in_audio: bool,
    #[serde(rename = "inText")]
    pub in_text: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Price {
    #[serde(rename = "final")]
    pub final_price: String,
    #[serde(rename = "base")]
    pub base_price: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub discount: Option<String>,
    #[serde(rename = "finalMoney")]
    pub final_money: FinalMoney,
    #[serde(rename = "baseMoney")]
    pub base_money: BaseMoney,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BaseMoney {
    pub amount: String,
    pub currency: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FinalMoney {
    pub amount: String,
    pub currency: String,
    pub discount: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Editions {
    id: u64,
    #[serde(rename = "isRootEdition")]
    is_root_edition: bool,
    name: String,
}
