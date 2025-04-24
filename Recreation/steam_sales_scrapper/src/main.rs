extern crate exitcode;

use dotenv::dotenv;
use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};
use std::fs::{File, write, read_to_string};
use std::path::{Path, PathBuf};
use regex::Regex;
use std::io;
use std::process;
use clap::{arg, command, value_parser, Arg, ArgAction, Command, ArgMatches};

mod mailer;

// Structs
#[derive(Deserialize, Serialize, Debug)]
struct App{
    #[serde(rename = "appid")]
    app_id: usize,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Threshold{
    name: String,
    #[serde(rename="appid")]
    app_id: usize,
    currency: String,
    desired_price: f64,
}

struct PriceOverview{
    currency: String,
    discount_percent: usize,
    initial: f64,
    final_price: f64,
}

// Get data from files
fn get_api_key() -> String {
    dotenv().ok();
    let steam_api_token = std::env::var("STEAM_API_KEY").expect("STEAM_API_KEY must be set");
    return steam_api_token;
}

fn get_recipient() -> String {
    dotenv().ok();
    let recipient = std::env::var("RECIPIENT_EMAIL").expect("RECIPIENT_EMAIL must be set");
    return recipient;
}

fn get_load_path() -> String{
    let load_path_str = "./thresholds.json";
    let path = Path::new(load_path_str);
    let mut load_fp = String::from("");
    if !path.is_file(){
        File::create_new(load_path_str).expect("Failed to create load file");
        println!("File created: {}", load_fp);
        load_fp = path.display().to_string();
    } 
    else{
        load_fp = path.display().to_string();
    }
    return load_fp;
}

fn get_cache_path() -> String{
    let cache_path_str = "./cache.json";
    let path = Path::new(cache_path_str);
    let mut cache_fp = String::from("");
    if !path.is_file(){
        File::create_new(cache_path_str).expect("Failed to create cache file");
        println!("File created: {}", cache_fp);
        cache_fp = path.display().to_string();
    } 
    else{
        cache_fp = path.display().to_string();
    }
    return cache_fp;
}

// API Functions 
async fn get_all_games() -> Result<String> {
    let http_client = reqwest::Client::new();
    let steam_key = get_api_key();
    let format = "json";
    let url = format!("https://api.steampowered.com/ISteamApps/GetAppList/v2/?key={}&format={}",steam_key, format);
    let resp = http_client.get(url)
        .send()
        .await
        .expect("Failed to get response")
        .text()
        .await
        .expect("Failed to get data");
    Ok(resp)
}

async fn get_game_data(app_id : usize) -> Result<String>{
    let http_client = reqwest::Client::new();
    let filters = "basic,price_overview";
    let url = format!("https://store.steampowered.com/api/appdetails?appids={}&filters={}", app_id, filters);
    let resp = http_client.get(url)
        .send()
        .await
        .expect("Failed to get response")
        .text()
        .await
        .expect("Failed to get data");
    Ok(resp)
}

async fn get_price(app_id : usize) -> Result<PriceOverview>{
    let mut overview = PriceOverview {
        currency: String::from(""), 
        discount_percent: 0, 
        initial: 0.0, 
        final_price: 0.0,
    };
    match get_game_data(app_id).await {
        Ok(success) => {
            let body : Value = serde_json::from_str(&success).expect("Could convert to JSON");
            let price_overview : &Value = &body[app_id.to_string()]["data"]["price_overview"];
            let data = body[app_id.to_string()]["success"].clone();
            match data{
                serde_json::Value::Bool(true) => {
                    overview.final_price = price_overview["final"].as_f64().unwrap()/100.0;
                    overview.initial = price_overview["initial"].as_f64().unwrap()/100.0;
                    overview.discount_percent = price_overview["discount_percent"].as_f64().unwrap() as usize;
                    overview.currency = price_overview["currency"].to_string();
                },
                serde_json::Value::Bool(false) => {
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

// Caching Functions
async fn load_cached_games() -> Result<Vec<App>> {
    let filepath = get_cache_path();
    let data = read_to_string(filepath).unwrap();
    let temp = serde_json::from_str::<Vec<App>>(&data);
    return temp;
}

async fn update_cached_games(){
    let mut games_list : Vec<App> = Vec::new();
    match load_cached_games().await{
        Ok(data) => games_list = data,
        Err(e) => println!("No cached data.")
    }
    let mut temp : Vec<App> = Vec::new();
    match get_all_games().await {
        Ok(success) => {
            println!("Updating cached game titles (this will take a while)...");
            let body : Value = serde_json::from_str(&success).expect("Could convert to JSON");
            let app_list:  &Value = &body["applist"]["apps"];
            let app_list_str = serde_json::to_string(&body["applist"]["apps"]).unwrap();
            let data = serde_json::from_str::<Vec<App>>(&app_list_str);
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
                app_id: game.app_id.clone()
            });
        }
    }
    let data_str = serde_json::to_string(&games_list).unwrap();
    write_to_file(get_cache_path(), data_str);
    println!("Cache update complete")
}

// JSON Functions
fn write_to_file(path: String, data: String){
    write(path, data).expect("Data could not be saved.");
}

fn load_thresholds() -> Result<Vec<Threshold>> {
    let filepath = get_load_path();
    let data = read_to_string(filepath).unwrap();
    let temp = serde_json::from_str::<Vec<Threshold>>(&data);
    return temp;
}

async fn add_game(name: &str, price: f64){
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
    let mut thresholds : Vec<Threshold> = Vec::new();
    match load_thresholds(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let mut title_exists = false;
    for elem in games_list.iter(){
        if name.to_owned() == elem.name {
            title_exists = true;
            match get_price(elem.app_id).await {
                Ok(po) => {
                    let mut unique : bool = true;
                    for elem in thresholds.iter() {
                        if elem.name == name.to_owned() {
                            unique = false;
                            break;
                        }
                    }
                    if unique { 
                        thresholds.push(Threshold {
                            name: elem.name.clone(),
                            app_id: elem.app_id.clone(),
                            currency: po.currency.clone(),
                            desired_price: price
                        });
                        let data_str = serde_json::to_string(&thresholds).unwrap();
                        write_to_file(get_load_path(), data_str);
                        println!("Successfully added \"{}\".", name);
                    }
                    else { println!("Duplicate title: \"{}\".", name); }
                    
                },
                Err(e) => println!("{}", e)
            }
        }
    }
    if !title_exists { println!("\"{}\" is an incomplete/incorrect game title.", name); }
}

fn remove_game(name: &str){
    let game_name = name.to_owned();
    let mut thresholds : Vec<Threshold> = Vec::new();
    match load_thresholds(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let idx = thresholds.iter().position(|app| *app.name == game_name);
    if !idx.is_none(){
        thresholds.remove(idx.unwrap());
        let data_str = serde_json::to_string(&thresholds).unwrap();
        write_to_file(get_load_path(), data_str);
        println!("Successfully removed \"{}\".", name);
    }
    else { println!("Failed to remove: \"{}\".", name); }
    
}

fn update_game(name: &str, price: f64) {
    let game_name = name.to_owned();
    let mut thresholds : Vec<Threshold> = Vec::new();
    match load_thresholds(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let idx = thresholds.iter().position(|app| *app.name == game_name);
    if !idx.is_none(){
        let i = idx.unwrap();
        if price != thresholds[i].desired_price {
            let old_threshold = thresholds[i].desired_price.clone();
            thresholds[i].desired_price = price;
            println!("\"{}\" updated price threshold from {} to {}", thresholds[i].name,
                                                       old_threshold,
                                                       thresholds[i].desired_price);
            let data_str = serde_json::to_string(&thresholds).unwrap();
            write_to_file(get_load_path(), data_str)
        }
        else {
            println!("\"{}\" did not update. Value given: {}", thresholds[i].name, price);
        }
        
    }
}

fn list_game_thresholds() {
    match load_thresholds(){
        Ok(data) => {
            println!("Price Thresholds");
            for threshold in data.iter() {
                println!("\t- {} => {} ({})", threshold.name, 
                                              threshold.desired_price, 
                                              threshold.currency);
            }
        }
        Err(e) => println!("Error: {}", e)
    }
}

// Search Functions
async fn search_by_keyphrase(keyphrase: &str) -> Result<Vec<String>>{
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

async fn search_game(keyphrase: &str) {
    match search_by_keyphrase(keyphrase).await {
        Ok(data) => {
            let search_list = data;
            println!("Search Results: ");
            for (idx, title) in search_list.iter().enumerate() {
                println!("[{}] {}", idx, title);
            }
            println!("[q] Cancel");
            let mut input = String::new();
            println!("Please choose a value or type \"q\": ");
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read user input");
            let parse_idx = input.parse::<i32>();
            if input.trim() == "q" {
                println!("Cancelling prompt");
            }
            else {
                match input.trim().parse::<i32>() {
                    Ok(idx) => {
                        if idx >= 0 {
                            input = String::new();
                            let title = &search_list[idx as usize];
                            println!("Provide price threshold (as float): ");
                            io::stdin()
                                .read_line(&mut input)
                                .expect("Failed to read user input");
                            let price : f64 = input.trim().parse().unwrap();
                            add_game(&title, price).await;
                        }
                    },
                    Err(e) => println!("Invalid input: {}\nError: {}", input, e)
                }
            }
        }, 
        Err(e) => println!("Error: {}", e)
    }
}

// Output Functions
async fn check_prices() -> String {
    let mut thresholds : Vec<Threshold> = Vec::new();
    match load_thresholds(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let mut output = String::from("");
    for elem in thresholds.iter(){
        let fn_price = get_price(elem.app_id);
        match fn_price.await {
            Ok(po) => {
                let price_str = format!("\n\t- {} : ${} -> ${} {} ({}% off)", 
                                        elem.name, po.initial, po.final_price, 
                                        po.currency, po.discount_percent);
                if elem.desired_price > po.final_price {
                    output.push_str(&price_str);
                }
            },
            Err(e) => println!("{}", e)
        }
    }
    if !output.is_empty(){
        output = "Price Thresholds for the following have been met:".to_owned() + &output;
    }
    return output;
}

// Main function
#[tokio::main]
async fn main(){
    let title_arg = arg!(-t --title "Full title of game")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(String))
        .required(true);
    let price_arg = arg!(-p --price "Price threshold for game (f64)")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(f64))
        .required(true);
    let keyphrase_arg = arg!(-k --keyphrase "Provide a keyphrase for matching to game titles")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(String))
        .required(true);

    let cmd : ArgMatches = command!()
        .about("A simple script for checking prices on Steam games.")
        .subcommand(
            Command::new("search")
                .about("Search for a game")
                .arg(keyphrase_arg.clone())
        )
        .subcommand(
            Command::new("add")
                .about("Add a game to price thresholds")
                .args([title_arg.clone(), price_arg.clone()])
        )
        .subcommand(
            Command::new("update")
                .about("Update price threshold for game")
                .args([title_arg.clone(), price_arg.clone()])
        )
        .subcommand(
            Command::new("remove")
                .about("Remove game from price thresholds")
                .arg(title_arg.clone())
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list-all")
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["cache", "email"])
                .required(false)
                .help("List all game price thresholds")
        )
        .arg(
            Arg::new("cache")
                .short('c')
                .long("update-cache")
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["list", "email"])
                .required(false)
                .help("Updated cached list of games")
        )
        .arg(
            Arg::new("email")
                .short('e')
                .long("send-email")
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["list", "cache"])
                .required(false)
                .help("Send email if game(s) are below price threshold")
        )
    .get_matches();

    match cmd.subcommand() {
        Some(("search", search_args)) => {
            if search_args.contains_id("keyphrase"){
                let keyphrase = search_args.get_one::<String>("keyphrase").unwrap().clone();
                search_game(&keyphrase).await;
            }
        },
        Some(("add", add_args)) => {
            if add_args.contains_id("title"){
                if add_args.contains_id("price"){
                    let title = add_args.get_one::<String>("title").unwrap().clone();
                    let price = add_args.get_one::<f64>("price").unwrap().clone();
                    add_game(&title, price).await;
                }                
            }
        },
        Some(("update", update_args)) => {
            if update_args.contains_id("title"){
                if update_args.contains_id("price"){
                    let title = update_args.get_one::<String>("title").unwrap().clone();
                    let price = update_args.get_one::<f64>("price").unwrap().clone();
                    update_game(&title, price);
                }                
            }
        },
        Some(("remove", remove_args)) => {
            if remove_args.contains_id("title"){
                let title = remove_args.get_one::<String>("title").unwrap().clone();
                remove_game(&title);
            }
        },
        _ => {
            if cmd.get_flag("list") {
                list_game_thresholds();
            }
            else if cmd.get_flag("cache"){
                println!("Caching started");
                update_cached_games().await;
            }
            else if cmd.get_flag("email"){
                let email_str = check_prices().await;
                if email_str.is_empty(){
                    println!("No game(s) on sale at price thresholds");
                }
                else {
                    println!("Sending email...");
                    let to_address = &get_recipient();
                    mailer::send_email(to_address, "Steam Games At Desired Prices",&email_str);
                }
            }
            else {
                println!("No/incorrect command given. Use \'--help\' for assistance.");
            }
        }      
    };
}