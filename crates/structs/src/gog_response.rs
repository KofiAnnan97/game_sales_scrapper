use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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

/*-----------*
 | VERSION 2 |
 *-----------*/

#[derive(Deserialize, Serialize, Debug)]
pub struct GameInfo {
    pub id: String,
    pub title: String,
    pub price: Option<Price>,
    #[serde(rename="coverHorizontal")]
    pub c_horizontal: String,
    #[serde(rename="storeLink")]
    pub store_link: String,
    #[serde(rename="coverVertical")]
    c_vertical: String,
    developers: Vec<String>,
    editions: Vec<Editions>,
    features: Vec<HashMap<String, String>>,
    genres: Vec<HashMap<String, String>>,
    #[serde(rename="operatingSystems")]
    os: Vec<String>,
    #[serde(rename="productState")]
    product_state: String,
    #[serde(rename="productType")]
    product_type: String,
    publishers: Vec<String>,
    ratings: Vec<HashMap<String, String>>,
    #[serde(rename="releaseDate")]
    release_date: String,
    #[serde(rename="reviewsRating")]
    reviews_rating: u32,
    screenshots: Vec<String>,
    slug: String,
    #[serde(rename="storeReleaseDate")]
    store_release_date: String,
    tags: Vec<HashMap<String,String>>,
    #[serde(rename="userPreferredLanguage")]
    user_pref_lang: UserPreferredLanguage,
}

pub struct GameInfoBuilder {
    pub data: GameInfo,
}

impl GameInfoBuilder{
    pub fn new(id_str: String, game_title: String,
               price_info: Price, icon_link: String,
               store_page_link: String) -> GameInfo {
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
            genres: vec![],
            os: vec![],
            product_state: "".to_string(),
            product_type: "".to_string(),
            publishers: vec![],
            ratings: vec![],
            release_date: "".to_string(),
            reviews_rating: 0,
            screenshots: vec![],
            slug: "".to_string(),
            store_release_date: "".to_string(),
            tags: vec![],
            user_pref_lang: Default::default(),
        }
    }
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Editions{
    id: u64,
    #[serde(rename="isRootEdition")]
    is_root_edition: bool,
    name: String,
}