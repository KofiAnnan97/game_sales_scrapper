use std::collections::HashMap;
use std::io::{self, Write};
use clap::{arg, command, Arg, ArgAction, Command, ArgMatches};
use clap::parser::ValueSource;
use serde_json::Value;

// Internal libraries
use constants::operations::properties::{PROP_PROJECT_PATH, PROP_TEST_PATH, PROP_RECIPIENT_EMAIL, PROP_SMTP_EMAIL, 
                                        PROP_SMTP_HOST, PROP_SMTP_PORT, PROP_SMTP_USERNAME, PROP_TEST_MODE};
use constants::cli::args::*;
use stores::pc::{steam};
use alerting::email;
use file_types::csv;
use properties;
use file_ops::{settings, thresholds};
use structs::internal::data::{SimpleGameThreshold};
use structs::internal::enums::GameStore;
use gss_cli::{check_prices, gog_insert_sequence, microsoft_store_insert_sequence, steam_insert_sequence, storefront_check};

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
    let alias_arg = arg!(-a --alias "Add an alias to Game title (optional)")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(String))
        .required(false);
    let file_arg = arg!(-f --file "Provide CSV file")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(String))
        .required(true);

    let cmd : ArgMatches = command!()
        .about("A simple script for checking prices on games.")
        .subcommand(
            Command::new("config")
                .about("Set script settings and properties")
                .subcommand(
                    Command::new("settings")
                       .about("Configure settings") 
                       .arg(
                            arg!(-s --steam "Search Steam Store")
                                .action(ArgAction::SetTrue)
                                .required(false)
                        )
                        .arg(
                            arg!(-g --gog "Search Good Old Games (GOG) Store")
                                .action(ArgAction::SetTrue)
                                .required(false)
                        )
                        .arg(
                            arg!(-m --microsoft_store "Search Microsoft Store")
                                .action(ArgAction::SetTrue)
                                .required(false)
                        )
                        .arg(
                            arg!(-a --all_stores "Search all game stores")
                                .action(ArgAction::SetTrue)
                                .conflicts_with_all(["steam", "gog", "microsoft_store"])
                                .required(false)
                        )
                        .arg(
                            arg!(-e --enable_aliases "Enable aliases for game titles (Possible options: [0,1])")
                                .action(ArgAction::Set)
                                .value_parser(clap::value_parser!(i32))
                                .required(false)
                        )
                        .arg(
                            arg!(-r --allow_alias_reuse "Enable alias reuse after initial creation (Possible options: [0,1])")
                                .action(ArgAction::Set)
                                .value_parser(clap::value_parser!(i32))
                                .required(false)
                        )
                       
                )
                .subcommand(
                    Command::new("properties")
                        .about("Configure properties")
                        .arg(
                            Arg::new(FROM_ENV) 
                                .short('f')
                                .long(FROM_ENV)
                                .action(ArgAction::SetTrue)
                                .conflicts_with_all(["test_mode", SET_SMTP, SET_RECIPIENT, SET_API_KEY, 
                                                     SET_PROJECT_PATH, SET_TEST_PATH, LIST_PROPERTIES])
                                .required(false)
                                .help("Set/update properties from .env file")                      
                        )
                        .arg(
                            Arg::new(SET_SMTP) 
                                .short('s')
                                .long(SET_SMTP)
                                .action(ArgAction::SetTrue)
                                .conflicts_with_all(["test_mode"])
                                .required(false)
                                .help("Set SMTP properties in properties")                      
                        )
                        .arg(
                            Arg::new(SET_RECIPIENT) 
                                .short('r')
                                .long(SET_RECIPIENT)
                                .action(ArgAction::Set)
                                .conflicts_with_all(["test_mode"])
                                .required(false)
                                .help("Set recipient email in properties")                      
                        )
                        .arg(
                            Arg::new(SET_API_KEY) 
                                .short('a')
                                .long(SET_API_KEY)
                                .action(ArgAction::Set)
                                .conflicts_with_all(["test_mode"])
                                .required(false)
                                .help("Set Steam API key in properties")                      
                        )
                        .arg(
                            Arg::new(SET_PROJECT_PATH) 
                                .short('p')
                                .long(SET_PROJECT_PATH)
                                .action(ArgAction::Set)
                                .conflicts_with_all(["test_mode"])
                                .required(false)
                                .help("Set project path in properties")                      
                        )
                        .arg(
                            Arg::new(SET_TEST_PATH) 
                                .short('t')
                                .long(SET_TEST_PATH)
                                .action(ArgAction::Set)
                                .conflicts_with_all(["test_mode"])
                                .required(false)
                                .help("Set test path in properties")                      
                        )
                        .arg(
                            Arg::new(LIST_PROPERTIES) 
                                .short('l')
                                .long(LIST_PROPERTIES)
                                .action(ArgAction::SetTrue)
                                .conflicts_with_all(["test_mode", FROM_ENV, SET_SMTP, SET_RECIPIENT, 
                                                     SET_API_KEY, SET_PROJECT_PATH, SET_TEST_PATH])
                                .required(false)
                                .help("List properties")                      
                        )
                        .arg(
                            Arg::new(REVEAL_SECRETS) 
                                .short('v')
                                .long(REVEAL_SECRETS)
                                .action(ArgAction::SetTrue)
                                .conflicts_with_all(["test_mode", FROM_ENV, SET_SMTP, SET_RECIPIENT, 
                                                     SET_API_KEY, SET_PROJECT_PATH, SET_TEST_PATH])
                                .required(false)
                                .help("Reveal secrets as plain text (only works with list-properties)")                      
                        )
                        .arg(
                            arg!(-z --test_mode "Flag for saving data using the TEST_PATH env variable")
                                .action(ArgAction::Set)
                                .value_parser(clap::value_parser!(i32))
                                .hide(true)
                                .required(false)
                        )
                )
        )
        .subcommand(
            Command::new("add")
                .about("Add a game to price thresholds")
                .args([&title_arg, &price_arg, &alias_arg])
        )
        .subcommand(
            Command::new("bulk-insert")
                .about("Add multiple games via CSV file")
                .args([&file_arg])
        )
        .subcommand(
            Command::new("update")
                .about("Update game thresholds")
                .subcommand(
                    Command::new("price")
                        .about("Update the price of a game")
                        .args([&title_arg, &price_arg])
                )
                .subcommand(
                    Command::new("alias")
                        .about("Update the alias of a game")
                        .args([&title_arg, &alias_arg])
                )
        )
        .subcommand(
            Command::new("remove")
                .about("Remove game from price thresholds")
                .args([&title_arg])
        )
        .arg(
            Arg::new(LIST_SELECTED_STORES)
                .short('l')
                .long(LIST_SELECTED_STORES)
                .action(ArgAction::SetTrue)
                .conflicts_with_all([LIST_THRESHOLDS, UPDATE_CACHE, SEND_EMAIL, CHECK_PRICES])
                .required(false)
                .help("Display the selected storefronts")
        )
        .arg(
            Arg::new(LIST_THRESHOLDS)
                .short('t')
                .long(LIST_THRESHOLDS)
                .action(ArgAction::SetTrue)
                .conflicts_with_all([UPDATE_CACHE, SEND_EMAIL, LIST_SELECTED_STORES, CHECK_PRICES])
                .required(false)
                .help("List all game price thresholds")
        )
        .arg(
            Arg::new(UPDATE_CACHE)
                .short('c')
                .long(UPDATE_CACHE)
                .action(ArgAction::SetTrue)
                .conflicts_with_all([LIST_THRESHOLDS, SEND_EMAIL, LIST_SELECTED_STORES, CHECK_PRICES])
                .required(false)
                .help("Updated cached list of games")
        )
        .arg(
            Arg::new(CHECK_PRICES)
                .short('p')
                .long(CHECK_PRICES)
                .action(ArgAction::SetTrue)
                .conflicts_with_all([LIST_THRESHOLDS, UPDATE_CACHE, LIST_SELECTED_STORES, SEND_EMAIL])
                .required(false)
                .help("Print out which games are on sale")
        )
        .arg(
            Arg::new(SEND_EMAIL)
                .short('e')
                .long(SEND_EMAIL)
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all([LIST_THRESHOLDS, UPDATE_CACHE, LIST_SELECTED_STORES, CHECK_PRICES])
                .required(false)
                .help("Send email if game(s) are below price threshold")
        )
        .get_matches();
    
    match cmd.subcommand() {
        Some(("config", config_args)) => {
            match config_args.subcommand(){
                Some(("settings", settings_args)) => {
                    // Parameters
                    let enable_aliases = settings_args.value_source("enable_aliases");
                    let allow_alias_reuse = settings_args.value_source("allow_alias_reuse");

                    // Stores
                    let search_steam = settings_args.value_source("steam").unwrap();
                    let search_gog = settings_args.value_source("gog").unwrap();
                    let search_microsoft_store = settings_args.value_source("microsoft_store").unwrap();
                    let search_all = settings_args.value_source("all_stores").unwrap();

                    let mut selected : Vec<GameStore> = Vec::new();
                    if search_steam == ValueSource::CommandLine { selected.push(GameStore::STEAM); }
                    if search_gog == ValueSource::CommandLine { selected.push(GameStore::GOOD_OLD_GAMES); }
                    if search_microsoft_store == ValueSource::CommandLine { selected.push(GameStore::MICROSOFT_STORE_PC); }
                    if search_all == ValueSource::CommandLine { selected = settings::get_available_stores(); }
                    if selected.len() > 0 { settings::update_selected_stores(selected); }

                    // If alias state is used
                    match enable_aliases {
                        Some(val_src) => {
                            if val_src == ValueSource::CommandLine {
                                let alias_state : i32 = settings_args.get_one::<i32>("enable_aliases").unwrap().clone();
                                if alias_state == 0 || alias_state == 1{ settings::update_alias_state(alias_state); }
                                else { panic!("enable_aliases must be set to 0 or 1 not \'{}\'", alias_state); }
                            }
                        },
                        None => ()
                    }
                    // If allow alias reuse is used
                    match allow_alias_reuse {
                        Some(val_src) => {
                            if val_src == ValueSource::CommandLine {
                                let alias_state : i32 = settings_args.get_one::<i32>("allow_alias_reuse").unwrap().clone();
                                if alias_state == 0 || alias_state == 1{ settings::update_alias_reuse_state(alias_state); }
                                else { panic!("allow_alias_reuse must be set to 0 or 1 not \'{}\'", alias_state); }
                            }
                        },
                        None => ()
                    }
                },
                Some(("properties", properties_args)) => {
                    // Update properties from env
                    let from_env = properties_args.value_source(FROM_ENV).unwrap();
                    if from_env == ValueSource::CommandLine { properties::update_properties_from_env(); }
                    else if from_env == ValueSource::DefaultValue {
                        match properties_args.value_source("test_mode") {
                            Some(test_mode)  => {
                                if test_mode == ValueSource::CommandLine {
                                    let test_state: i32 = properties_args.get_one::<i32>("test_mode").unwrap().clone();
                                    if test_state == 1 { properties::set_test_mode(true); } else { properties::set_test_mode(false); }
                                    println!("Test mode set to {}", test_state);
                                }
                            },
                            None => ()
                        }
                    }

                    // Set SMTP variables
                    match properties_args.value_source(SET_SMTP){
                        Some(val_src) => {
                            if val_src == ValueSource::CommandLine{
                                let mut host = String::new();
                                print!("SMTP Hostname: ");
                                let _ = io::stdout().flush();
                                io::stdin()
                                    .read_line(&mut host)
                                    .expect("Failed to read user input");
                                host = host[0..host.len()-1].to_string();
                                let mut port_str = String::new();
                                print!("SMTP Port: ");
                                let _ = io::stdout().flush();
                                io::stdin()
                                    .read_line(&mut port_str)
                                    .expect("Failed to read user input");
                                let port_num: u16 = (&port_str.trim()).parse::<u16>().expect("Could not convert value to integer");
                                let mut email = String::new();
                                print!("SMTP Email: ");
                                let _ = io::stdout().flush();
                                io::stdin()
                                    .read_line(&mut email)
                                    .expect("Failed to read user input");
                                email = email[0..email.len()-1].to_string();
                                let mut user = String::new();
                                print!("SMTP User: ");
                                let _ = io::stdout().flush();
                                io::stdin()
                                    .read_line(&mut user)
                                    .expect("Failed to read user input");
                                user = user[0..user.len()-1].to_string();
                                let mut pass = String::new();
                                print!("SMTP Password: ");
                                let _ = io::stdout().flush();
                                io::stdin()
                                    .read_line(&mut pass)
                                    .expect("Failed to read user input");
                                pass = pass[0..pass.len()-1].to_string();
                                properties::set_stmp_vars(host, port_num, email, user, pass);
                            }
                        },
                        None => ()
                    }
                
                    // Set recipient email
                    match properties_args.get_one::<String>(SET_RECIPIENT){
                        Some(recipient) => {
                            let prev_recipient = properties::get_recipient();
                            if !recipient.is_empty() && prev_recipient != *recipient { 
                                properties::set_recipient(recipient); 
                            }
                        },
                        None => ()
                    }

                    // Set Steam api key
                    match properties_args.get_one::<String>(SET_API_KEY){
                        Some(key) => {
                            let prev_key = properties::get_steam_api_key(false);
                            if !key.is_empty() && prev_key != *key{
                                properties::set_steam_api_key(key.to_string());
                            }
                        },
                        None => (),
                    }

                    // Set project path
                    match properties_args.get_one::<String>(SET_PROJECT_PATH){
                        Some(path) => {
                            let prev_path = properties::get_project_path();
                            if !path.is_empty() && prev_path != *path {
                                properties::set_project_path(path);
                            }
                        },
                        None => (),
                    }

                    // Set test path
                    match properties_args.get_one::<String>(SET_TEST_PATH){
                        Some(path) => {
                            let prev_path = properties::get_test_path();
                            if !path.is_empty() && prev_path != *path {
                                properties::set_test_path(path);
                            }
                        },
                        None => (),
                    }

                    // Reveal secrets
                    let reveal_secrets = properties_args.value_source(REVEAL_SECRETS).unwrap();
                    let hidden: bool = if reveal_secrets == ValueSource::CommandLine { false } else { true };

                    // List properties
                    let list_properties = properties_args.value_source(LIST_PROPERTIES).unwrap();
                    if list_properties == ValueSource::CommandLine {
                        match properties::load_properties(){
                            Ok(properties) => {
                                let properties_str = serde_json::to_string(&properties).unwrap();
                                let lookup: HashMap<String, Value> = serde_json::from_str(&properties_str).unwrap();
                                println!("PROPERTIES:\n-----------");
                                let test_mode = lookup.get(PROP_TEST_MODE).unwrap_or_default();
                                match test_mode.as_i64() {
                                    Some(1) => println!("Test Path: {}", lookup.get(PROP_TEST_PATH).unwrap_or_default()),
                                    Some(0) => println!("Project Path: {}", lookup.get(PROP_PROJECT_PATH).unwrap_or_default()),
                                    None => (),
                                    _ => println!("Cannot show path given test_mode '{}'", &test_mode)
                                }
                                println!("Recipient Email: {}", lookup.get(PROP_RECIPIENT_EMAIL).unwrap_or_default());
                                println!("Steam API Key: {}", properties::get_steam_api_key(hidden));
                                println!("SMTP Host: {}", lookup.get(PROP_SMTP_HOST).unwrap_or_default());
                                println!("SMTP Port: {}", lookup.get(PROP_SMTP_PORT).unwrap_or_default());
                                println!("SMTP Email: {}", lookup.get(PROP_SMTP_EMAIL).unwrap_or_default());
                                println!("SMTP User: {}", lookup.get(PROP_SMTP_USERNAME).unwrap_or_default());
                                println!("SMTP Password: {}", properties::get_smtp_pwd(hidden));
                                println!("Test Mode: {}", test_mode);
                            },
                            Err(e) => eprintln!("Failed to list properties.\n{}", e)
                        }
                    }
                }
                _ => ()
            }
        },
        Some(("add", add_args)) => {
            let selected_stores = storefront_check();
            if properties::is_testing_enabled() { println!("------------------------\n* TEST MODE IS ENABLED *\n------------------------"); }
            let alias = if add_args.contains_id("alias") {
                add_args.get_one::<String>("alias").unwrap().clone()
            } else if settings::get_alias_state() {
                thresholds::set_game_alias()
            } else {
                String::new()
            };
            let title = add_args.get_one::<String>("title").unwrap().clone();
            let price = add_args.get_one::<f64>("price").unwrap().clone();
            let http_client = reqwest::Client::new();
            for store in selected_stores{
                if store == GameStore::STEAM {
                    steam_insert_sequence(&alias, &title, price, &http_client).await;
                }
                if store == GameStore::GOOD_OLD_GAMES {
                    gog_insert_sequence(&alias, &title, price, &http_client).await;
                }
                if store == GameStore::MICROSOFT_STORE_PC {
                    microsoft_store_insert_sequence(&alias, &title, price, &http_client).await;
                }
            }
        },
        Some(("bulk-insert", bulk_args)) => {
            let selected_stores = storefront_check();
            if properties::is_testing_enabled() { println!("------------------------\n* TEST MODE IS ENABLED *\n------------------------"); }
            let mut game_list: Vec<SimpleGameThreshold> = Vec::new();
            let file_path = bulk_args.get_one::<String>("file").unwrap().clone();
            match csv::parse_game_prices_from_path(&file_path){
                Ok(gl) => game_list = gl,
                Err(e) => eprintln!("Could not parse file: {}\n{}", file_path, e),
            }
            let http_client = reqwest::Client::new();
            for game in game_list.iter(){
                println!("INSERT GAME -> \"{}\"", game.name);
                let title = &game.name;
                let alias = thresholds::set_game_alias();
                let price: f64 = game.price;
                for store in selected_stores.iter(){
                    if store == &GameStore::STEAM {
                        steam_insert_sequence(&alias, &title, price, &http_client).await;
                    }
                    if store == &GameStore::GOOD_OLD_GAMES {
                        gog_insert_sequence(&alias, &title, price, &http_client).await;
                    }
                    if store == &GameStore::MICROSOFT_STORE_PC {
                        microsoft_store_insert_sequence(&alias, &title, price, &http_client).await;
                    }
                }
            }
        },
        Some(("update", update_args)) => {
            match update_args.subcommand(){
                Some(("price", price_args)) => {
                    if properties::is_testing_enabled() { println!("------------------------\n* TEST MODE IS ENABLED *\n------------------------"); }
                    let title = price_args.get_one::<String>("title").unwrap().clone();
                    let price = price_args.get_one::<f64>("price").unwrap().clone();
                    thresholds::update_price_fuzzy(&title, price);
                },
                Some(("alias", alias_args)) => {
                    if properties::is_testing_enabled() { println!("------------------------\n* TEST MODE IS ENABLED *\n------------------------"); }
                    let title = alias_args.get_one::<String>("title").unwrap().clone();
                    let alias = alias_args.get_one::<String>("alias").unwrap().clone();
                    thresholds::update_threshold_alias_fuzzy(title, &alias);
                },
                _ => (),
            }  
        },
        Some(("remove", remove_args)) => {
            if properties::is_testing_enabled() { println!("------------------------\n* TEST MODE IS ENABLED *\n------------------------"); }
            let title = remove_args.get_one::<String>("title").unwrap().clone();
            thresholds::remove_fuzzy(&title);
        },
        _ => {
            if properties::is_testing_enabled() { println!("------------------------\n* TEST MODE IS ENABLED *\n------------------------"); }
            if cmd.get_flag(LIST_THRESHOLDS) { thresholds::list_games(); }
            else if cmd.get_flag(LIST_SELECTED_STORES) { settings::list_selected_stores(); }
            else if cmd.get_flag(UPDATE_CACHE){
                println!("Caching started (this might take a while)...");
                steam::update_cached_games().await;
            }
            else if cmd.get_flag(CHECK_PRICES) {
                let use_html = false;
                let prices_str = check_prices(use_html).await;
                if !prices_str.is_empty() {
                    println!("------------\nCHECK PRICES\n------------\n{}", prices_str);
                }
            }
            else if cmd.get_flag(SEND_EMAIL){
                email::params_check();
                let use_html = true;
                let sales_str = check_prices(use_html).await;
                let html_body = format!(r#"{}"#, email::create_html_body(&sales_str));
                println!("Email Contents:\n{}", html_body);
                if sales_str.is_empty(){ println!("No game(s) on sale at price thresholds"); }
                else {
                    println!("Sending email...");
                    let to_address = &properties::get_recipient();
                    email::send_html_msg(to_address, "Check Out Which Games Are On Sale", &html_body);
                }
            }
            else { println!("No/incorrect command given. Use \'--help\' for assistance."); }
        }
    };
}