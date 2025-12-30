//use std::vec;
use serde_json::Result;
use serde_json::{Value, json};
use std::fs::{read_to_string, metadata};
use std::collections::HashMap;
use std::path::PathBuf;

use file_types::json;

static CONFIG_FILENAME : &str = "config.json";
static ALIAS_ENABLED : i32 = 1;
static ALIAS_DISABLED : i32 = 0;

// Store IDs
pub const STEAM_STORE_ID : &str = "steam";
pub const GOG_STORE_ID : &str = "gog";
pub const MICROSOFT_STORE_ID : &str = "microsoft_store";

// Store Names (Plain text)
pub const STEAM_STORE_NAME : &str = "Steam";
pub const GOG_STORE_NAME : &str = "Good Old Games (GOG)";
pub const MICROSOFT_STORE_NAME : &str = "Microsoft Store (PC)";

fn get_store_map() -> HashMap<String, String> {
    let store_map = HashMap::from([
        (STEAM_STORE_ID.to_string(), STEAM_STORE_NAME.to_string()),
        (GOG_STORE_ID.to_string(), GOG_STORE_NAME.to_string()),
        (MICROSOFT_STORE_ID.to_string(), MICROSOFT_STORE_NAME.to_string()),
    ]);
    store_map
}

pub fn get_available_stores() -> Vec<String> {
    let stores_map = get_store_map();
    let mut available_stores: Vec<String> = Vec::new();
    for key in stores_map.keys(){
        available_stores.push(key.to_string());
    }
    available_stores
}

pub fn get_proper_store_name(id: &str) -> Option<String> {
    let stores_map = get_store_map();
    if stores_map.contains_key(id){
        return Some(stores_map[id].clone());
    }
    None
}

fn get_path() -> String{
    let path_buf: PathBuf = [json::get_data_path(), CONFIG_FILENAME.to_string()].iter().collect();
    let config_path = path_buf.display().to_string();
    let path_str = json::get_path(&config_path);  //Creates file if it does not exist already
    match metadata(&path_str){
        Ok(md) => {
            if md.len() == 0 {
                let settings = json!({"selected_stores": [], "alias_enabled": 1});
                let settings_str = serde_json::to_string_pretty(&settings);
                json::write_to_file(config_path.to_string(), settings_str.expect("Initial settings could not be created."));
            }
        },
        Err(e) => eprintln!("Error: {}", e)
    }
    path_str
}

pub fn load_data() -> Result<Value> {
    let filepath = get_path();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data)?;
    Ok(body)
}

pub fn get_selected_stores() -> Vec<String> {
    let filepath = get_path();
    let mut stores : Vec<String> = Vec::new();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get selected stores - could not convert to JSON");
    let selected = serde_json::to_string(&body["selected_stores"]).unwrap();
    match serde_json::from_str::<Vec<String>>(&selected){
        Ok(data) => stores = data,
        Err(e) => eprintln!("Error: {}", e)
    };
    stores
}

pub fn get_alias_state() -> bool {
    let filepath = get_path();
    let mut state : bool = true;
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get alias state - could not convert to JSON");
    let alias_enabled =serde_json::to_string(&body["alias_enabled"]).unwrap();
    match serde_json::from_str::<i32>(&alias_enabled){
        Ok(state_val) => {
            if state_val == 1 { state = true; }
            else { state = false; }
        },
        Err(e) => eprintln!("Error: {}", e)
    }
    state
}

pub fn update_selected_stores(selected: Vec<String>) {
    match load_data(){
        Ok(data) => {
            let mut settings = data;
            let selected_stores = settings.get_mut("selected_stores").unwrap();
            let mut unique_stores : Vec<String> = Vec::new();
            for store in selected {
                if !unique_stores.contains(&store) { unique_stores.push(store); }
            }
            *selected_stores = json!(unique_stores);
            let settings_str = serde_json::to_string_pretty(&settings);
            json::write_to_file(get_path(), settings_str.expect("Cannot update store search settings"));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn update_alias_state(is_enabled: i32){
    match load_data(){
        Ok(data) => {
            let mut settings = data;
            let enabled_status = if is_enabled == ALIAS_ENABLED || is_enabled == ALIAS_DISABLED { is_enabled } else { ALIAS_DISABLED };
            *settings.get_mut("alias_enabled").unwrap() = json!(enabled_status);
            let settings_str = serde_json::to_string_pretty(&settings);
            json::write_to_file(get_path(), settings_str.expect("Cannot set state of aliases"));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn list_selected(){
    let available_stores = get_available_stores();
    let selected = get_selected_stores();
    println!("Selected Stores");
    for a_store in available_stores.iter(){
        let mut is_selected = false;
        let proper_name = get_proper_store_name(a_store).unwrap();
        for s_store in selected.iter() {
            if a_store == s_store {
                is_selected = true;
                break;
            }
        }
        if is_selected { println!("  [X] {}", proper_name); }
        else { println!("  [ ] {}", proper_name); }
    }
}