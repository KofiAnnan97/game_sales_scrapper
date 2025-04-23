extern crate exitcode;

use dotenv::dotenv;
use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};
use std::fs::{File, write, read_to_string};
use std::path::Path;
use regex::Regex;
use std::io;
use std::process;

/*use clap::Parser;
#[derive(Parser)]
struct Cli{
    pattern: String,
    path: std::path::PathBuf,
}*/

#[derive(Deserialize, Debug)]
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

fn get_api_key() -> String {
    dotenv().ok();
    let steam_api_token = std::env::var("STEAM_API_KEY").expect("STEAM_API_KEY must be set");
    return steam_api_token;
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

fn write_to_load_file(data: String){
    let path = get_load_path();
    write(path, data).expect("Data could not be saved.");
}

async fn search_by_keyphrase(keyphrase: &str) -> Result<Vec<String>>{
    let mut games_list : Vec<App> = Vec::new();
    match get_all_games().await {
        Ok(success) => {
            let body : Value = serde_json::from_str(&success).expect("Could convert to JSON");
            let app_list:  &Value = &body["applist"]["apps"];
            let app_list_str = serde_json::to_string(&body["applist"]["apps"]).unwrap();
            let data = serde_json::from_str::<Vec<App>>(&app_list_str);
            games_list = data.unwrap();
        }, 
        Err(e) => println!("Error: {}", e)
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

// Add error handling for incorrect string 
async fn add_game(name: &str, price: f64){
    let mut games_list : Vec<App> = Vec::new();
    match get_all_games().await {
        Ok(success) => {
            let body : Value = serde_json::from_str(&success).expect("Could convert to JSON");
            let app_list:  &Value = &body["applist"]["apps"];
            let app_list_str = serde_json::to_string(&body["applist"]["apps"]).unwrap();
            let data = serde_json::from_str::<Vec<App>>(&app_list_str);
            games_list = data.unwrap();
        }, 
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    let mut thresholds : Vec<Threshold> = Vec::new();
    match load_thresholds(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    for elem in games_list.iter(){
        if name.to_owned() == elem.name {
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
                        write_to_load_file(data_str);
                        println!("Successfully added \"{}\".", name);
                    }
                    else { println!("Duplicate title: \"{}\".", name); }
                    
                },
                Err(e) => println!("{}", e)
            }
        }
    }
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
        write_to_load_file(data_str);
        println!("Successfully removed \"{}\".", name);
    }
    else { println!("Failed to remove: \"{}\".", name); }
    
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

fn load_thresholds() -> Result<Vec<Threshold>> {
    let filepath = get_load_path();
    let data = read_to_string(filepath).unwrap();
    let temp = serde_json::from_str::<Vec<Threshold>>(&data);
    return temp;
}

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
    output = "Price Thresholds for the following have been met:".to_owned() + &output;
    return output;
}

#[tokio::main]
async fn main(){
    //let args = Cli::parse();    
    //println!("Pattern: {:?}, Path: {:?}", args.pattern, args.path);
    //let api_key = get_api_key();
    
    let command = "SEARCH";
    let title = "The Last of Us™ Part II";
    match command {
        "ADD" => add_game(title, 25.00).await,
        "REMOVE" => remove_game(title),
        "CHECK" => {
            let email_str = check_prices().await;
            println!("{}", email_str);
        },
        "SEARCH" => search_game(title).await,
        &_ => println!("Unknown command: {}", command)
    }

    // Add data and check prices
    /*add_game("The Last of Us™ Part I, 25.00).await;
    add_game("Kunitsu-Gami: Path of the Goddess", 20.00).await;
    add_game("South of Midnight", 20.00).await;
    add_game("Returnal™", 40.00).await;
    add_game("Persona 3 Reload", 30.00).await;
    add_game("The First Berserker: Khazan", 30.00).await;
    add_game("Indiana Jones and the Great Circle", 30.00).await;
    add_game("Sekiro™: Shadows Die Twice", 30.00).await;
    add_game("Disgaea 7: Vows of the Virtueless", 30.00).await;
    add_game("Disgaea 6 Complete", 25.00).await;
    */
}
