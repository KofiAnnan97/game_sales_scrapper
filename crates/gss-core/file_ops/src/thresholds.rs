use std::collections::HashMap; 
use std::io::{self, Write};
use std::path::PathBuf;
use serde_json::{json, Result, Value};
use std::fs::{metadata, read_to_string};

use file_types::general;
use constants::operations::settings::{STEAM_STORE_ID, GOG_STORE_ID, MICROSOFT_STORE_ID,
                                      ALLOW_ALIAS_REUSE_AFTER_CREATION};
use constants::operations::thresholds::*;
use crate::settings::{self, get_alias_reuse_state};
use stores::pc::steam; //, gog, microsoft_store};
use stores::algorithms::fuzzy;
use structs::response::steam::App;
use structs::response::gog::GameInfo as GOGGameInfo;
use structs::response::microsoft_store::ProductInfo;
use structs::internal::data::GameThreshold;
use properties;

pub fn get_path() -> String {
    let path_buf: PathBuf = [properties::get_data_path(), THRESHOLD_FILENAME.to_string()].iter().collect();
    let thresh_path = path_buf.display().to_string();
    let path_str = general::get_path(&thresh_path);
    match metadata(&path_str){
        Ok(md) => {
            if md.len() == 0 {
                let data = json!({
                    THRESHOLDS.to_string(): [],
                    ALIAS_MAP.to_string(): {},
                });
                let data_str = serde_json::to_string_pretty(&data);
                general::write_to_file(thresh_path.to_string(), data_str.expect("Initial settings could not be created."));
            }
        },
        Err(e) => eprintln!("Error: {}", e)
    }
    path_str
}

pub fn load_data() -> Result<Value> {
    let filepath = get_path();
    let data = read_to_string(filepath).unwrap();
    serde_json::from_str(&data)
}

pub fn load_thresholds() -> Result<Vec<GameThreshold>> {
    let filepath = get_path();
    let data = read_to_string(filepath).unwrap();
    let body: Value = serde_json::from_str(&data).expect("Load thresholds - could not convert to JSON");
    let thresholds = serde_json::to_string(&body[THRESHOLDS])?;
    serde_json::from_str::<Vec<GameThreshold>>(&thresholds)
}

pub fn load_alias_map() -> Result<HashMap<String, Vec<String>>> {
    let filepath = get_path();
    let data = read_to_string(filepath).unwrap();
    let body: Value = serde_json::from_str(&data).expect("Load alias map - could not convert to JSON");
    let alias_map = serde_json::to_string(&body[ALIAS_MAP])?;
    serde_json::from_str::<HashMap<String, Vec<String>>>(&alias_map)
}

pub fn update_thresholds(thresholds: Vec<GameThreshold>) {
    match load_data() {
        Ok(data) => {
            let mut thresholds_data = data;
            *thresholds_data.get_mut(THRESHOLDS.to_string()).unwrap() = json!(thresholds);
            let thresholds_str = serde_json::to_string_pretty(&thresholds_data);
            general::write_to_file(get_path(), thresholds_str.expect("Cannot update thresholds"));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn update_alias_map(alias_map: HashMap<String, Vec<String>>){
    match load_data() {
        Ok(data) => {
            let mut alias_data = data;
            *alias_data.get_mut(ALIAS_MAP.to_string()).unwrap() = json!(alias_map);
            let alias_str = serde_json::to_string_pretty(&alias_data);
            general::write_to_file(get_path(), alias_str.expect("Cannot update alais map"));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn update_threshold_alias(game_title: String, new_alias: &str) {
    let mut alias_map = load_alias_map().unwrap_or_default();
    
    // Remove old alias for game in alias map
    let mut thresholds = load_thresholds().unwrap_or_default();
    let mut old_alias: String = String::new();
    if let Some(i) = thresholds.iter().position(|threshold| *threshold.title == game_title){
        old_alias = thresholds[i].alias.to_string();
        thresholds[i].alias = String::from(new_alias);
    }
    if !old_alias.is_empty() && alias_map.contains_key(&old_alias){
        let alias_list = alias_map.get_mut(&old_alias).unwrap();
        if let Some(j) = alias_list.iter().position(|threshold_title| *threshold_title == game_title){
            alias_list.remove(j);
        }
        if alias_list.is_empty(){ alias_map.remove(&old_alias); }
    }

    // Add new alias for game in alias map
    if !new_alias.is_empty() {
        if alias_map.contains_key(new_alias) {
            alias_map.get_mut(new_alias).unwrap().push(game_title);
        }
        else { alias_map.insert(new_alias.to_string(), vec![game_title]); }
    }

    // Save changes to thresholds and alias map
    match load_data() {
        Ok(data) => {
            let mut alias_data = data;
            *alias_data.get_mut(ALIAS_MAP.to_string()).unwrap() = json!(alias_map);
            *alias_data.get_mut(THRESHOLDS.to_string()).unwrap() = json!(thresholds);
            let alias_map_str = serde_json::to_string_pretty(&alias_data);
            general::write_to_file(get_path(), alias_map_str.expect("Cannot update alias map"));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn does_alias_exist(alias_name: &str) -> bool {
    let alias_map = load_alias_map().unwrap_or_default();
    alias_map.contains_key(alias_name)
}

fn is_threshold(title: &str, game_thresh: &GameThreshold) -> bool {
    title == game_thresh.title || title == game_thresh.alias
}

pub async fn add_steam_game(new_alias: String, app: App, price: f64, client: &reqwest::Client){
    let mut thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
    match steam::get_price(app.app_id, &client).await {
        Ok(po) => {
            let mut unique : bool = true;
            for elem in thresholds.iter() {
                if is_threshold(&app.name, elem) {
                    unique = false;
                    if elem.steam_id == 0 {
                        update_id(&elem.title, STEAM_STORE_ID, app.app_id);
                    }
                    break;
                }
            }
            if unique {
                let mut alias_str = String::new();
                if !does_alias_exist(&new_alias) || (does_alias_exist(&new_alias) && get_alias_reuse_state()){
                    update_threshold_alias(app.name.to_string(), &new_alias);
                    alias_str = new_alias;
                }
                else{
                    eprintln!("Alias '{}' is already in use. If needed, set '{}' to 1 in config file.",
                              new_alias, ALLOW_ALIAS_REUSE_AFTER_CREATION);
                }
                thresholds.push(GameThreshold {
                    title: app.name.clone(),
                    alias: alias_str,
                    steam_id: app.app_id.clone(),
                    gog_id: 0,
                    microsoft_store_id: String::new(),
                    currency: po.currency[1..po.currency.len()-1].to_string(),
                    desired_price: price
                });
                update_thresholds(thresholds);
                println!("Successfully added Steam game: \"{}\".", app.name);
            }
            //else { println!("Duplicate title: \"{}\".", app.name); }
        },
        Err(e) => println!("{}", e)
    }
}

pub fn add_gog_game(new_alias: String, game: &GOGGameInfo, price: f64){
    let mut thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
    let mut unique : bool = true;
    for elem in thresholds.iter(){
        if is_threshold(&game.title, elem){
            unique = false;
            if elem.gog_id == 0 {
                let game_id = game.id.parse::<u32>().unwrap();
                update_id(&elem.title, GOG_STORE_ID, game_id);
            }
            break;
        }
    }
    if unique { 
        let currency_code = match &game.price {
            Some(price_data) => price_data.base_money.currency.clone(),
            None => "USD".to_string(),
        };
        let mut alias_str = String::new();
        if !does_alias_exist(&new_alias) || (does_alias_exist(&new_alias) && get_alias_reuse_state()){
            update_threshold_alias(game.title.to_string(), &new_alias);
            alias_str = new_alias;
        }
        else{
            eprintln!("Alias '{}' is already in use. If needed, set '{}' to 1 in config file.",
                      new_alias, ALLOW_ALIAS_REUSE_AFTER_CREATION);
        }
        thresholds.push(GameThreshold {
            title: game.title.clone(),
            alias: alias_str,
            steam_id: 0,
            gog_id: game.id.parse::<u32>().unwrap(),
            microsoft_store_id: String::new(),
            //currency: game.price.currency.clone(), // Version 1
            currency: currency_code,
            desired_price: price
        });
        update_thresholds(thresholds);
        println!("Successfully added GOG game \"{}\".", game.title);
    }
    //else { println!("Duplicate title: \"{}\".", game.title); }
}

pub fn add_microsoft_store_game(new_alias: String, game: &ProductInfo, price: f64){
    let mut thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
    let mut unique : bool = true;
    for elem in thresholds.iter(){
        if is_threshold(&game.title, elem){
            unique = false;
            if elem.microsoft_store_id.is_empty() {
                let game_id = &game.product_id;
                update_id_str(&elem.title, MICROSOFT_STORE_ID, game_id);
            }
            break;
        }
    }
    if unique {
        let mut alias_str = String::new();
        if !does_alias_exist(&new_alias) || (does_alias_exist(&new_alias) && get_alias_reuse_state()){
            update_threshold_alias(game.title.to_string(), &new_alias);
            alias_str = new_alias;
        }
        else{
            eprintln!("Alias '{}' is already in use. If needed, set '{}' to 1 in config file.",
                      new_alias, ALLOW_ALIAS_REUSE_AFTER_CREATION);
        }
        thresholds.push(GameThreshold{
            title: game.title.clone(),
            alias: alias_str,
            steam_id: 0,
            gog_id: 0,
            microsoft_store_id: game.product_id.clone(),
            currency: String::from("USD"),
            desired_price: price
        });
        update_thresholds(thresholds);
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

pub fn update_price(title: &str, price: f64) -> bool {
    let mut thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
    let mut price_updated = false;
    for threshold in thresholds.iter_mut(){
        if is_threshold(title, threshold){
            if price != threshold.desired_price{
                let old_threshold = threshold.desired_price.clone();
                threshold.desired_price = price;
                println!("\"{}\": updated price threshold from {} to {}", threshold.title,
                         old_threshold,
                         threshold.desired_price);
                price_updated = true;
            }
            else{
                println!("Price was not updated because it is already set to {}", price);
            }
        }
    }
    if price_updated{ 
        update_thresholds(thresholds); 
        true
    }
    else{ 
        // println!("\"{}\" does not have a configured threshold.", title); 
        false
    }
}

pub fn update_price_fuzzy(title: &str, price: f64) {
    //Try to run the normal update price runtine first
    let price_updated = update_price(title, price);

    if !price_updated {
        // Find other potential games
        let thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
        let mut fuzzy_threholds: Vec<String> = Vec::new();
        let mut fuzzy_alias: Vec<String> = Vec::new();
        for thresh in thresholds {
            let mut dist = fuzzy::levenshtein_distance(title, thresh.title.as_str());
            if 1.0 - (dist/thresh.title.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                fuzzy_threholds.push(thresh.title);
                fuzzy_alias.push(thresh.alias);
            }
            else if !thresh.alias.is_empty() {
                dist = fuzzy::levenshtein_distance(title, thresh.alias.as_str());
                if 1.0 - (dist/thresh.alias.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                    fuzzy_threholds.push(thresh.title);
                    fuzzy_alias.push(thresh.alias);
                }
            }
        }

        if fuzzy_threholds.is_empty() {
            return;
        }

        println!("Did you mean one of the following?");
        for (idx, game_title) in fuzzy_threholds.iter().enumerate() {
            println!("  [{}] {}", idx, game_title);
        }
        println!("  [c] Cancel");
        let mut input = String::new();
        print!("Update price by index or type \'c\' to cancel: ");
        let _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        if input.trim() == "c" {
            println!("Threshold removal of '{}' cancelled.", title);
        } else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < fuzzy_threholds.len(){
                        let alias = fuzzy_alias[idx].as_str();
                        if alias != "" {
                            update_price(alias, price);
                        } else {
                            update_price(&fuzzy_threholds[idx], price);
                        }
                    }
                    else if idx >= fuzzy_threholds.len(){
                        eprintln!("Integer \"{}\" is invalid. Remove cancelled.", idx);
                    }
                },
                Err(e) => println!("Invalid input: {}\nError: {}", input, e)
            }
        }
    }
}

pub fn update_id(title: &str, store_type: &str, id: u32){
    let mut thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    //println!("{:?}", idx);
    if !idx.is_none() {
        let mut updated_id : bool = false;
        let i = idx.unwrap();
        let mut store_name = String::new();
        match store_type{
            STEAM_STORE_ID => {
                store_name = settings::get_proper_store_name(STEAM_STORE_ID).unwrap();
                thresholds[i].steam_id = id;
                updated_id = true;
            },
            GOG_STORE_ID => {
                store_name = settings::get_proper_store_name(GOG_STORE_ID).unwrap();
                thresholds[i].gog_id = id;
                updated_id = true;
            },
            _ => eprintln!("Unknown store type: {}", store_type),
        }
        if updated_id {
            let _update_err = format!("Could not convert the {} id update to a string object.", store_type);
            update_thresholds(thresholds);
            println!("Updated {} ID for \"{}\"", store_name, title);
        }
    }
}

pub fn update_id_str(title: &str, store_type: &str, id: &str){
    let mut thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
    let idx = thresholds.iter().position(|threshold| is_threshold(title, threshold));
    if !idx.is_none() {
        let mut updated_id : bool = false;
        let i = idx.unwrap();
        let mut store_name = String::new();
        match store_type {
            MICROSOFT_STORE_ID => {
                store_name = settings::get_proper_store_name(MICROSOFT_STORE_ID).unwrap();
                thresholds[i].microsoft_store_id = id.to_string();
                updated_id = true;
            }
            _ => eprintln!("Unknown store type: {}", store_type),
        }
        if updated_id {
            let _update_err = format!("Could not convert the {} id update to a string object.", store_type);
            update_thresholds(thresholds);
            println!("Updated {} ID for \"{}\"", store_name, title);
        }  
    }
}

pub fn remove(title: &str) -> bool {
    let mut alias_map: HashMap<String, Vec<String>> = load_alias_map().unwrap_or_default();
    let mut thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
    let mut threshold_removed = false; 
    if does_alias_exist(title) {
        let title_list = alias_map.get(title).unwrap();
        for game_title in title_list.iter() {
            if let Some(idx) =  thresholds.iter().position(|threshold| *game_title == threshold.title) {
                println!("Removing \"{}\".", thresholds[idx].title);
                thresholds.remove(idx);
                threshold_removed = true;
            };
        }
        alias_map.remove(title);
    } 
    else {
        for i in (0..thresholds.len()).rev(){
            if is_threshold(title, &thresholds[i]){
                if !thresholds[i].alias.is_empty() {
                    match alias_map.get_mut(&thresholds[i].alias) {
                        Some(aliases) => {
                            if let Some(idx) = aliases.iter().position(|title| title == &thresholds[i].title){
                                aliases.remove(idx);
                            };
                            if aliases.is_empty() { alias_map.remove(&thresholds[i].alias); }
                        }, 
                        None => ()
                    }
                }
                println!("Removing \"{}\".", thresholds[i].title);
                thresholds.remove(i);            
                threshold_removed = true;
            }
        }
    }
    if threshold_removed{ 
        update_alias_map(alias_map);
        update_thresholds(thresholds); 
        true
    }
    else { 
        // println!("Failed to remove game using title/alias: \"{}\".", title); 
        false
    }
}

pub fn remove_fuzzy(title: &str) {
    //Try to run the normal remove runtine first
    let is_removed = remove(title);

    if !is_removed {
        // Find other potential games
        let thresholds = load_thresholds().unwrap_or_else(|_e|Vec::new());
        let mut fuzzy_threholds: Vec<String> = Vec::new();
        for thresh in thresholds {
            let mut dist = fuzzy::levenshtein_distance(title, thresh.title.as_str());
            if 1.0 - (dist/thresh.title.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                fuzzy_threholds.push(thresh.title);
            }
            else if !thresh.alias.is_empty() {
                dist = fuzzy::levenshtein_distance(title, thresh.alias.as_str());
                if 1.0 - (dist/thresh.alias.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                    fuzzy_threholds.push(thresh.title);
                }
            }
        }

        if fuzzy_threholds.is_empty() {
            return;
        }

        println!("Did you mean one of the following?");
        for (idx, game_title) in fuzzy_threholds.iter().enumerate() {
            println!("  [{}] {}", idx, game_title);
        }
        println!("  [c] Cancel");
        let mut input = String::new();
        print!("Remove game title by index or type \'c\' to cancel: ");
        let _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        if input.trim() == "c" {
            println!("Threshold removal of '{}' cancelled.", title);
        } else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < fuzzy_threholds.len(){
                        remove(&fuzzy_threholds[idx]);
                    }
                    else if idx >= fuzzy_threholds.len(){
                        eprintln!("Integer \"{}\" is invalid. Remove cancelled.", idx);
                    }
                },
                Err(e) => println!("Invalid input: {}\nError: {}", input, e)
            }
        }
    }
}

pub fn list_games() {
    match load_thresholds(){
        Ok(data) => {
            println!("Price Thresholds");
            for threshold in data.iter() {
                if threshold.alias.is_empty() {
                    println!("  - {} => {} ({})", threshold.title, 
                                                threshold.desired_price, 
                                                threshold.currency);
                } else {
                    println!("  - {} [{}] => {} ({})", threshold.title,
                                                    threshold.alias, 
                                                    threshold.desired_price, 
                                                    threshold.currency);
                }
            }
        },
        Err(e) => println!("Error: {}", e)
    }
}