use std::vec;
use serde_json::Result;
use serde_json::{Value, json};
use std::fs::{read_to_string, metadata};

use crate::data::json;

static FILE_PATH : &str = "./config.json";

pub const STEAM_STORE_ID : &str = "steam";
pub const GOG_STORE_ID : &str = "gog";

pub fn get_available_stores() -> Vec<String> {
    return vec![
        String::from(STEAM_STORE_ID),
        String::from(GOG_STORE_ID),
    ];
}

fn get_path() -> String{
    let path_str = json::get_path(FILE_PATH);
    match metadata(&path_str){
        Ok(md) => {
            if md.len() == 0 {
                let settings = json!({"selected_stores": [], "alias_enabled": 1});
                let settings_str = serde_json::to_string(&settings);
                json::write_to_file(FILE_PATH.to_string(), settings_str.expect("Initial settings could not be created.")); 
            }
        },
        Err(e) => eprintln!("Error: {}", e)
    }
    return path_str;
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
    return stores;
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
    return state;
}

pub fn update_selected_stores(selected: Vec<String>) {
    match load_data(){
        Ok(data) => {
            let mut settings = data;
            *settings.get_mut("selected_stores").unwrap() = json!(selected);
            let settings_str = serde_json::to_string(&settings);
            json::write_to_file(get_path(), settings_str.expect("Cannot update store search settings"));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn update_alias_state(is_enabled: i32){
    match load_data(){
        Ok(data) => {
            let mut settings = data;
            *settings.get_mut("alias_enabled").unwrap() = json!(is_enabled);
            let settings_str = serde_json::to_string(&settings);
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
        for s_store in selected.iter() {
            if a_store == s_store {
                is_selected = true;
                break;
            }
        }
        if is_selected { println!("  [X] {}", a_store); }
        else { println!("  [ ] {}", a_store); }
    }
}