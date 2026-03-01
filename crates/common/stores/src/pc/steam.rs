use serde_json::{Result, Value, Error};
use std::fs::read_to_string;
use regex::Regex;
use std::io::{self, Write};

use std::path::PathBuf;
use file_types::common;
use properties;
use constants::operations::properties::PROP_STEAM_API_KEY;
use structs::internal::data::SaleInfo;
use structs::response::steam::{App, PriceOverview};

static CACHE_FILENAME : &str = "cached_steam_games.json";

static API_BASE_URL : &str = "https://api.steampowered.com";
static STORE_BASE_URL : &str = "https://store.steampowered.com";

static APP_LIST_ENDPOINT : &str = "/IStoreService/GetAppList/v1";
static DETAILS_ENDPOINT : &str = "/api/appdetails";

static NUM_OF_RESULTS : u32 = 40000;
static SLIDING_UPDATE_START_SIZE : usize = 100000;

// Caching Functions
fn get_cache_path() -> String{
    let path_buf: PathBuf = [properties::get_data_path(), CACHE_FILENAME.to_string()].iter().collect();
    let cache_file_path = path_buf.display().to_string();
    common::get_path(&cache_file_path)
}

fn load_cached_games() -> Result<Vec<App>> {
    let filepath = get_cache_path();
    let data = read_to_string(filepath).unwrap();
    let cached_games = serde_json::from_str::<Vec<App>>(&data);
    cached_games
}

fn get_last_appid(cached_games: &Vec<App>) -> u32 {
    if cached_games.len() > 0 && let Some(app) = cached_games.last() {
        app.app_id
    } else {
        0
    }
}

// Adds/updates entries in cache then empties the list contains potentially new games
fn add_entries_to_cache(new_games: &mut Vec<App>, cached_games: &mut Vec<App>){
    let mut game_idx = 0;
    for ng in new_games.iter() {
        let mut unique = true;
        for i in game_idx..cached_games.len() {
            let cached_game = cached_games.get_mut(i).unwrap();
            if ng.app_id == cached_game.app_id {
                if ng.last_modified > cached_game.last_modified {
                    cached_game.name = ng.name.to_string();
                    cached_game.last_modified = ng.last_modified;
                    cached_game.price_change_number = ng.price_change_number;
                }
                game_idx = i;
                unique = false;
                break;
            }
            else if ng.app_id < cached_game.app_id {
                game_idx = i;
                break;
            }
        }
        if unique && ng.name != "".to_string() {
            cached_games.push(App {
                name: ng.name.clone(),
                app_id: ng.app_id.clone(),
                last_modified: ng.last_modified,
                price_change_number: ng.price_change_number,
            });
        }
    }
    new_games.clear();
}

pub async fn update_cached_games(){
    let client = reqwest::Client::new();
    let mut games_list : Vec<App> = load_cached_games().unwrap_or_default();
    let last_appid = get_last_appid(&games_list);
    let mut temp : Vec<App> = get_games(&client, NUM_OF_RESULTS, last_appid).await.unwrap_or_default();
    add_entries_to_cache(&mut temp, &mut games_list);
    let sliding_last_appid = properties::get_sliding_steam_appid();
    if sliding_last_appid < last_appid  && games_list.len() > SLIDING_UPDATE_START_SIZE {
        temp = get_games(&client, NUM_OF_RESULTS, sliding_last_appid).await.unwrap_or_default();
        properties::set_sliding_steam_appid(temp.last().unwrap().app_id);
        add_entries_to_cache(&mut temp, &mut games_list);
    }
    else{ properties::set_sliding_steam_appid(0); }
    println!("Sorting entries...");
    games_list.sort_by(|a, b| a.app_id.cmp(&b.app_id));
    let data_str = serde_json::to_string_pretty(&games_list).unwrap();
    common::write_to_file(get_cache_path(), data_str);
    println!("Cache update complete")
}

// API Functions 
async fn get_games(client: &reqwest::Client, max_results: u32, last_appid: u32) -> Result<Vec<App>> {
    let steam_api_key = properties::get_steam_api_key();
    if steam_api_key.is_empty() { panic!("Missing '{}' property.", PROP_STEAM_API_KEY) }
    let query_string = [
        ("key", steam_api_key.as_str()),
        ("max_results", &max_results.to_string()),
        ("last_appid", &last_appid.to_string()),
        ("format", "json")
    ];
    let url = format!("{}{}/", API_BASE_URL, APP_LIST_ENDPOINT);
    let resp = client.get(url)
        .query(&query_string)
        .send()
        .await
        .expect("Failed to get response")
        .text()
        .await
        .expect("Failed to get data");
    let body : Value = serde_json::from_str(&resp).expect("Could convert Steam app list to JSON");
    let app_list_str = serde_json::to_string(&body["response"]["apps"]).unwrap();
    let app_list = serde_json::from_str::<Vec<App>>(&app_list_str);
    app_list
}

async fn get_game_data(app_id : u32, client: &reqwest::Client) -> Result<String>{
    let app_id_str = app_id.to_string();
    let query_string = [
        ("appids", app_id_str.as_str()),
        ("filters", "basic,price_overview"),
    ];
    let url = format!("{}{}", STORE_BASE_URL, DETAILS_ENDPOINT);
    let resp = client.get(url)
        .query(&query_string)
        .send()
        .await
        .expect("Failed to get response")
        .text()
        .await
        .expect("Failed to get data");
    Ok(resp)
}

pub async fn get_price(app_id : u32, client: &reqwest::Client) -> Result<PriceOverview>{
    let mut overview = PriceOverview {
        currency: String::from(""),
        discount_percent: 0,
        initial: 0.0,
        final_price: 0.0,
    };
    match get_game_data(app_id, &client).await {
        Ok(success) => {
            let body : Value = serde_json::from_str(&success).expect("Could convert to game data json");
            let data = body[app_id.to_string()]["success"].clone();
            match data{
                Value::Bool(true) => {
                    let price_overview : &Value = &body[app_id.to_string()]["data"]["price_overview"];
                    if *price_overview != Value::Null {
                        overview.final_price = price_overview["final"].as_f64().unwrap()/100.0;
                        overview.initial = price_overview["initial"].as_f64().unwrap()/100.0;
                        overview.discount_percent = price_overview["discount_percent"].as_u64().unwrap() as u32;
                        overview.currency = price_overview["currency"].to_string();
                    }
                    else{
                        eprintln!("Could not find pricing data for {:?}", &body[app_id.to_string()]["data"]["name"]);
                    }
                },
                Value::Bool(false) => {
                    eprintln!("Error: No data available for game.");
                    std::process::exit(exitcode::DATAERR);
                },
                _ => panic!("Something strange occurred")
            }
        },
        Err(e) =>  println!("{}", e)
    }
    Ok(overview)
}

pub async fn get_price_details(app_id : u32, client: &reqwest::Client) -> Result<SaleInfo>{
    let mut sale_info = SaleInfo {
        icon_link: String::new(),
        title: String::new(),
        original_price: String::new(),
        current_price: String::new(),
        discount_percentage: String::new(),
        store_page_link: String::new(),
    };
    match get_game_data(app_id, &client).await {
        Ok(success) => {
            let body : Value = serde_json::from_str(&success).expect("Could convert to game data json");
            let data = &body[app_id.to_string()]["success"];
            match data{
                Value::Bool(true) => {
                    let data : &Value = &body[app_id.to_string()]["data"];
                    if *data != Value::Null {
                        sale_info.icon_link = data["header_image"].as_str().unwrap().to_string();
                        sale_info.title = data["name"].as_str().unwrap().to_string();
                        sale_info.original_price = format!("{}", data["price_overview"]["initial"].as_f64().unwrap()/100.0);
                        sale_info.current_price = format!("{}", data["price_overview"]["final"].as_f64().unwrap()/100.0);
                        sale_info.discount_percentage = format!("{}", data["price_overview"]["discount_percent"].as_f64().unwrap() as usize);
                        sale_info.store_page_link = format!("https://store.steampowered.com/app/{}", app_id);
                    }
                    else{
                        eprintln!("Could not find pricing data for {:?}", &body[app_id.to_string()]["data"]["name"]);
                    }
                },
                Value::Bool(false) => {
                    eprintln!("Error: No data available for game.");
                    std::process::exit(exitcode::DATAERR);
                },
                _ => panic!("Something strange occurred")
            }
        },
        Err(e) =>  println!("{}", e)
    }
    Ok(sale_info)
}

// Command Functions
pub async fn check_game(name: &str) -> Option<App> {
    let mut games_list : Vec<App> = load_cached_games().unwrap_or_default();
    if games_list.is_empty() {
        update_cached_games().await;
        games_list = load_cached_games().unwrap_or_default();
    }
    for elem in games_list.iter(){
        if name.to_owned() == elem.name {
            return Ok::<App, Error>(App {
                name: name.to_owned(),
                app_id: elem.app_id,
                last_modified: elem.last_modified,
                price_change_number: elem.price_change_number,
            }).ok();
        }
    }
    None
}

// Search Functions
pub async fn search_by_keyphrase(keyphrase: &str) -> Result<Vec<String>>{
    let mut games_list : Vec<App> = load_cached_games().unwrap_or_default();
    if games_list.len() == 0 {
        update_cached_games().await;
        games_list = load_cached_games().unwrap_or_default();
    }
    let mut search_list : Vec<String> = Vec::new();
    let keyphrase_ignore_case = format!("(?i){}", keyphrase);
    let re = Regex::new(&keyphrase_ignore_case).unwrap();
    for game in games_list.iter(){
        let caps = re.captures(&game.name);
        if !caps.is_none() { search_list.push(game.name.clone()); }
    }
    Ok(search_list)
}

pub async fn search_game(keyphrase: &str) -> Option<String>{
    match search_by_keyphrase(keyphrase).await {
        Ok(search_list) => {
            if !search_list.is_empty() {
                println!("Steam search results:");
                for (idx, game_title) in search_list.iter().enumerate() {
                    println!("  [{}] {}", idx, game_title);
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
                                let title = search_list[idx].clone();
                                return Ok::<String, Error>(title).ok();
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
                println!("Could not find a game title matching \"{}\" on Steam.", keyphrase);
            }
        }, 
        Err(e) => println!("Error: {}", e)
    }
    None
}