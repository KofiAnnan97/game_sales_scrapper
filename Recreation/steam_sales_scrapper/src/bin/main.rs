use dotenv::dotenv;
use serde_json::Result;
use serde::{Deserialize, Serialize};
use std::fs::{File, write, read_to_string};
use std::path::Path;
use std::io;
use std::io::Write;
use clap::{arg, command, Arg, ArgAction, Command, ArgMatches};

// Internal libraries
use steam_sales_scrapper::email::mailer;
use steam_sales_scrapper::store_api::steam;

#[derive(Deserialize, Serialize, Debug)]
struct Threshold{
    name: String,
    #[serde(rename="appid")]
    app_id: usize,
    currency: String,
    desired_price: f64,
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

async fn add_game(app: steam::App, price: f64){
    let mut thresholds : Vec<Threshold> = Vec::new();
    match load_thresholds(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    match steam::get_price(app.app_id).await {
        Ok(po) => {
            let mut unique : bool = true;
            for elem in thresholds.iter() {
                if elem.name == app.name {
                    unique = false;
                    break;
                }
            }
            if unique { 
                thresholds.push(Threshold {
                    name: app.name.clone(),
                    app_id: app.app_id.clone(),
                    currency: po.currency.clone(),
                    desired_price: price
                });
                let data_str = serde_json::to_string(&thresholds).unwrap();
                write_to_file(get_load_path(), data_str);
                println!("Successfully added \"{}\".", app.name);
            }
            else { println!("Duplicate title: \"{}\".", app.name); }
                    
        },
        Err(e) => println!("{}", e)
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

// Output Functions
async fn check_prices() -> String {
    let mut thresholds : Vec<Threshold> = Vec::new();
    match load_thresholds(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let mut output = String::from("");
    for elem in thresholds.iter(){
        match steam::get_price(elem.app_id).await {
            Ok(po) => {
                let price_str = format!("\n\t- {} : ${} -> ${} {} ({}% off)", 
                                        elem.name, po.initial, po.final_price, 
                                        po.currency, po.discount_percent);
                if elem.desired_price >= po.final_price {
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
                match steam::search_game(&keyphrase).await {
                    Some(title) =>{
                        let mut input = String::new();
                        print!("Provide price threshold (as float): ");
                        let _ = io::stdout().flush();
                        io::stdin()
                            .read_line(&mut input)
                            .expect("Failed to read user input");
                        let price : f64 = input.trim().parse().unwrap();
                        match steam::check_game(&title).await {
                            Some(data) => add_game(data, price).await,
                            None => println!("Something went wrong")
                        }
                    },
                    None => ()
                }    
            }
        },
        Some(("add", add_args)) => {
            if add_args.contains_id("title"){
                if add_args.contains_id("price"){
                    let title = add_args.get_one::<String>("title").unwrap().clone();
                    let price = add_args.get_one::<f64>("price").unwrap().clone();
                    match steam::check_game(&title).await {
                        Some(app) => add_game(app, price).await,
                        None => {
                            match steam::search_game(&title).await {
                                Some(t) => {
                                    match steam::check_game(&t).await {
                                        Some(data) => add_game(data, price).await,
                                        None => eprintln!("Something went wrong")
                                    }
                                }
                                None => ()
                            }
                        }
                    }
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
            if cmd.get_flag("list") { list_game_thresholds(); }
            else if cmd.get_flag("cache"){
                println!("Caching started");
                steam::update_cached_games().await;
            }
            else if cmd.get_flag("email"){
                let email_str = check_prices().await;
                println!("Email Contents:\n{}\n", email_str);
                if email_str.is_empty(){ println!("No game(s) on sale at price thresholds"); }
                else {
                    println!("Sending email...");
                    let to_address = &get_recipient();
                    mailer::send_email(to_address, "Steam Games At Desired Prices",&email_str);
                }
            }
            else { println!("No/incorrect command given. Use \'--help\' for assistance."); }
        }      
    };
}