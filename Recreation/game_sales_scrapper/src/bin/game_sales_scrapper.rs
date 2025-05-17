use dotenv::dotenv;
use std::io::Write;
use std::io;
use clap::{arg, command, Arg, ArgAction, Command, ArgMatches};
use clap::parser::ValueSource;

// Internal libraries
use game_sales_scrapper::store_api::{steam, gog};
use game_sales_scrapper::email::mailer;
use game_sales_scrapper::data::{settings, thresholds};

fn get_recipient() -> String {
    dotenv().ok();
    let recipient = std::env::var("RECIPIENT_EMAIL").expect("RECIPIENT_EMAIL must be set");
    return recipient;
}

async fn check_prices() -> String {
    let mut thresholds : Vec<thresholds::GameThreshold> = Vec::new();
    match thresholds::load_data(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let mut steam_output = String::from("");
    let mut gog_output = String::from("");
    for elem in thresholds.iter(){
        if elem.steam_id != 0 {
            match steam::get_price(elem.steam_id).await {
                Ok(po) => {
                    if elem.desired_price >= po.final_price {
                        let price_str = format!("\n\t- {} : {} -> {} {} ({}% off)", 
                                            elem.title, po.initial, po.final_price, 
                                            po.currency, po.discount_percent);
                        steam_output.push_str(&price_str);
                    }
                },
                Err(e) => println!("{}", e)
            }
        }
        if elem.gog_id != 0 {
            match gog::get_price(&elem.title).await {
                Some(po) => {
                    let current_price = po.final_amount.parse::<f64>().unwrap();
                    if elem.desired_price >= current_price {
                        let price_str = format!("\n\t- {} : {} -> {} {} ({}% off)", 
                                            elem.title, po.base_amount, po.final_amount, 
                                            po.currency, po.discount_percentage);
                        gog_output.push_str(&price_str);
                    }
                },
                None => ()
            }
        }
    }
    if !steam_output.is_empty(){
        steam_output = "Steam price thresholds that have been met:".to_owned() + &steam_output;
    }
    if !gog_output.is_empty(){
        gog_output = "\n\nGOG price thresholds that have been met:".to_owned() + &gog_output;
    }
    let output = steam_output + &gog_output;
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
    let steam_store_arg = arg!(-s --steam "Search Steam game store")
        .action(ArgAction::SetTrue)
        .required(false);
    let gog_store_arg = arg!(-g --gog "Search Good Old Games (GOG) store")
        .action(ArgAction::SetTrue)
        .required(false);
    let all_stores_arg = arg!(-a --all_stores "Search all game stores")
        .action(ArgAction::SetTrue)
        .exclusive(true)
        .required(false);

    let cmd : ArgMatches = command!()
        .about("A simple script for checking prices on games.")
        .subcommand(
            Command::new("config")
                .about("Set which store fronts are searched")
                .args([
                    steam_store_arg.clone(), 
                    gog_store_arg.clone(), 
                    all_stores_arg.clone()
                ])
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
            Arg::new("selected-stores")
                .short('l')
                .long("list-selected-stores")
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all([ "thresholds", "cache", "email"])
                .required(false)
                .help("Display the selected storefronts")
        )
        .arg(
            Arg::new("thresholds")
                .short('t')
                .long("list-thresholds")
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["cache", "email", "selected-stores"])
                .required(false)
                .help("List all game price thresholds")
        )
        .arg(
            Arg::new("cache")
                .short('c')
                .long("update-cache")
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["thresholds", "email", "selected-stores"])
                .required(false)
                .help("Updated cached list of games")
        )
        .arg(
            Arg::new("email")
                .short('e')
                .long("send-email")
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["thresholds", "cache", "selected-stores"])
                .required(false)
                .help("Send email if game(s) are below price threshold")
        )
    .get_matches();

    match cmd.subcommand() {
        Some(("config", config_args)) => {
            let mut selected : Vec<String> = Vec::new();
            let search_all = config_args.value_source("all_stores").unwrap();
            if search_all == ValueSource::CommandLine {
               selected = settings::get_available_stores();
            } 
            else {
                let search_steam = config_args.value_source("steam").unwrap();
                let search_gog = config_args.value_source("gog").unwrap();
                if search_steam == ValueSource::CommandLine { selected.push(settings::STEAM_STORE_ID.to_owned()); }
                if search_gog == ValueSource::CommandLine { selected.push(settings::GOG_STORE_ID.to_owned())}
            }
            settings::update_selected_stores(selected);
        },
        Some(("add", add_args)) => {
            if add_args.contains_id("title"){
                if add_args.contains_id("price"){
                    let mut title = add_args.get_one::<String>("title").unwrap().clone();
                    let price = add_args.get_one::<f64>("price").unwrap().clone();
                    let selected_stores = settings::get_selected_stores();
                    for store in selected_stores.iter(){
                        if store == settings::STEAM_STORE_ID {
                            match steam::check_game(&title).await {
                                Some(data) => thresholds::add_steam_game(data, price).await,
                                None => {
                                    match steam::search_game(&title).await {
                                        Some(t) => {
                                            title = t.clone();
                                            match steam::check_game(&t).await {
                                                Some(data) => thresholds::add_steam_game(data, price).await,
                                                None => eprintln!("Something went wrong")
                                            }
                                        }
                                        None => ()
                                    }
                                }
                            }
                        } 
                        if store == settings::GOG_STORE_ID {
                            let mut search_list : Vec<gog::Game> = Vec::new();
                            match gog::search_game_by_title(&title).await {
                                Ok(data) => search_list = data,
                                Err(e) => println!("Error: {}", e)
                            }
                            if search_list.len() > 0 {
                                println!("GOG search results:");
                                for (i, game) in search_list.iter().enumerate(){
                                    println!("  [{}] {}", i, game.title);
                                }
                                println!("  [q] Quit");
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
                                                title = search_list[idx].title.clone();
                                                let game = &search_list[idx];
                                                thresholds::add_gog_game(game, price);
                                            }
                                            else if idx >= search_list.len(){
                                                eprintln!("Integer \"{}\" is invalid. Request terminated.", idx);
                                            }
                                        },
                                        Err(e) => println!("Invalid input: {}\nError: {}", input, e)
                                    }
                                }
                            }
                            else{
                                println!("Could not find a game title matching \"{}\" on GOG.", title);
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
                    thresholds::update_price(&title, price);
                }                
            }
        },
        Some(("remove", remove_args)) => {
            if remove_args.contains_id("title"){
                let title = remove_args.get_one::<String>("title").unwrap().clone();
                thresholds::remove(&title);
            }
        },
        _ => {
            if cmd.get_flag("thresholds") { thresholds::list_games(); }
            else if cmd.get_flag("selected-stores") { settings::list_selected(); }
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