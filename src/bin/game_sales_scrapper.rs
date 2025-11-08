use dotenv::dotenv;
use std::io::Write;
use std::io;
use clap::{arg, command, Arg, ArgAction, Command, ArgMatches};
use clap::parser::ValueSource;

// Internal libraries
use game_sales_scrapper::stores::{steam, gog}; //, humble_bundle};
use game_sales_scrapper::alerting::email;
use game_sales_scrapper::file_ops::{csv, settings, thresholds};

fn get_recipient() -> String {
    dotenv().ok();
    let recipient = std::env::var("RECIPIENT_EMAIL").expect("RECIPIENT_EMAIL must be set");
    return recipient;
}

fn storefront_check() -> Vec<String> {
    let selected_stores = settings::get_selected_stores();
    if selected_stores.len() == 0 {
        panic!("Please configure which stores to query. Run \'game_sales_scrapper config --help\' for more info.");
    }
    selected_stores
}

fn set_game_alias() -> String {
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

async fn check_prices() -> String {
    let mut thresholds : Vec<thresholds::GameThreshold> = Vec::new();
    match thresholds::load_data(){
        Ok(data) => thresholds = data,
        Err(e) => println!("Error: {}", e)
    }
    let mut steam_output = String::from("");
    let mut gog_output = String::from("");
    //let mut humble_bundle_output = String::from("");
    let http_client = reqwest::Client::new();
    for elem in thresholds.iter(){
        if elem.steam_id != 0 {
            match steam::get_price(elem.steam_id, &http_client).await {
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
            if gog::VERSION == 1{
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
            if gog::VERSION == 2{
                match gog::get_price_v2(&elem.title, &http_client).await {
                    Some(po) => {
                        let current_price = po.final_money.amount.parse::<f64>().unwrap();
                        if elem.desired_price >= current_price {
                            let mut discount = String::new();
                            match po.discount {
                                Some(d) => discount = d,
                                None => (),
                            }
                            let price_str = format!("\n\t- {} : {} -> {} {} ({} off)",
                                                elem.title, po.base_money.amount, po.final_money.amount, 
                                                po.base_money.currency, discount[1..].to_string());
                            gog_output.push_str(&price_str);
                        }
                    },
                    None => ()
                }
            }
        }
        /*if !elem.humble_bundle_id.is_empty() {
            match humble_bundle::get_price(&elem.humble_bundle_id, &http_client).await {
                Some(po) => {
                    let current_price = po.amount;
                    if elem.desired_price >= current_price {
                        let price_str = format!("\n\t- {} : {} {}", 
                                                elem.title, current_price, po.currency);
                        humble_bundle_output.push_str(&price_str);
                    }
                },
                None => ()
            }
        }*/
    }
    if !steam_output.is_empty(){
        steam_output = "Steam price thresholds that have been met:".to_owned() + &steam_output;
    }
    if !gog_output.is_empty(){
        gog_output = "\n\nGOG price thresholds that have been met:".to_owned() + &gog_output;
    }
    /*if !humble_bundle_output.is_empty(){
        humble_bundle_output = "\n\nHumble Bundle price thresholds that have been met:".to_owned() + &humble_bundle_output;
    }*/
    let output = steam_output + &gog_output; // + &humble_bundle_output;
    return output;
}

async fn steam_insert_sequence(alias: &str, title: &str, price: f64, client: &reqwest::Client) {
    match steam::check_game(title).await {
        Some(data) => thresholds::add_steam_game(alias.to_string(), data, price, &client).await,
        None => {
            match steam::search_game(title).await {
                Some(t) => {
                    match steam::check_game(&t).await {
                            Some(data) => thresholds::add_steam_game(alias.to_string(), data, price, &client).await,
                            None => eprintln!("Something went wrong")
                    }
                }
                None => ()
            }
        }
    }
}

async fn gog_insert_sequence(alias: &str, title: &str, price: f64, client: &reqwest::Client){
    let mut search_list : Vec<gog::GameInfo> = Vec::new();
    match gog::search_game_by_title_v2(title, &client).await {
        Ok(data) => search_list = data,
        Err(e) => println!("Search GOG Game Error: {}", e)
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
                        //title = &search_list[idx].title;
                        let game = &search_list[idx];
                        thresholds::add_gog_game(alias.to_string(), game, price);
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

/*async fn humble_bundle_insert_sequence(alias: &str, title: &str, price: f64, client: &reqwest::Client){
    let mut search_list: Vec<humble_bundle::GameInfo> = Vec::new();
    humble_bundle::search_game_v2(&title).await //{
        //Ok(data) => search_list = data,
        //Err(e) => println!("Search GOG Game Error: {}", e)
    //}
    /*match humble_bundle::search_game(&title, &http_client).await {
        Ok(data) => search_list = data,
        Err(e) => println!("Search GOG Game Error: {}", e)
    }
    if search_list.len() > 0 {
        print!("Humble Bundle search results: ");
        for (i, game) in search_list.iter().enumerate(){
            println!("  [{}] {}", i, game.human_name);
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
                        title = search_list[idx].human_name.clone();
                        let game = &search_list[idx];
                        thresholds::add_humble_bundle_game(alias.clone(), game, price);
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
        println!("Could not find a game title matching \"{}\" on Humble Bundle.", title);
    }*/
}*/

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
    let steam_store_arg = arg!(-s --steam "Search Steam Store")
        .action(ArgAction::SetTrue)
        .required(false);
    let gog_store_arg = arg!(-g --gog "Search Good Old Games (GOG) Store")
        .action(ArgAction::SetTrue)
        .required(false);
    /*let humble_bundle_store_arg =  arg!(-b --humble_bundle "Search Humble Bundle Store")
        .action(ArgAction::SetTrue)
        .required(false);*/
    let all_stores_arg = arg!(-a --all_stores "Search all game stores")
        .action(ArgAction::SetTrue)
        .exclusive(true)
        .required(false);
    let alias_state_arg = arg!(-i --alias_state "Enable aliases for game titles (Possible options: [0,1])")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(i32))
        .required(false);

    let cmd : ArgMatches = command!()
        .about("A simple script for checking prices on games.")
        .subcommand(
            Command::new("config")
                .about("Set which store fronts are searched")
                .args([
                    &steam_store_arg,
                    &gog_store_arg,
                    //&humble_bundle_store_arg,
                    &all_stores_arg,
                    &alias_state_arg
                ])
        )
        .subcommand(
            Command::new("add")
                .about("Add a game to price thresholds")
                .args([&title_arg, &price_arg, &alias_arg])
        )
        .subcommand(
            Command::new("bulk_insert")
                .about("Add multiple games via CSV file")
                .args([&file_arg])
        )
        .subcommand(
            Command::new("update")
                .about("Update price threshold for game")
                .args([&title_arg, &price_arg])
        )
        .subcommand(
            Command::new("remove")
                .about("Remove game from price thresholds")
                .arg(&title_arg)
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
            let search_steam = config_args.value_source("steam").unwrap();
            let search_gog = config_args.value_source("gog").unwrap();
            //let search_humble_bundle = config_args.value_source("humble_bundle").unwrap();
            let search_all = config_args.value_source("all_stores").unwrap();
            
            let mut selected : Vec<String> = Vec::new();
            if search_steam == ValueSource::CommandLine { selected.push(settings::STEAM_STORE_ID.to_string()); }
            if search_gog == ValueSource::CommandLine { selected.push(settings::GOG_STORE_ID.to_string()); }
            //if search_humble_bundle == ValueSource::CommandLine { selected.push(settings::HUMBLE_BUNDLE_STORE_ID.to_string()); }
            if search_all == ValueSource::CommandLine { selected = settings::get_available_stores(); } 
            if selected.len() > 0 { settings::update_selected_stores(selected); }
            if config_args.contains_id("alias_state"){
                let alias_state : i32 = config_args.get_one::<i32>("alias_state").unwrap().clone();
                if alias_state == 0 || alias_state == 1{ settings::update_alias_state(alias_state); }
                else { panic!("The alias state must be set to 0 or 1 not \'{}\'", alias_state); }
            }
        },
        Some(("add", add_args)) => {
            let selected_stores = storefront_check();
            let mut alias = String::new();
            if add_args.contains_id("alias") && settings::get_alias_state() {
                alias = add_args.get_one::<String>("alias").unwrap().clone();
            }
            else { alias = set_game_alias(); }
            let title = add_args.get_one::<String>("title").unwrap().clone();
            let price = add_args.get_one::<f64>("price").unwrap().clone();
            let http_client = reqwest::Client::new();
            for store in selected_stores.iter(){
                if store == settings::STEAM_STORE_ID {
                    steam_insert_sequence(&alias, &title, price, &http_client).await;
                } 
                if store == settings::GOG_STORE_ID {
                    gog_insert_sequence(&alias, &title, price, &http_client).await;
                }
                /*if store == settings::HUMBLE_BUNDLE_STORE_ID {
                    humble_bundle_insert_sequence(&alias, &title, price, &http_client).await;
                }*/
            }
        },
        Some(("bulk_insert", bulk_args)) => {
            let selected_stores = storefront_check();
            let mut game_list: Vec<csv::DesiredGamePrice> = Vec::new();
            let file_path = bulk_args.get_one::<String>("file").unwrap().clone();
            match csv::parse_game_prices(&file_path){
                Ok(gl) => game_list = gl,
                Err(e) => eprintln!("Could not parse file: {}\n{}", file_path, e),
            }
            let http_client = reqwest::Client::new();
            for game in game_list.iter(){
                println!("INSERT GAME -> \"{}\"", game.name);
                let title = &game.name;
                let alias = set_game_alias();
                let price: f64 = game.price;
                for store in selected_stores.iter(){
                    if store == settings::STEAM_STORE_ID {
                        steam_insert_sequence(&alias, &title, price, &http_client).await;
                    }
                    if store == settings::GOG_STORE_ID {
                        gog_insert_sequence(&alias, &title, price, &http_client).await;
                    }
                }
            }
        },
        Some(("update", update_args)) => {
            let title = update_args.get_one::<String>("title").unwrap().clone();
            let price = update_args.get_one::<f64>("price").unwrap().clone();
            thresholds::update_price(&title, price);
        },
        Some(("remove", remove_args)) => {
            let title = remove_args.get_one::<String>("title").unwrap().clone();
            thresholds::remove(&title);
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
                    email::send(to_address, "Steam Games At Desired Prices",&email_str);
                }
            }
            else { println!("No/incorrect command given. Use \'--help\' for assistance."); }
        }      
    };
}