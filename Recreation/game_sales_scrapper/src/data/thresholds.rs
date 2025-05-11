use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs::read_to_string;

use crate::data::json;
use crate::store_api::{steam, gog};

static FILE_PATH : &str = "./thresholds.json";

#[derive(Deserialize, Serialize, Debug)]
pub struct GameThreshold{
    pub title: String,
    pub aliases: Vec<String>,
    pub steam_id: usize,
    pub gog_id: usize,
    pub currency: String,
    pub desired_price: f64,
}

pub fn get_path() -> String {
    return json::get_path(FILE_PATH);
}

pub fn load_data() -> Result<Vec<GameThreshold>> {
    let filepath = get_path();
    let data = read_to_string(filepath).unwrap();
    let temp = serde_json::from_str::<Vec<GameThreshold>>(&data);
    return temp;
}

fn is_threshold(title: &str, thres: &GameThreshold) -> bool {
    if title == thres.title {
        return true;
    }
    else {
        for alias in thres.aliases.iter(){
            if title == alias {
                return true;
            }
        }
    }
    return false;
}

pub fn add_alias(title: &str, new_alias: &str){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => eprintln!("update_price Error: {}", e)
    };
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none() {
        let i = idx.unwrap();
        thresholds[i].aliases.push(new_alias.to_string());
        let data_str = serde_json::to_string(&thresholds).expect("Could not convert GOG id update to string.");
        json::write_to_file(get_path(), data_str);
    }
    else {
        println!("Could not find threshold with title : \"{}\"", title);
    }
}

pub async fn add_steam_game(app: steam::App, price: f64){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    match steam::get_price(app.app_id).await {
        Ok(po) => {
            let mut unique : bool = true;
            for elem in thresholds.iter() {
                if is_threshold(&app.name, elem) {
                    unique = false;
                    update_steam_id(&elem.title, app.app_id);
                    break;
                }
            }
            if unique { 
                thresholds.push(GameThreshold {
                    title: app.name.clone(),
                    aliases: Vec::new(),
                    steam_id: app.app_id.clone(),
                    gog_id: 0,
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

pub fn add_gog_game(game: &gog::Game, price: f64){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let mut unique : bool = true;
    for elem in thresholds.iter(){
        if is_threshold(&game.title, elem) {
            unique = false;
            update_gog_id(&elem.title, game.id as usize);
            break;
        }
    }
    if unique { 
        thresholds.push(GameThreshold {
            title: game.title.clone(),
            aliases: Vec::new(),
            steam_id: 0,
            gog_id: game.id as usize,
            currency: game.price.currency.clone(),
            desired_price: price
        });
        let data_str = serde_json::to_string(&thresholds).unwrap();
        json::write_to_file(get_path(), data_str);
        println!("Successfully added \"{}\".", game.title);
    }
    //else { println!("Duplicate title: \"{}\".", game.title); }
}

pub fn update_price(title: &str, price: f64) {
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => eprintln!("update_price Error: {}", e)
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

pub fn update_steam_id(title: &str, id: usize){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => eprintln!("update_price Error: {}", e)
    };
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none() {
        let i = idx.unwrap();
        thresholds[i].steam_id = id;
        let data_str = serde_json::to_string(&thresholds).expect("Could not convert Steam id update to string.");
        json::write_to_file(get_path(), data_str);
    }
}

pub fn update_gog_id(title: &str, id: usize){
    let mut thresholds : Vec<GameThreshold> = Vec::new();
    match load_data(){
        Ok(data) => thresholds = data,
        Err(e) => eprintln!("update_price Error: {}", e)
    };
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none() {
        let i = idx.unwrap();
        thresholds[i].gog_id = id;
        let data_str = serde_json::to_string(&thresholds).expect("Could not convert GOG id update to string.");
        json::write_to_file(get_path(), data_str);
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