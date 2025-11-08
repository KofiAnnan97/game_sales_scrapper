use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs::read_to_string;

use crate::file_ops::{json, settings};
use crate::stores::{steam, gog}; //, humble_bundle};

static THRESHOLD_FILENAME : &str = "thresholds.json";

#[derive(Deserialize, Serialize, Debug)]
pub struct GameThreshold{
    pub title: String,
    pub alias: String,
    pub steam_id: usize,
    pub gog_id: usize,
    //pub humble_bundle_id: String,
    pub currency: String,
    pub desired_price: f64,
}

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

fn is_threshold(title: &str, thres: &GameThreshold) -> bool {
    return title == thres.title || title == thres.alias;
}

pub async fn add_steam_game(new_alias: String, app: steam::Game, price: f64, client: &reqwest::Client){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
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
                    //humble_bundle_id: "".to_string(),
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

pub fn add_gog_game(new_alias: String, game: &gog::GameInfo, price: f64){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
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
        let mut currency_code = String::new();
        match &game.price {
            Some(price_data) => currency_code = price_data.base_money.currency.clone(),
            None => currency_code = "USD".to_string(),
        }
        thresholds.push(GameThreshold {
            title: game.title.clone(),
            alias: new_alias,
            steam_id: 0,
            gog_id: game.id.parse::<u64>().unwrap() as usize,
            //humble_bundle_id: "".to_string(),
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

/*pub fn add_humble_bundle_game(new_alias: String, game: &humble_bundle::GameInfo, price: f64){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let mut unique : bool = true;
    for elem in thresholds.iter(){
        if is_threshold(&game.human_name, elem){
            unique = false;
            if elem.humble_bundle_id.len() == 0 {
                update_id_str(&elem.title, settings::HUMBLE_BUNDLE_STORE_ID, &game.human_name);
            }
            break;
        }
    }
    if unique { 
        thresholds.push(GameThreshold {
            title: game.human_name.clone(),
            alias: new_alias,
            steam_id: 0,
            gog_id: 0,
            humble_bundle_id: game.human_url.to_string(),
            currency: game.current_price.currency.clone(),
            desired_price: price
        });
        let data_str = serde_json::to_string(&thresholds).unwrap();
        json::write_to_file(get_path(), data_str);
        println!("Successfully added Humble Bundle game \"{}\".", game.human_name);
    }
}*/

pub fn update_alias(title: &str, new_alias: &str){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => eprintln!("update_price Error: {}", e)
    };
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
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => eprintln!("Error: {}", e)
    };
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
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => eprintln!("Error: {}", e)
    };
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    //println!("{:?}", idx);
    if !idx.is_none() {
        let mut updated_id : bool = false;
        let i = idx.unwrap();
        match store_type{
            settings::STEAM_STORE_ID => {
                thresholds[i].steam_id = id;
                updated_id = true;
            },
            settings::GOG_STORE_ID => {
                thresholds[i].gog_id = id;
                updated_id = true;
            },
            _ => eprintln!("Unknown store type: {}", store_type),
        }
        if updated_id {
            let update_err = format!("Could not convert the {} id update to a string object.", store_type);
            let data_str = serde_json::to_string(&thresholds).expect(&update_err);
            json::write_to_file(get_path(), data_str);
            let mut store_name = "";
            match store_type {
                settings::STEAM_STORE_ID => store_name = "Steam",
                settings::GOG_STORE_ID => store_name = "GOG",
                _ => (),
            }
            println!("Updated {} ID for \"{}\"", store_name, title);
        }
    }
}

pub fn update_id_str(title: &str, store_type: &str, id: &str){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => eprintln!("Error: {}", e)
    };
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none() {
        let mut updated_id : bool = false;
        let i = idx.unwrap();
        match store_type {
            /*settings::HUMBLE_BUNDLE_STORE_ID => {
                thresholds[i].humble_bundle_id = id.to_string();
                updated_id = true;
            },*/
            _ => eprintln!("Unknown store type: {}", store_type),
        }
        if updated_id {
            let update_err = format!("Could not convert the {} id update to a string object.", store_type);
            let data_str = serde_json::to_string(&thresholds).expect(&update_err);
            json::write_to_file(get_path(), data_str);
            let mut store_name = "";
            /*match store_type {
                settings::HUMBLE_BUNDLE_STORE_ID => store_name = "Humble Bundle",
                _ => (),
            }*/
            println!("Updated {} ID for \"{}\"", store_name, title);
        }  
    }
}

pub fn remove(title: &str){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
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