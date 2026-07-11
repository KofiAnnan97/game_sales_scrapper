use std::collections::HashMap;

use reqwest::Client;

use constants::operations::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};
use alerting::email;
use file_ops::{settings, thresholds};
use stores::pc::{gog, microsoft_store, steam};
use structs::internal::data::SaleInfo;
use constants::stores::gog::VERSION as GOG_VERSION;

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

pub async fn check_prices_for_display() -> Result<HashMap<String, Vec<SaleInfo>>, String> {
    let thresholds = thresholds::load_thresholds().unwrap_or_else(|_e|Vec::new());
    let http_client = Client::new();
    let mut sales_info_by_store: HashMap<String, Vec<SaleInfo>> = HashMap::new();
    for store in settings::get_available_stores() {
        sales_info_by_store.insert(store.clone(), Vec::new());
    }
    for elem in thresholds.iter(){
        if elem.steam_id != 0 {
            match steam::get_price_details(elem.steam_id, &http_client).await {
                Ok(info) => {
                    let current_price = info.current_price.parse::<f64>().unwrap();
                    if elem.desired_price >= current_price {
                        sales_info_by_store.get_mut(STEAM_STORE_ID).unwrap().push(info);
                    }
                },
                Err(e) => println!("{}", e)
            }
        }
        if elem.gog_id != 0 {
            if GOG_VERSION == 2{
                match gog::get_price_details_v2(&elem.title, &http_client).await {
                    Some(info) => {
                        let current_price = info.current_price.parse::<f64>().unwrap();
                        if elem.desired_price >= current_price {
                            sales_info_by_store.get_mut(GOG_STORE_ID).unwrap().push(info);
                        }
                    },
                    None => ()
                }
            }
        }
        if !elem.microsoft_store_id.is_empty() {
            match microsoft_store::get_price_details(&elem.microsoft_store_id, &http_client).await {
                Some(info) => {
                    let current_price = info.current_price.parse::<f64>().unwrap();
                    if elem.desired_price >= current_price {
                        sales_info_by_store.get_mut(MICROSOFT_STORE_ID).unwrap().push(info);
                    }
                },
                None => ()
            }
        }
    }
    if sales_info_by_store.get_mut(STEAM_STORE_ID).unwrap().len() > 0 ||
        sales_info_by_store.get_mut(GOG_STORE_ID).unwrap().len() > 0 ||
        sales_info_by_store.get_mut(MICROSOFT_STORE_ID).unwrap().len() > 0 {
        return Ok(sales_info_by_store);
    }

    Err(String::from("Failed to get prices"))
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
                    let current_price = info.current_price.parse::<f64>().unwrap();
                    if elem.desired_price >= current_price {
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
                        let current_price = info.current_price.parse::<f64>().unwrap();
                        if elem.desired_price >= current_price {
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
                    let current_price = info.current_price.parse::<f64>().unwrap();
                    if elem.desired_price >= current_price {
                        microsoft_store_sales.push(info);
                    }
                },
                None => ()
            }
        }
    }
    if !steam_sales.is_empty(){
        let store_name = settings::get_proper_store_name(STEAM_STORE_ID).unwrap();
        if use_html { output.push_str(&email::create_storefront_table_html(&store_name, steam_sales)); }
        else { output.push_str(&get_simple_prices_str(&store_name, steam_sales)); }
    }
    if !gog_sales.is_empty(){
        let store_name = settings::get_proper_store_name(GOG_STORE_ID).unwrap();
        if use_html { output.push_str(&email::create_storefront_table_html(&store_name, gog_sales)); }
        else { output.push_str(&get_simple_prices_str(&store_name, gog_sales)); }
    }
    if !microsoft_store_sales.is_empty(){
        let store_name = settings::get_proper_store_name(MICROSOFT_STORE_ID).unwrap();
        if use_html { output.push_str(&email::create_storefront_table_html(&store_name, microsoft_store_sales)); }
        else{ output.push_str(&get_simple_prices_str(&store_name, microsoft_store_sales)); }
    }

    if output.is_empty() {
        Ok(String::from("No thresholded games are currently below desired price."))
    } else {
        Ok(output)
    }
}