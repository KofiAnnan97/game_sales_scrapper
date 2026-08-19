use std::collections::HashMap;
use iced::widget::image::Handle;

use constants::operations::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};
use alerting::email;
use file_ops::{thresholds};
use crate::utils::file_utils::load_image_from_url;
use stores::pc::{gog, microsoft_store, steam};
use types::internal::{data::SaleInfo, store::GameStore};
use constants::stores::gog::VERSION as GOG_VERSION;

#[derive(Debug, Clone)]
pub struct StoreSale {
    pub store: GameStore,
    pub info: SaleInfo,
    pub alias: String,
    pub game_id: String,
    pub icon_handler: Option<Handle>,
}

#[derive(Debug, Clone)]
pub struct SaleInfoCompare {
    pub icon_handler: Option<Handle>,
    pub title: String, 
    pub steam_price: Option<f64>,
    pub gog_price: Option<f64>,
    pub microsoft_store_price: Option<f64>,
    pub lowest_price_stores: Vec<String>
}

#[derive(Debug, Clone)]
pub struct SalesCache {
    pub store_sales: Vec<StoreSale>,
    pub comparisons: Vec<SaleInfoCompare>,
    pub by_store: HashMap<GameStore, Vec<usize>>
}

impl SalesCache {
    pub fn new() -> Self {
        SalesCache { 
            store_sales: Vec::new(),
            comparisons: Vec::new(),
            by_store: HashMap::new()
        }
    }

    pub fn clear(&mut self){
        self.store_sales.clear();
        self.comparisons.clear();
        self.by_store.clear();
    }
}

pub fn get_simple_prices_str(store_name: &str, sales: Vec<SaleInfo>) -> String{
    let mut prices_str = String::new();
    sales.iter().for_each(| game | {
        prices_str.push_str(&format!("\n\t- {} : {} -> {} ({}% off)",
                                     game.title, game.original_price, game.current_price,
                                     game.discount_percentage));
    });
    if !prices_str.is_empty() {
        let header_str = format!("\n{} game(s) that met your desired price:", store_name);
        prices_str = header_str + &prices_str;
    }
    prices_str
}

pub fn check_prices_for_display(store_sales: &Vec<StoreSale>) -> HashMap<GameStore, Vec<usize>> {
    let mut by_store: HashMap<GameStore, Vec<usize>> = HashMap::new();

    for i in 0..store_sales.len() {
        let sale: &StoreSale = store_sales.get(i).unwrap();
        by_store.entry(sale.store).or_insert_with(Vec::new).push(i);
    }
    by_store
}

pub async fn check_prices(use_html: bool) -> Result<String, String> {
    let thresholds = thresholds::load_thresholds().unwrap_or_else(|_e|Vec::new());
    let mut steam_sales: Vec<SaleInfo> = Vec::new();
    let mut gog_sales: Vec<SaleInfo> = Vec::new();
    let mut microsoft_store_sales: Vec<SaleInfo> = Vec::new();
    let http_client = reqwest::Client::new();
    let mut output = String::new();
    for elem in thresholds.iter(){
        if elem.steam_id != 0 {
            match steam::get_price_details(elem.steam_id, &http_client).await {
                Ok(info) => {
                    if elem.desired_price >= info.current_price {
                        steam_sales.push(info);
                    }
                },
                Err(e) => println!("{}", e)
            }
        }
        if elem.gog_id != 0 {
            if GOG_VERSION == 1{
                match gog::get_price_details(&elem.title).await {
                    Some(po) => {
                        let current_price = po.final_amount.parse::<f64>().unwrap();
                        if elem.desired_price >= current_price {
                            let price_str = format!("\n\t- {} : {} -> {} {} ({}% off)",
                                                    elem.title, po.base_amount, po.final_amount,
                                                    po.currency, po.discount_percentage);
                            output.push_str(&price_str);
                        }
                    },
                    None => ()
                }
            }
            else if GOG_VERSION == 2{
                match gog::get_price_details_v2(&elem.title, &http_client).await {
                    Some(info) => {
                        if elem.desired_price >= info.current_price {
                            gog_sales.push(info);
                        }
                    },
                    None => ()
                }
            }
        }
        if !elem.microsoft_store_id.is_empty() {
            match microsoft_store::get_price_details(&elem.microsoft_store_id, &http_client).await {
                Some(info) => {
                    if elem.desired_price >= info.current_price {
                        microsoft_store_sales.push(info);
                    }
                },
                None => ()
            }
        }
    }
    if !steam_sales.is_empty(){
        let store_name =  GameStore::STEAM.get_name();
        if use_html { output.push_str(&email::create_store_cards(store_name, steam_sales)); }
        else { output.push_str(&get_simple_prices_str(&store_name, steam_sales)); }
    }
    if !gog_sales.is_empty(){
        let store_name = GameStore::GOOD_OLD_GAMES.get_name();
        if use_html { output.push_str(&email::create_store_cards(store_name, gog_sales)); }
        else { output.push_str(&get_simple_prices_str(&store_name, gog_sales)); }
    }
    if !microsoft_store_sales.is_empty(){
        let store_name = GameStore::MICROSOFT_STORE_PC.get_name();
        if use_html { output.push_str(&email::create_store_cards(&store_name, microsoft_store_sales)); }
        else{ output.push_str(&get_simple_prices_str(&store_name, microsoft_store_sales)); }
    }

    if output.is_empty() {
        Ok(String::from("No thresholded games are currently below desired price."))
    } else {
        Ok(output)
    }
}

pub fn compare_prices(store_sales: &Vec<StoreSale>) -> Vec<SaleInfoCompare> {
    let mut compared_sales: HashMap<String, SaleInfoCompare> = HashMap::new();
    for sale in store_sales.iter() {
        let cmp_id = if !sale.alias.is_empty() { sale.alias.clone() } else { sale.info.title.clone() };
        let entry = compared_sales.entry(cmp_id)
            .or_insert_with(|| SaleInfoCompare { 
                icon_handler: sale.icon_handler.clone(),
                title: sale.info.title.clone(), 
                steam_price: None, 
                gog_price: None, 
                microsoft_store_price: None,
                lowest_price_stores: Vec::new()
            });

        let price = sale.info.current_price.clone();
        match sale.store {
            GameStore::STEAM => entry.steam_price = Some(price),
            GameStore::GOOD_OLD_GAMES => entry.gog_price = Some(price),
            GameStore::MICROSOFT_STORE_PC => entry.microsoft_store_price = Some(price),
        }
    }  
    let mut compiled_sales: Vec<SaleInfoCompare> = compared_sales.into_values().collect();
    for sale in compiled_sales.iter_mut() {
        let lowest_price = sale.steam_price.unwrap_or(f64::MAX)
            .min(sale.gog_price.unwrap_or(f64::MAX))
            .min(sale.microsoft_store_price.unwrap_or(f64::MAX));
        if sale.steam_price == Some(lowest_price) {
            sale.lowest_price_stores.push(STEAM_STORE_ID.into());
        }
        if sale.gog_price == Some(lowest_price) {
            sale.lowest_price_stores.push(GOG_STORE_ID.into());
        }
        if sale.microsoft_store_price == Some(lowest_price) {
            sale.lowest_price_stores.push(MICROSOFT_STORE_ID.into());
        }
    }
    compiled_sales
}

pub async fn get_sales() -> Result<Vec<StoreSale>, String> {
    let mut sales: Vec<StoreSale> = Vec::new();
    let thresholds = thresholds::load_thresholds().unwrap_or_else(|_e|Vec::new());
    let http_client = reqwest::Client::new();
    for game in thresholds.iter() {
        if game.steam_id != 0 {
            match steam::get_price_details(game.steam_id, &http_client).await {
                Ok(info) => {
                    if game.desired_price >= info.current_price {
                        let icon_handler= match load_image_from_url(&info.icon_link).await {
                            Ok(handler) => {
                                Some(handler)
                            },
                            Err(_) => None
                        };
                        sales.push(StoreSale{ 
                            store: GameStore::STEAM, 
                            info, 
                            alias: game.alias.clone(), 
                            game_id: format!("{}_{}", STEAM_STORE_ID, &game.steam_id), 
                            icon_handler 
                        })
                    }
                },
                Err(e) => return Err(format!("{}", e.to_string()))
            }
        }
        if game.gog_id != 0 {
            match gog::get_price_details_v2(&game.title, &http_client).await {
                Some(info) => {
                    if game.desired_price >= info.current_price {
                        let icon_handler= match load_image_from_url(&info.icon_link).await {
                            Ok(handler) => {
                                Some(handler)
                            },
                            Err(_) => None
                        };
                        sales.push(StoreSale{ 
                            store: GameStore::GOOD_OLD_GAMES, 
                            info, 
                            alias: game.alias.clone(), 
                            game_id: format!("{}_{}", GOG_STORE_ID, &game.gog_id), 
                            icon_handler 
                        })
                    }
                },
                None => () //Err(String::from("Could not find"))
            }
        }
        if !game.microsoft_store_id.is_empty() {
            match microsoft_store::get_price_details(&game.microsoft_store_id, &http_client).await {
                Some(info) => {
                    if game.desired_price >= info.current_price {
                        let icon_handler= match load_image_from_url(&info.icon_link).await {
                            Ok(handler) => {
                                Some(handler)
                            },
                            Err(_) => None
                        };
                        sales.push(StoreSale{ 
                            store: GameStore::MICROSOFT_STORE_PC, 
                            info, 
                            alias: game.alias.clone(), 
                            game_id: format!("{}_{}", MICROSOFT_STORE_ID, &game.microsoft_store_id), 
                            icon_handler 
                        })
                    }
                },
                None => ()
            }
        }
    }
    Ok(sales)
}