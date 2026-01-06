use std::io::Write;
use std::io;
use clap::{arg, command, Arg, ArgAction, Command, ArgMatches};
use clap::parser::ValueSource;

// Internal libraries
use stores::{steam, gog, microsoft_store};
use alerting::email;
use file_types::csv;
use properties;
use file_ops::{settings::{self, GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID}, thresholds};
use structs::data::{SaleInfo, SimpleGameThreshold};
use structs::gog::GameInfo as GOGGameInfo;
use structs::microsoft_store::ProductInfo;

fn storefront_check() -> Vec<String> {
    let selected_stores = settings::get_selected_stores();
    if selected_stores.len() == 0 {
        panic!("Please configure which stores to query. Run \'game_sales_scrapper config --help\' for more info.");
    }
    selected_stores
}

fn get_simple_prices_str(store_name: &str, sales: Vec<SaleInfo>) -> String{
    let mut prices_str = String::new();
    for game in sales.iter(){
        prices_str.push_str(&format!("\n\t- {} : {} -> {} ({}% off)",
                                     game.title, game.original_price, game.current_price,
                                     game.discount_percentage));
    }
    if !prices_str.is_empty() {
        let header_str = format!("\n{} game(s) that met your desired price:", store_name);
        prices_str = header_str + &prices_str;
    }
    prices_str
}

async fn check_prices(use_html: bool) -> String {
    let thresholds = thresholds::load_thresholds().unwrap_or_else(|_e|Vec::new());
    let mut steam_sales: Vec<SaleInfo> = Vec::new();
    let mut gog_sales: Vec<SaleInfo> = Vec::new();
    let mut microsoft_store_sales: Vec<SaleInfo> = Vec::new();
    let http_client = reqwest::Client::new();
    let mut output = String::new();
    for elem in thresholds.iter(){
        if elem.steam_id != 0 {
            match steam::get_price_details(elem.steam_id, &http_client).await {
                Ok(info) => {
                    let current_price = info.current_price.parse::<f64>().unwrap();
                    if elem.desired_price >= current_price {
                        steam_sales.push(info);
                    }
                },
                Err(e) => println!("{}", e)
            }
        }
        if elem.gog_id != 0 {
            if gog::VERSION == 1{
                match gog::get_price_details(&elem.title).await {
                    Some(po) => {
                        let current_price = po.final_amount.parse::<f64>().unwrap();
                        if elem.desired_price >= current_price {
                            let price_str = format!("\n\t- {} : {} -> {} {} ({}% off)",
                                                    elem.title, po.base_amount, po.final_amount,
                                                    po.currency, po.discount_percentage);
                            output.push_str(&price_str);
                        }
                    },
                    None => ()
                }
            }
            else if gog::VERSION == 2{
                match gog::get_price_details_v2(&elem.title, &http_client).await {
                    Some(info) => {
                        let current_price = info.current_price.parse::<f64>().unwrap();
                        if elem.desired_price >= current_price {
                            gog_sales.push(info);
                        }
                    },
                    None => ()
                }
            }
        }
        if !elem.microsoft_store_id.is_empty() {
            match microsoft_store::get_price_details(&elem.microsoft_store_id, &http_client).await {
                Some(info) => {
                    let current_price = info.current_price.parse::<f64>().unwrap();
                    if elem.desired_price >= current_price {
                        microsoft_store_sales.push(info);
                    }
                },
                None => ()
            }
        }
    }
    if !steam_sales.is_empty(){
        let store_name = settings::get_proper_store_name(STEAM_STORE_ID).unwrap();
        if use_html { output.push_str(&email::create_storefront_table_html(&store_name, steam_sales)); }
        else { output.push_str(&get_simple_prices_str(&store_name, steam_sales)); }
    }
    if !gog_sales.is_empty(){
        let store_name = settings::get_proper_store_name(GOG_STORE_ID).unwrap();
        if use_html { output.push_str(&email::create_storefront_table_html(&store_name, gog_sales)); }
        else { output.push_str(&get_simple_prices_str(&store_name, gog_sales)); }
    }
    if !microsoft_store_sales.is_empty(){
        let store_name = settings::get_proper_store_name(MICROSOFT_STORE_ID).unwrap();
        if use_html { output.push_str(&email::create_storefront_table_html(&store_name, microsoft_store_sales)); }
        else{ output.push_str(&get_simple_prices_str(&store_name, microsoft_store_sales)); }
    }
    output
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
    let mut search_list : Vec<GOGGameInfo> = Vec::new();
    match gog::search_game_by_title_v2(title, &client).await {
        Ok(data) => search_list = data,
        Err(e) => println!("Search GOG Game Error: {}", e)
    }
    if !search_list.is_empty() {
        println!("GOG search results:");
        for (i, game) in search_list.iter().enumerate(){
            let price = match &game.price{
                Some(po) => po.base_money.amount.clone(),
                None => String::from("0"),
            };
            println!("  [{}] {} - ${}", i, game.title, price);
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

async fn microsoft_store_insert_sequence(alias: &str, title: &str, price: f64, client: &reqwest::Client){
    let mut search_list : Vec<ProductInfo> = Vec::new();
    match microsoft_store::search_game_by_title(title, &client).await {
        Ok(data) => search_list = data,
        Err(e) => println!("Search Microsoft Store Error: {}", e)
    }
    if !search_list.is_empty() {
        println!("Microsoft Store search results:");
        for(i, game) in search_list.iter().enumerate(){
            println!("  [{}] {} - ${}", i, game.title, game.price_info.msrp.unwrap_or_default());
        }
        println!("  [q] SKIP");
        let mut input = String::new();
        print!("Type integer corresponding to game title or type \"q\" to quit: ");
        let _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        if input.trim() == "q" { eprintln!("Request terminated."); }
        else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < search_list.len(){
                        let game = &search_list[idx];
                        thresholds::add_microsoft_store_game(alias.to_string(), game, price);
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
        println!("Could not find a game title matching \"{}\" on the Microsoft Store.", title);
    }
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
    let microsoft_store_arg =  arg!(-m --microsoft_store "Search Microsoft Store")
        .action(ArgAction::SetTrue)
        .required(false);
    let all_stores_arg = arg!(-a --all_stores "Search all game stores")
        .action(ArgAction::SetTrue)
        .conflicts_with_all(["steam", "gog", "microsoft_store"])
        .required(false);
    let enable_aliases_arg = arg!(-e --enable_aliases "Enable aliases for game titles (Possible options: [0,1])")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(i32))
        .required(false);
    let allow_alias_reuse_arg = arg!(-r --allow_alias_reuse "Enable alias reuse after initial creation")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(i32))
        .required(false);
    let update_properties_arg = arg!(-p --update_properties "Update properties using env file")
        .action(ArgAction::SetTrue)
        .conflicts_with("test_mode")
        .required(false);
    let test_mode_arg = arg!(-z --test_mode "Flag for saving data using the TEST_PATH env variable")
        .action(ArgAction::Set)
        .value_parser(clap::value_parser!(i32))
        .hide(true)
        .required(false);

    let cmd : ArgMatches = command!()
        .about("A simple script for checking prices on games.")
        .subcommand(
            Command::new("config")
                .about("Set which store fronts are searched")
                .args([
                    &steam_store_arg,
                    &gog_store_arg,
                    &microsoft_store_arg,
                    &all_stores_arg,
                    &enable_aliases_arg,
                    &allow_alias_reuse_arg,
                    &update_properties_arg,
                    &test_mode_arg
                ])
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
                .about("Update price threshold for game")
                .args([&title_arg, &price_arg])
        )
        .subcommand(
            Command::new("remove")
                .about("Remove game from price thresholds")
                .args([&title_arg])
        )
        .arg(
            Arg::new("selected-stores")
                .short('l')
                .long("list-selected-stores")
                .action(ArgAction::SetTrue)
                .conflicts_with_all([ "thresholds", "cache", "email", "check-prices"])
                .required(false)
                .help("Display the selected storefronts")
        )
        .arg(
            Arg::new("thresholds")
                .short('t')
                .long("list-thresholds")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["cache", "email", "selected-stores", "check-prices"])
                .required(false)
                .help("List all game price thresholds")
        )
        .arg(
            Arg::new("cache")
                .short('c')
                .long("update-cache")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["thresholds", "email", "selected-stores", "check-prices"])
                .required(false)
                .help("Updated cached list of games")
        )
        .arg(
            Arg::new("check-prices")
                .short('p')
                .long("check-prices")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["thresholds", "cache", "selected-stores", "email"])
                .required(false)
                .help("Print out which games are on sale")
        )
        .arg(
            Arg::new("email")
                .short('e')
                .long("send-email")
                .exclusive(true)
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["thresholds", "cache", "selected-stores", "check-prices"])
                .required(false)
                .help("Send email if game(s) are below price threshold")
        )
        .get_matches();

    match cmd.subcommand() {
        Some(("config", config_args)) => {
            // Parameters
            let enable_aliases = config_args.value_source("enable_aliases");
            let allow_alias_reuse = config_args.value_source("allow_alias_reuse");
            let update_properties = config_args.value_source("update_properties").unwrap();

            // Update properties
            if update_properties == ValueSource::CommandLine { properties::update_properties(); }
            else if update_properties == ValueSource::DefaultValue {
                match config_args.value_source("test_mode") {
                    Some(test_mode)  => {
                        if test_mode == ValueSource::CommandLine {
                            let test_state: i32 = config_args.get_one::<i32>("test_mode").unwrap().clone();
                            if test_state == 1 { properties::set_test_mode(true); } else { properties::set_test_mode(false); }
                            println!("Test mode set to {}", test_state);
                        }
                    },
                    None => ()
                }
            }

            // Stores
            let search_steam = config_args.value_source("steam").unwrap();
            let search_gog = config_args.value_source("gog").unwrap();
            let search_microsoft_store = config_args.value_source("microsoft_store").unwrap();
            let search_all = config_args.value_source("all_stores").unwrap();

            let mut selected : Vec<String> = Vec::new();
            if search_steam == ValueSource::CommandLine { selected.push(STEAM_STORE_ID.to_string()); }
            if search_gog == ValueSource::CommandLine { selected.push(GOG_STORE_ID.to_string()); }
            if search_microsoft_store == ValueSource::CommandLine { selected.push(MICROSOFT_STORE_ID.to_string()); }
            if search_all == ValueSource::CommandLine { selected = settings::get_available_stores(); }
            if selected.len() > 0 { settings::update_selected_stores(selected); }

            // If alias state is used
            match enable_aliases {
                Some(val_src) => {
                    if val_src == ValueSource::CommandLine {
                        let alias_state : i32 = config_args.get_one::<i32>("enable_aliases").unwrap().clone();
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
                        let alias_state : i32 = config_args.get_one::<i32>("allow_alias_reuse").unwrap().clone();
                        if alias_state == 0 || alias_state == 1{ settings::update_alias_reuse_state(alias_state); }
                        else { panic!("allow_alias_reuse must be set to 0 or 1 not \'{}\'", alias_state); }
                    }
                },
                None => ()
            }
        },
        Some(("add", add_args)) => {
            let selected_stores = storefront_check();
            let alias = if add_args.contains_id("alias") && settings::get_alias_state() {
                add_args.get_one::<String>("alias").unwrap().clone()
            } else {
                thresholds::set_game_alias()
            };
            let title = add_args.get_one::<String>("title").unwrap().clone();
            let price = add_args.get_one::<f64>("price").unwrap().clone();
            let http_client = reqwest::Client::new();
            for store in selected_stores.iter(){
                if store == STEAM_STORE_ID {
                    steam_insert_sequence(&alias, &title, price, &http_client).await;
                }
                if store == GOG_STORE_ID {
                    gog_insert_sequence(&alias, &title, price, &http_client).await;
                }
                if store == MICROSOFT_STORE_ID {
                    microsoft_store_insert_sequence(&alias, &title, price, &http_client).await;
                }
            }
        },
        Some(("bulk-insert", bulk_args)) => {
            let selected_stores = storefront_check();
            let mut game_list: Vec<SimpleGameThreshold> = Vec::new();
            let file_path = bulk_args.get_one::<String>("file").unwrap().clone();
            match csv::parse_game_prices(&file_path){
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
                    if store == STEAM_STORE_ID {
                        steam_insert_sequence(&alias, &title, price, &http_client).await;
                    }
                    if store == GOG_STORE_ID {
                        gog_insert_sequence(&alias, &title, price, &http_client).await;
                    }
                    if store == MICROSOFT_STORE_ID {
                        microsoft_store_insert_sequence(&alias, &title, price, &http_client).await;
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
                println!("Caching started (this might take a while)...");
                steam::update_cached_games().await;
            }
            else if cmd.get_flag("check-prices") {
                let use_html = false;
                let prices_str = check_prices(use_html).await;
                if !prices_str.is_empty() {
                    println!("------------\nCHECK PRICES\n------------\n{}", prices_str);
                }
            }
            else if cmd.get_flag("email"){
                let use_html = true;
                let email_str = check_prices(use_html).await;
                println!("Email Contents:\n{}\n", email_str);
                if email_str.is_empty(){ println!("No game(s) on sale at price thresholds"); }
                else {
                    println!("Sending email...");
                    let to_address = &properties::get_recipient();
                    email::send_html_msg(to_address, "Check Out Which Games Are On Sale", &email_str);
                }
            }
            else { println!("No/incorrect command given. Use \'--help\' for assistance."); }
        }
    };
}