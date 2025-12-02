use std::io::{self, Write};
use serde_json::Result;
use std::fs::read_to_string;

use crate::file_ops::{json, settings};
use crate::stores::{steam, gog, microsoft_store};
use crate::structs::steam_response::Game;
use crate::structs::gog_response::GameInfo as GOGGameInfo;
use crate::structs::microsoft_store_response::ProductInfo;
use crate::structs::data::GameThreshold;

static THRESHOLD_FILENAME : &str = "thresholds.json";

pub fn get_path() -> String {
    let mut thresh_path = json::get_data_path();
    thresh_path.push_str("/");
    thresh_path.push_str(THRESHOLD_FILENAME);
    return json::get_path(&thresh_path);
}

pub fn load_data() -> Result<Vec<GameThreshold>> {
    let filepath = get_path();
    let data = read_to_string(filepath).unwrap();
    let temp = serde_json::from_str::<Vec<GameThreshold>>(&data);
    return temp;
}

fn is_threshold(title: &str, game_thresh: &GameThreshold) -> bool {
    title == game_thresh.title || title == game_thresh.alias
}

pub async fn add_steam_game(new_alias: String, app: Game, price: f64, client: &reqwest::Client){
    let mut thresholds = load_data().unwrap_or_else(|_e|Vec::new());
    match steam::get_price(app.app_id, &client).await {
        Ok(po) => {
            let mut unique : bool = true;
            for elem in thresholds.iter() {
                if is_threshold(&app.name, elem) {
                    unique = false;
                    if elem.steam_id == 0 {
                        update_id(&elem.title, settings::STEAM_STORE_ID, app.app_id);
                    }
                    break;
                }
            }
            if unique { 
                thresholds.push(GameThreshold {
                    title: app.name.clone(),
                    alias: new_alias,
                    steam_id: app.app_id.clone(),
                    gog_id: 0,
                    microsoft_store_id: String::new(),
                    currency: po.currency[1..po.currency.len()-1].to_string(),
                    desired_price: price
                });
                let data_str = serde_json::to_string(&thresholds).unwrap();
                json::write_to_file(get_path(), data_str);
                println!("Successfully added Steam game: \"{}\".", app.name);
            }
            //else { println!("Duplicate title: \"{}\".", app.name); }
        },
        Err(e) => println!("{}", e)
    }
}

pub fn add_gog_game(new_alias: String, game: &GOGGameInfo, price: f64){
    let mut thresholds = load_data().unwrap_or_else(|_e|Vec::new());
    let mut unique : bool = true;
    for elem in thresholds.iter(){
        if is_threshold(&game.title, elem){
            unique = false;
            if elem.gog_id == 0 {
                let game_id = game.id.parse::<u64>().unwrap();
                update_id(&elem.title, settings::GOG_STORE_ID, game_id as usize);
            }
            break;
        }
    }
    if unique { 
        let currency_code = match &game.price {
            Some(price_data) => price_data.base_money.currency.clone(),
            None => "USD".to_string(),
        };
        thresholds.push(GameThreshold {
            title: game.title.clone(),
            alias: new_alias,
            steam_id: 0,
            gog_id: game.id.parse::<u64>().unwrap() as usize,
            microsoft_store_id: String::new(),
            //currency: game.price.currency.clone(), // Version 1
            currency: currency_code,
            desired_price: price
        });
        let data_str = serde_json::to_string(&thresholds).unwrap();
        json::write_to_file(get_path(), data_str);
        println!("Successfully added GOG game \"{}\".", game.title);
    }
    //else { println!("Duplicate title: \"{}\".", game.title); }
}

pub fn add_microsoft_store_game(new_alias: String, game: &ProductInfo, price: f64){
    let mut thresholds = load_data().unwrap_or_else(|_e|Vec::new());
    let mut unique : bool = true;
    for elem in thresholds.iter(){
        if is_threshold(&game.title, elem){
            unique = false;
            if elem.microsoft_store_id.is_empty() {
                let game_id = &game.product_id;
                update_id_str(&elem.title, settings::MICROSOFT_STORE_ID, game_id);
            }
            break;
        }
    }
    if unique {
        thresholds.push(GameThreshold{
            title: game.title.clone(),
            alias: new_alias,
            steam_id: 0,
            gog_id: 0,
            microsoft_store_id: game.product_id.clone(),
            currency: String::new(),
            desired_price: price
        });
        let data_str = serde_json::to_string(&thresholds).unwrap();
        json::write_to_file(get_path(), data_str);
        println!("Successfully added Microsoft Store game \"{}\".", game.title);
    }
}

pub fn set_game_alias() -> String {
    let mut alias = "".to_string();
    if settings::get_alias_state() {
        let mut input = String::new();
        print!("Do you want to assign an alias [Y\\N]? ");
        let _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to permission to assign alias.");
        if input.trim() == "Yes" || input.trim() == "Y" || input.trim() == "YES"{
            print!("Alias name: ");
            let _ = io::stdout().flush();
            let mut alias_name = String::new();
            io::stdin()
                .read_line(&mut alias_name)
                .expect("Failed to read alias.");
            alias = alias_name.trim().to_string();
        }
    }
    alias
}

pub fn update_alias(title: &str, new_alias: &str){
    let mut thresholds = load_data().unwrap_or_else(|_e|Vec::new());
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none() {
        let i = idx.unwrap();
        thresholds[i].alias = new_alias.to_string();
        let data_str = serde_json::to_string(&thresholds).expect("Could not convert GOG id update to string.");
        json::write_to_file(get_path(), data_str);
    }
    else {
        println!("Could not find threshold with title : \"{}\"", title);
    }
}

pub fn update_price(title: &str, price: f64) {
    let mut thresholds = load_data().unwrap_or_else(|_e|Vec::new());
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none() {
        let i = idx.unwrap();
        if price != thresholds[i].desired_price{
            let old_threshold = thresholds[i].desired_price.clone();
            thresholds[i].desired_price = price;
            let data_str = serde_json::to_string(&thresholds).expect("Could not price update to string.");
            json::write_to_file(get_path(), data_str);
            println!("\"{}\": updated price threshold from {} to {}", thresholds[i].title,
                                                       old_threshold,
                                                       thresholds[i].desired_price);
        }
        else{
            println!("Price was not updated because it is already set to {}", price);
        }
    }
    else{
        println!("\"{}\" does not have a configured threshold.", title);
    }
}

pub fn update_id(title: &str, store_type: &str, id: usize){
    let mut thresholds = load_data().unwrap_or_else(|_e|Vec::new());
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    //println!("{:?}", idx);
    if !idx.is_none() {
        let mut updated_id : bool = false;
        let i = idx.unwrap();
        let mut store_name = String::new();
        match store_type{
            settings::STEAM_STORE_ID => {
                store_name = settings::get_proper_store_name(settings::STEAM_STORE_ID).unwrap();
                thresholds[i].steam_id = id;
                updated_id = true;
            },
            settings::GOG_STORE_ID => {
                store_name = settings::get_proper_store_name(settings::GOG_STORE_ID).unwrap();
                thresholds[i].gog_id = id;
                updated_id = true;
            },
            _ => eprintln!("Unknown store type: {}", store_type),
        }
        if updated_id {
            let update_err = format!("Could not convert the {} id update to a string object.", store_type);
            let data_str = serde_json::to_string(&thresholds).expect(&update_err);
            json::write_to_file(get_path(), data_str);
            println!("Updated {} ID for \"{}\"", store_name, title);
        }
    }
}

pub fn update_id_str(title: &str, store_type: &str, id: &str){
    let mut thresholds = load_data().unwrap_or_else(|_e|Vec::new());
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none() {
        let mut updated_id : bool = false;
        let i = idx.unwrap();
        let mut store_name = String::new();
        match store_type {
            settings::MICROSOFT_STORE_ID => {
                store_name = settings::get_proper_store_name(settings::MICROSOFT_STORE_ID).unwrap();
                thresholds[i].microsoft_store_id = id.to_string();
                updated_id = true;
            }
            _ => eprintln!("Unknown store type: {}", store_type),
        }
        if updated_id {
            let update_err = format!("Could not convert the {} id update to a string object.", store_type);
            let data_str = serde_json::to_string(&thresholds).expect(&update_err);
            json::write_to_file(get_path(), data_str);
            println!("Updated {} ID for \"{}\"", store_name, title);
        }  
    }
}

pub fn remove(title: &str){
    let mut thresholds = load_data().unwrap_or_else(|_e|Vec::new());
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none(){
        thresholds.remove(idx.unwrap());
        let data_str = serde_json::to_string(&thresholds).unwrap();
        json::write_to_file(get_path(), data_str);
        println!("Successfully removed \"{}\".", title);
    }
    else { println!("Failed to remove: \"{}\".", title); }
}

pub fn list_games() {
    match load_data(){
        Ok(data) => {
            println!("Price Thresholds");
            for threshold in data.iter() {
                println!("  - {} => {} ({})", threshold.title, 
                                              threshold.desired_price, 
                                              threshold.currency);
            }
        },
        Err(e) => println!("Error: {}", e)
    }
}