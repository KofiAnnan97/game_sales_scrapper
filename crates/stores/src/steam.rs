use dotenv::dotenv as dotenv_linux;
use dotenvy::dotenv as dotenv_windows;
use serde_json::{Result, Value, Error};
use std::fs::read_to_string;
use regex::Regex;
use std::io::{self, Write};
use std::env;
use std::path::PathBuf;

use file_types::json;
use structs::data::{SaleInfo};
use structs::steam::{App, PriceOverview};

static CACHE_FILENAME : &str = "steam_game_titles_cache.json";

static API_BASE_URL : &str = "https://api.steampowered.com";
static STORE_BASE_URL : &str = "https://store.steampowered.com";

static APP_LIST_ENDPOINT : &str = "/IStoreService/GetAppList/v1";
static DETAILS_ENDPOINT : &str = "/api/appdetails";

// Secrets
fn get_api_key() -> String {
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }
    let steam_api_token = match env::var("STEAM_API_KEY"){
        Ok(token) => token,
        Err(_) => panic!("STEAM_API_KEY environment variable not found"),
    };
    steam_api_token
}

// Caching Functions
fn get_cache_path() -> String{
    let path_buf: PathBuf = [json::get_data_path(), CACHE_FILENAME.to_string()].iter().collect();
    let cache_file_path = path_buf.display().to_string();
    json::get_path(&cache_file_path)
}

pub async fn load_cached_games() -> Result<Vec<App>> {
    let filepath = get_cache_path();
    let data = read_to_string(filepath).unwrap();
    let cached_games = serde_json::from_str::<Vec<App>>(&data);
    cached_games
}

pub async fn get_last_appid() -> String {
    let cached_games = load_cached_games().await.unwrap_or(Vec::new());
    if cached_games.len() > 0 {
        if let Some(app) = cached_games.last() {
            let app_id_str = format!("{}", app.app_id);
            return String::from(app_id_str);
        }
    }
    String::from("0")
}

pub async fn update_cached_games(){
    let mut games_list : Vec<App> = Vec::new();
    match load_cached_games().await{
        Ok(data) => games_list = data,
        Err(e) => println!("No cached data. {}", e)
    }
    let mut temp : Vec<App> = Vec::new();
    let client = reqwest::Client::new();
    match get_all_games(&client).await {
        Ok(success) => {
            println!("Updating cached game titles (this will take a while)...");
            let body : Value = serde_json::from_str(&success).expect("Could convert Steam app list to JSON");
            let app_list = serde_json::to_string(&body["response"]["apps"]).unwrap();
            let data = serde_json::from_str::<Vec<App>>(&app_list);
            temp = data.unwrap();
        }, 
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    for game in temp.iter() {
        let mut unique = true;
        for cached_game in games_list.iter() {
            if game.app_id == cached_game.app_id && game.name != "" {
                unique = false;
                break;
            }
        }
        if unique && game.name != "".to_string() {
            games_list.push(App {
                name: game.name.clone(),
                app_id: game.app_id.clone(),
                last_modified: game.last_modified,
                price_change_number: game.price_change_number,
            });
        }
    }
    let data_str = serde_json::to_string_pretty(&games_list).unwrap();
    json::write_to_file(get_cache_path(), data_str);
    println!("Cache update complete")
}

// API Functions 
async fn get_all_games(client: &reqwest::Client) -> Result<String> {
    let steam_api_key = get_api_key();
    let last_app_id = get_last_appid().await;
    //println!("Last appid: {}", last_app_id);
    let query_string = [
        ("key", steam_api_key.as_str()),
        ("max_results", "40000"),
        ("last_appid", &last_app_id),
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
    Ok(resp)
}

async fn get_game_data(app_id : usize, client: &reqwest::Client) -> Result<String>{
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

pub async fn get_price(app_id : usize, client: &reqwest::Client) -> Result<PriceOverview>{
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
                        overview.discount_percent = price_overview["discount_percent"].as_f64().unwrap() as usize;
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
        Err(e) => {
            println!("{}", e);
        }
    }
    Ok(overview)
}

pub async fn get_price_details(app_id : usize, client: &reqwest::Client) -> Result<SaleInfo>{
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
            let data = body[app_id.to_string()]["success"].clone();
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
        Err(e) => {
            println!("{}", e);
        }
    }
    Ok(sale_info)
}

// Command Functions
pub async fn check_game(name: &str) -> Option<App> {
    let mut games_list : Vec<App> = Vec::new();
    match load_cached_games().await {
        Ok(data) => games_list = data,
        Err(e) => println!("Error: {}", e)
    }
    if games_list.is_empty() {
        update_cached_games().await;
        match load_cached_games().await {
            Ok(data) => games_list = data,
            Err(e) => println!("Error: {}", e)
        }
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
    let mut games_list : Vec<App> = Vec::new();
    match load_cached_games().await {
        Ok(data) => games_list = data,
        Err(e) => println!("Error: {}", e)
    }
    if games_list.len() == 0 {
        update_cached_games().await;
        match load_cached_games().await {
            Ok(data) => games_list = data,
            Err(e) => println!("Error: {}", e)
        }
    }
    let mut search_list : Vec<String> = Vec::new();
    let re = Regex::new(keyphrase).unwrap();
    for game in games_list.iter(){
        let caps = re.captures(&game.name);
        if !caps.is_none() {
            search_list.push(game.name.clone());
        }
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
                print!("Type integer corresponding to game title or type \"q\" to quit: ");
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