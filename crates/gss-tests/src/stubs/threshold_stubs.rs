use std::collections::HashMap;

use file_ops::thresholds;
use structs::{
    internal::data::GameThreshold, 
    response::{
        gog::{BaseMoney, FinalMoney, GameInfo as GOGGame, GameInfoBuilder as GOGGameBuilder, Price}, 
        microsoft_store::{PriceInfo, ProductInfo as MSGame, ProductInfoBuilder as MSGameBuilder}, 
        steam::App
    }
};

pub fn add_simple_threshold(game_title: &str, game_alias: &str, price: f64) {
    let mut alias_map: HashMap<String, Vec<String>> = thresholds::load_alias_map().unwrap_or_default();
    let mut thresholds = thresholds::load_thresholds().unwrap_or_default();
    let mut unique_title = true;
    for threshold in &thresholds{
        if threshold.title == game_title {
            unique_title = false;
            break;
        }
    }
    if unique_title {
        if alias_map.contains_key(game_alias) {
            let idx = alias_map.get(game_alias).unwrap().iter().position(|title| title == game_title);
            if idx.is_none() {
                alias_map.get_mut(game_alias).unwrap().push(game_title.to_string());
            }
        } else { 
            alias_map.insert(game_alias.to_string(), vec![game_title.to_string()]);
        }  
        thresholds.push(GameThreshold {
            title: game_title.to_string(),
            alias: game_alias.to_string(),
            steam_id: 123,
            gog_id: 456,
            microsoft_store_id: String::from("abc"),
            currency: String::from("USD"),
            desired_price: price
        });
    }
    thresholds::update_alias_map(alias_map);
    thresholds::update_thresholds(thresholds);
}

pub fn test_steam_app() -> App{
    App{
        app_id: 220,
        name: "Half-Life 2".to_string(),
        last_modified: 678910,
        price_change_number: 1112131415,
    }
}

pub fn test_gog_game() -> GOGGame {
    let id_str = String::from("123");
    let title = String::from("Random GOG Game");
    let price = Price {
        final_price: String::new(),
        base_price: String::new(),
        discount: None,
        final_money: FinalMoney {
            amount: String::new(),
            currency: "USD".to_string(),
            discount: String::new(),
        },
        base_money: BaseMoney {
            amount: String::new(),
            currency: String::new(),
        }
    };
    let icon_link = String::new();
    let store_page_link = String::new();
    GOGGameBuilder::new(id_str, title, price, icon_link, store_page_link)
}

pub fn test_ms_game() -> MSGame {
    let id_str = String::from("abc");
    let title = String::from("Random Microsoft Game");
    let price = PriceInfo {
        msrp: None,
        price: None,
        badge_text: None,
        force_to_display_price: false,
        narrator_text: "".to_string(),
        ownership: 0,
    };
    let icon_link = String::new();
    let store_page_link = String::new();
    MSGameBuilder::new(id_str, title, price, icon_link, store_page_link)
}