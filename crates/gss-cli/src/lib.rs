use std::io::{self, Write};

use constants::stores::gog::VERSION as GOG_VERSION;
use stores::pc::{steam, gog, microsoft_store};
use alerting::email;
use file_ops::{settings, thresholds};
use structs::internal::{data::SaleInfo, enums::GameStore};
use structs::response::gog::{GameInfo as GOGGameInfo};
use structs::response::microsoft_store::ProductInfo;

pub fn storefront_check() -> Vec<GameStore> {
    let selected_stores = settings::get_selected_stores();
    if selected_stores.len() == 0 {
        panic!("Please configure which stores to query. Run \'game_sales_scrapper config --help\' for more info.");
    }
    selected_stores
}

pub fn get_simple_prices_str(store_name: &str, sales: Vec<SaleInfo>) -> String{
    let mut prices_str = String::new();
    for game in sales.iter(){
        prices_str.push_str(&format!("\n\t- {} : {} -> {} ({}% off)",
                                     game.title, game.original_price, game.current_price,
                                     game.discount_percentage));
    }
    if !prices_str.is_empty() {
        let header_str = format!("\n{} game(s) that met your desired price:", store_name);
        prices_str = header_str + &prices_str;
    }
    prices_str
}

pub async fn check_prices(use_html: bool) -> String {
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
        let store_name = GameStore::STEAM.get_name();
        if use_html { output.push_str(&email::create_store_cards(&store_name, steam_sales)); }
        else { output.push_str(&get_simple_prices_str(store_name, steam_sales)); }
    }
    if !gog_sales.is_empty(){
        let store_name = GameStore::GOOD_OLD_GAMES.get_name();
        if use_html { output.push_str(&email::create_store_cards(&store_name, gog_sales)); }
        else { output.push_str(&get_simple_prices_str(store_name, gog_sales)); }
    }
    if !microsoft_store_sales.is_empty(){
        let store_name = GameStore::MICROSOFT_STORE_PC.get_name();
        if use_html { output.push_str(&email::create_store_cards(&store_name, microsoft_store_sales)); }
        else{ output.push_str(&get_simple_prices_str(store_name, microsoft_store_sales)); }
    }
    output
}

pub async fn steam_insert_sequence(alias: &str, title: &str, price: f64, client: &reqwest::Client) {
    match steam::check_game(title).await {
        Some(data) => thresholds::add_steam_game(alias.to_string(), data, price, &client).await,
        None => {
            match steam::search_game(title).await {
                Some(t) => {
                    match steam::check_game(&t).await {
                        Some(data) => thresholds::add_steam_game(alias.to_string(), data, price, &client).await,
                        None => eprintln!("Something went wrong")
                    }
                }
                None => ()
            }
        }
    }
}

pub async fn gog_insert_sequence(alias: &str, title: &str, price: f64, client: &reqwest::Client){
    let mut search_list : Vec<GOGGameInfo> = Vec::new();
    match gog::search_game_by_title_v2(title, &client).await {
        Ok(data) => search_list = data,
        Err(e) => println!("Search GOG Game Error: {}", e)
    }
    if !search_list.is_empty() {
        println!("GOG search results:");
        for (i, game) in search_list.iter().enumerate(){
            let price = match &game.price{
                Some(po) => po.base_money.amount.clone(),
                None => String::from("0"),
            };
            println!("  [{}] {} - ${}", i, game.title, price);
        }
        println!("  [q] SKIP");
        let mut input = String::new();
        print!("Type integer corresponding to game title or type \'q\' to skip: ");
        let _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        if input.trim() == "q" {
            eprintln!("Request terminated.");
        }
        else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < search_list.len(){
                        //title = &search_list[idx].title;
                        let game = &search_list[idx];
                        thresholds::add_gog_game(alias.to_string(), game, price);
                    }
                    else if idx >= search_list.len(){
                        eprintln!("Integer \"{}\" is invalid. Request terminated.", idx);
                    }
                },
                Err(e) => println!("Invalid input: {}\nError: {}", input, e)
            }
        }
    }
    else{
        println!("Could not find a game title matching \"{}\" on GOG.", title);
    }
}

pub async fn microsoft_store_insert_sequence(alias: &str, title: &str, price: f64, client: &reqwest::Client){
    let mut search_list : Vec<ProductInfo> = Vec::new();
    match microsoft_store::search_game_by_title(title, &client).await {
        Ok(data) => search_list = data,
        Err(e) => println!("Search Microsoft Store Error: {}", e)
    }
    if !search_list.is_empty() {
        println!("Microsoft Store search results:");
        for(i, game) in search_list.iter().enumerate(){
            println!("  [{}] {} - ${}", i, game.title, game.price_info.msrp.unwrap_or_default());
        }
        println!("  [q] SKIP");
        let mut input = String::new();
        print!("Type integer corresponding to game title or type \"q\" to quit: ");
        let _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        if input.trim() == "q" { eprintln!("Request terminated."); }
        else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < search_list.len(){
                        let game = &search_list[idx];
                        thresholds::add_microsoft_store_game(alias.to_string(), game, price);
                    }
                    else if idx >= search_list.len(){
                        eprintln!("Integer \"{}\" is invalid. Request terminated.", idx);
                    }
                },
                Err(e) => println!("Invalid input: {}\nError: {}", input, e)
            }
        }
    }
    else {
        println!("Could not find a game title matching \"{}\" on the Microsoft Store.", title);
    }
}