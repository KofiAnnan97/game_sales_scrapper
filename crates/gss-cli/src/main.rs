use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command, arg, command};
use properties::env_vars;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

// Internal libraries
use constants::cli::args::*;
use constants::operations::properties::{
    PROP_PROJECT_PATH, PROP_RECIPIENT_EMAIL, PROP_SMTP_EMAIL, PROP_SMTP_HOST, PROP_SMTP_PORT,
    PROP_SMTP_USERNAME, PROP_TEST_MODE, PROP_TEST_PATH,
};
use constants::operations::settings::ALLOW_ALIAS_REUSE_AFTER_CREATION;
use files::csv;
use gss_cli::{
    check_prices, gog_insert_sequence, microsoft_store_insert_sequence, print_test_flag,
    print_title, steam_insert_sequence, storefront_check,
};
use notifications::email;
use operations::{inputs, settings, thresholds};
use stores::{pc::steam, search::provider::SearchProvider};
use types::internal::data::SimpleGameThreshold;
use types::internal::store::{DEFAULT_CONCURRENCY_LIMIT, GameStore};

// Main function
#[tokio::main]
async fn main() {
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

    let search_provider = match SearchProvider::new(DEFAULT_CONCURRENCY_LIMIT) {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("The inital setup for search the game stores failed. {}", e);
            std::process::exit(1);
        }
    };

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
                                .action(ArgAction::SetTrue)
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
            match config_args.subcommand() {
                Some(("settings", settings_args)) => {
                    // Parameters
                    let enable_aliases = settings_args.value_source("enable_aliases");
                    let allow_alias_reuse = settings_args.value_source("allow_alias_reuse");

                    // Stores
                    let search_steam = settings_args.value_source("steam").unwrap();
                    let search_gog = settings_args.value_source("gog").unwrap();
                    let search_microsoft_store =
                        settings_args.value_source("microsoft_store").unwrap();
                    let search_all = settings_args.value_source("all_stores").unwrap();

                    let mut selected: Vec<GameStore> = Vec::new();
                    if search_steam == ValueSource::CommandLine {
                        selected.push(GameStore::STEAM);
                    }
                    if search_gog == ValueSource::CommandLine {
                        selected.push(GameStore::GOOD_OLD_GAMES);
                    }
                    if search_microsoft_store == ValueSource::CommandLine {
                        selected.push(GameStore::MICROSOFT_STORE_PC);
                    }
                    if search_all == ValueSource::CommandLine {
                        selected = settings::get_available_stores();
                    }
                    if !selected.is_empty() {
                        match settings::update_selected_stores(selected) {
                            Ok(_) => println!("Stores successfully updated."),
                            Err(e) => eprintln!("{}", e),
                        }
                    }

                    // If alias state is used
                    if let Some(val_src) = enable_aliases
                        && val_src == ValueSource::CommandLine
                    {
                        let alias_state: i32 =
                            *settings_args.get_one::<i32>("enable_aliases").unwrap();
                        if alias_state == 0 || alias_state == 1 {
                            match settings::update_alias_state(alias_state) {
                                Ok(_) => println!("Aliases enabled successfully updated."),
                                Err(e) => eprintln!("{}", e),
                            }
                        } else {
                            eprintln!(
                                "enable_aliases must be set to 0 or 1 not \'{}\'",
                                alias_state
                            );
                        }
                    }

                    // If allow alias reuse is used
                    if let Some(val_src) = allow_alias_reuse
                        && val_src == ValueSource::CommandLine
                    {
                        let alias_state: i32 =
                            *settings_args.get_one::<i32>("allow_alias_reuse").unwrap();
                        if alias_state == 0 || alias_state == 1 {
                            match settings::update_alias_reuse_state(alias_state) {
                                Ok(_) => println!("Alias reuse state changed"),
                                Err(e) => eprintln!("{}", e),
                            }
                        } else {
                            eprintln!(
                                "allow_alias_reuse must be set to 0 or 1 not \'{}\'",
                                alias_state
                            );
                        }
                    }
                }
                Some(("properties", properties_args)) => {
                    // Update properties from env
                    let from_env = properties_args.value_source(FROM_ENV).unwrap();
                    if from_env == ValueSource::CommandLine {
                        match properties::update_properties_from_env() {
                            Ok(_) => println!("Properties have been successfully updated"),
                            Err(e) => eprintln!("{}", e),
                        }
                    } else if from_env == ValueSource::DefaultValue
                        && let Some(test_mode) = properties_args.value_source("test_mode")
                        && test_mode == ValueSource::CommandLine
                    {
                        let test_state: i32 = *properties_args.get_one::<i32>("test_mode").unwrap();
                        let testing_enabled = test_state == 1;
                        match properties::set_test_mode(testing_enabled) {
                            Ok(_) => println!("Test mode set to {}", testing_enabled),
                            Err(e) => eprintln!("{}", e),
                        }
                    }

                    // Set SMTP variables
                    if let Some(val_src) = properties_args.value_source(SET_SMTP)
                        && val_src == ValueSource::CommandLine
                    {
                        let host = match inputs::get_string_input("Enter smtp host name:") {
                            Ok(host_str) => host_str,
                            Err(e) => {
                                eprintln!("Error getting host name: {}", e);
                                return;
                            }
                        };
                        let port_num = match inputs::get_u16_input("Enter SMTP port number:") {
                            Ok(port) => port,
                            Err(e) => {
                                eprintln!("Error getting port: {}", e);
                                return;
                            }
                        };
                        let email = match inputs::get_string_input("Enter smtp email:") {
                            Ok(email_str) => email_str,
                            Err(e) => {
                                eprintln!("Error getting port: {}", e);
                                return;
                            }
                        };
                        let user = match inputs::get_string_input("Enter smtp user:") {
                            Ok(user_str) => user_str,
                            Err(e) => {
                                eprintln!("Error getting port: {}", e);
                                return;
                            }
                        };
                        let pass = rpassword::prompt_password("SMTP Password: ").unwrap_or_else(|_| {
                            println!(
                                "An error occurred while setting the SMTP password. No changes were made."
                            );
                            String::new()
                        });
                        match properties::set_stmp_vars(host, port_num, email, user, pass) {
                            Ok(_) => println!("Stmp propertiews were successfully set."),
                            Err(e) => eprintln!("{}", e),
                        };
                    }

                    // Set recipient email
                    if let Some(recipient) = properties_args.get_one::<String>(SET_RECIPIENT) {
                        if let Ok(prev_recipient) = properties::get_recipient()
                            && !recipient.is_empty()
                            && prev_recipient != *recipient
                        {
                            match properties::set_recipient(recipient) {
                                Ok(_) => println!("Recipient was successfuly set."),
                                Err(e) => eprintln!("{}", e),
                            }
                        } else {
                            eprintln!("Recipient could not be set");
                        }
                    }

                    // Set Steam api key
                    let steam_key = properties_args.value_source(SET_API_KEY).unwrap();
                    if steam_key == ValueSource::CommandLine {
                        match rpassword::prompt_password("Provide Steam API Key: ") {
                            Ok(new_key) => {
                                if let Ok(prev_key) = properties::get_steam_api_key(false)
                                    && !new_key.is_empty()
                                    && prev_key != new_key
                                {
                                    match properties::set_steam_api_key(new_key) {
                                        Ok(_) => println!("Steam API Key was successfully set"),
                                        Err(e) => eprintln!("{}", e),
                                    }
                                } else {
                                    eprintln!("Steam API key could not be set");
                                }
                            }
                            Err(_) => {
                                eprintln!(
                                    "An error occurred while setting the Steam API Key. No changes were made."
                                )
                            }
                        }
                    }

                    // Set project path
                    if let Some(path) = properties_args.get_one::<String>(SET_PROJECT_PATH) {
                        if let Ok(prev_path) = properties::get_project_path()
                            && !path.is_empty()
                            && prev_path != *path
                        {
                            match properties::set_project_path(path) {
                                Ok(_) => println!("Project path was successfully set."),
                                Err(e) => eprintln!("{}", e),
                            }
                        } else {
                            eprintln!("Project path could not be set");
                        }
                    }

                    // Set test path
                    if let Some(path) = properties_args.get_one::<String>(SET_TEST_PATH) {
                        if let Ok(prev_path) = properties::get_test_path()
                            && !path.is_empty()
                            && prev_path != *path
                        {
                            match properties::set_test_path(path) {
                                Ok(_) => println!("Test path was successfully set."),
                                Err(e) => eprintln!("{}", e),
                            }
                        } else {
                            eprintln!("Test path could not be set");
                        }
                    }

                    // Reveal secrets (gated by last 12 characters of decrypt key)
                    let reveal_secrets = properties_args.value_source(REVEAL_SECRETS).unwrap();
                    let mut confirm_str = String::new();
                    if reveal_secrets == ValueSource::CommandLine {
                        confirm_str = match inputs::get_string_input(
                            "Are you sure? This will print out sensitive data (Y/N):",
                        ) {
                            Ok(confirm) => confirm,
                            Err(e) => {
                                eprintln!("{}", e);
                                return;
                            }
                        };
                    }

                    let mut hidden = true;
                    if confirm_str.trim().eq_ignore_ascii_case("Y")
                        || confirm_str.trim().eq_ignore_ascii_case("Yes")
                    {
                        hidden = match rpassword::prompt_password(
                            "Provide the last 12 characters of the decryption key: ",
                        ) {
                            Ok(last_chars) => match properties::get_project_path() {
                                Ok(project_path) => match env_vars::get_decrypt_key(project_path) {
                                    Ok(key) => {
                                        if key.len() > 12
                                            && let Some(key_end) =
                                                key.as_str().get(key.len() - 12..)
                                        {
                                            !(last_chars.len() == 12
                                                && last_chars.as_str().eq(key_end))
                                        } else {
                                            false
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to get decrypt key. {}", e);
                                        true
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Failed to get project path. {}", e);
                                    true
                                }
                            },
                            Err(_) => {
                                eprintln!("Failed to parse provided decyption key values.");
                                true
                            }
                        }
                    }

                    // List properties
                    let list_properties = properties_args.value_source(LIST_PROPERTIES).unwrap();
                    if list_properties == ValueSource::CommandLine {
                        match properties::load_properties() {
                            Ok(properties) => match serde_json::to_string(&properties) {
                                Ok(properties_str) => {
                                    let lookup: HashMap<String, Value> =
                                        serde_json::from_str(&properties_str).unwrap();
                                    print_title("PROPERTIES");
                                    let test_mode = lookup.get(PROP_TEST_MODE).unwrap_or_default();
                                    match test_mode.as_i64() {
                                        Some(1) => println!(
                                            "Test Path: {}",
                                            lookup.get(PROP_TEST_PATH).unwrap_or_default()
                                        ),
                                        Some(0) => println!(
                                            "Project Path: {}",
                                            lookup.get(PROP_PROJECT_PATH).unwrap_or_default()
                                        ),
                                        None => (),
                                        _ => {
                                            println!(
                                                "Cannot show path given test_mode '{}'",
                                                test_mode
                                            )
                                        }
                                    }
                                    println!(
                                        "Recipient Email: {}",
                                        lookup.get(PROP_RECIPIENT_EMAIL).unwrap_or_default()
                                    );
                                    println!(
                                        "Steam API Key: {}",
                                        properties::get_steam_api_key(hidden).unwrap_or(
                                            String::from("<Steam key could not be displayed>")
                                        )
                                    );
                                    println!(
                                        "SMTP Host: {}",
                                        lookup.get(PROP_SMTP_HOST).unwrap_or_default()
                                    );
                                    println!(
                                        "SMTP Port: {}",
                                        lookup.get(PROP_SMTP_PORT).unwrap_or_default()
                                    );
                                    println!(
                                        "SMTP Email: {}",
                                        lookup.get(PROP_SMTP_EMAIL).unwrap_or_default()
                                    );
                                    println!(
                                        "SMTP User: {}",
                                        lookup.get(PROP_SMTP_USERNAME).unwrap_or_default()
                                    );
                                    println!(
                                        "SMTP Password: {}",
                                        properties::get_smtp_pwd(hidden).unwrap_or(String::from(
                                            "<SMTP password could not be displayed>"
                                        ))
                                    );
                                    println!("Test Mode: {}", test_mode);
                                }
                                Err(e) => println!("{}", e),
                            },
                            Err(e) => eprintln!("Failed to list properties.\n{}", e),
                        }
                    }
                }
                _ => (),
            }
        }
        Some(("add", add_args)) => match storefront_check() {
            Ok(selected_stores) => {
                print_test_flag();
                let mut has_errors = false;
                let input_alias = if let Some(alias) = add_args.get_one::<String>("alias") {
                    alias.trim().to_string()
                } else {
                    match settings::get_alias_state() {
                        Ok(true) => match thresholds::set_game_alias() {
                            Ok(alias) => alias,
                            Err(e) => {
                                eprintln!("{}", e);
                                has_errors = true;
                                String::new()
                            }
                        },
                        Ok(false) => String::new(),
                        Err(e) => {
                            eprintln!("{}", e);
                            has_errors = true;
                            String::new()
                        }
                    }
                };
                let title = add_args.get_one::<String>("title").unwrap().clone();
                let price = *add_args.get_one::<f64>("price").unwrap();
                let (alias, alias_available) =
                    match thresholds::get_alias_if_available(&input_alias) {
                        Ok(result) => {
                            if !result.1 {
                                eprintln!("Game title could not be added.");
                                has_errors = true;
                            }
                            result
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            has_errors = true;
                            (String::new(), false)
                        }
                    };
                if !has_errors {
                    for store in selected_stores.iter() {
                        if store == &GameStore::STEAM {
                            match steam_insert_sequence(&alias, &title, price, &search_provider)
                                .await
                            {
                                Ok(title) => {
                                    println!("Successfully added Steam game: \"{}\".", title)
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            }
                        }
                        if store == &GameStore::GOOD_OLD_GAMES {
                            match gog_insert_sequence(&alias, &title, price, &search_provider).await
                            {
                                Ok(title) => {
                                    println!("Successfully added GOG game: \"{}\".", title)
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            }
                        }
                        if store == &GameStore::MICROSOFT_STORE_PC {
                            match microsoft_store_insert_sequence(
                                &alias,
                                &title,
                                price,
                                &search_provider,
                            )
                            .await
                            {
                                Ok(title) => println!(
                                    "Successfully added Microsoft Store game: \"{}\".",
                                    title
                                ),
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            };
                        }
                    }
                }
                if !alias_available {
                    eprintln!(
                        "Alias '{}' is already in use. If needed, set '{}' to 1.",
                        input_alias, ALLOW_ALIAS_REUSE_AFTER_CREATION
                    );
                }
            }
            Err(e) => eprintln!("{}", e),
        },
        Some(("bulk-insert", bulk_args)) => match storefront_check() {
            Ok(selected_stores) => {
                print_test_flag();
                let mut game_list: Vec<SimpleGameThreshold> = Vec::new();
                let file_path = bulk_args.get_one::<String>("file").unwrap().clone();
                match csv::parse_game_prices_from_path(&file_path) {
                    Ok(gl) => game_list = gl,
                    Err(e) => eprintln!("Could not parse file: {}\n{}", file_path, e),
                }
                let mut has_errors = false;
                for game in game_list.iter() {
                    println!("INSERT GAME -> \"{}\"", game.name);
                    let title = &game.name;
                    let input_alias = match thresholds::set_game_alias() {
                        Ok(alias) => alias.trim().to_string(),
                        Err(e) => {
                            eprintln!("{}", e);
                            has_errors = true;
                            continue;
                        }
                    };
                    let price: f64 = game.price;
                    let (alias, _alias_available) =
                        match thresholds::get_alias_if_available(&input_alias) {
                            Ok(result) => {
                                if !result.1 {
                                    eprintln!("Game title could not be added.");
                                    has_errors = true;
                                }
                                result
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                has_errors = true;
                                continue;
                            }
                        };
                    for store in selected_stores.iter() {
                        if store == &GameStore::STEAM {
                            match steam_insert_sequence(&alias, title, price, &search_provider)
                                .await
                            {
                                Ok(title) => {
                                    println!("Successfully added Steam game: \"{}\".", title)
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    has_errors = true;
                                }
                            }
                        }
                        if store == &GameStore::GOOD_OLD_GAMES {
                            match gog_insert_sequence(&alias, title, price, &search_provider).await
                            {
                                Ok(title) => {
                                    println!("Successfully added GOG game: \"{}\".", title)
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    has_errors = true;
                                }
                            }
                        }
                        if store == &GameStore::MICROSOFT_STORE_PC {
                            match microsoft_store_insert_sequence(
                                &alias,
                                title,
                                price,
                                &search_provider,
                            )
                            .await
                            {
                                Ok(title) => {
                                    println!(
                                        "Successfully added Microsoft Store game: \"{}\".",
                                        title
                                    )
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    has_errors = true;
                                }
                            };
                        }
                    }
                }
                if !has_errors && !game_list.is_empty() {
                    println!("Bulk insert completed successfully.");
                }
            }
            Err(e) => eprintln!("{}", e),
        },
        Some(("update", update_args)) => match update_args.subcommand() {
            Some(("price", price_args)) => {
                print_test_flag();
                let title = price_args.get_one::<String>("title").unwrap().clone();
                let price = *price_args.get_one::<f64>("price").unwrap();
                match thresholds::update_price_fuzzy(&title, price) {
                    Ok(_) => println!(
                        "Successfully updated game '{}' with price {}.",
                        title, price
                    ),
                    Err(e) => eprintln!("Could not update game: {}.", e),
                };
            }
            Some(("alias", alias_args)) => {
                print_test_flag();
                let title = alias_args.get_one::<String>("title").unwrap().clone();
                let alias = alias_args.get_one::<String>("alias").unwrap().clone();
                match thresholds::update_threshold_alias_fuzzy(title, &alias) {
                    Ok(_) => println!(
                        "Successfully updated alias to {} for the game threshold(s)",
                        alias
                    ),
                    Err(e) => eprintln!("{}", e),
                };
            }
            _ => (),
        },
        Some(("remove", remove_args)) => {
            print_test_flag();
            let title = remove_args.get_one::<String>("title").unwrap().clone();
            match thresholds::remove_fuzzy(&title) {
                Ok(_) => println!("Successfully removed game '{}'.", title),
                Err(e) => eprintln!("Could not remove game: {}.", e),
            };
        }
        _ => {
            print_test_flag();
            if cmd.get_flag(LIST_THRESHOLDS) {
                thresholds::list_games();
            } else if cmd.get_flag(LIST_SELECTED_STORES) {
                match settings::list_selected_stores() {
                    Ok(_) => {}
                    Err(e) => eprintln!("{}", e),
                };
            } else if cmd.get_flag(UPDATE_CACHE) {
                println!("Caching started (this might take a while)...");
                println!("Retrieving entries...");
                match steam::update_cached_games(&search_provider.client).await {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Caching could not be completed due to {:?}", e),
                }
            } else if cmd.get_flag(CHECK_PRICES) {
                let use_html = false;
                match check_prices(use_html, &Arc::new(search_provider)).await {
                    Ok(prices_str) => {
                        if !prices_str.is_empty() {
                            print_title("CHECK PRICES");
                            println!("{}", prices_str);
                        }
                    }
                    Err(e) => eprintln!("{}", e),
                };
            } else if cmd.get_flag(SEND_EMAIL) {
                let use_html = true;
                match email::params_check() {
                    Ok(_) => match check_prices(use_html, &Arc::new(search_provider)).await {
                        Ok(sales_str) => {
                            let html_body = email::create_html_body(&sales_str);
                            println!("Email Contents:\n{}", html_body);
                            if sales_str.is_empty() {
                                println!("No game(s) on sale at price thresholds");
                            } else {
                                println!("Sending email...");
                                if let Ok(to_address) = &properties::get_recipient() {
                                    match email::send_html_msg(
                                        to_address,
                                        "Check Out Which Games Are On Sale",
                                        &html_body,
                                    ) {
                                        Ok(success) => println!("{}", success),
                                        Err(err) => eprintln!("{}", err),
                                    };
                                } else {
                                    eprintln!("Could not send an email to recipient.");
                                }
                            }
                        }
                        Err(e) => eprintln!("{}", e),
                    },
                    Err(err) => eprintln!("{}", err),
                }
            } else {
                println!("No/incorrect command given. Use \'--help\' for assistance.");
            }
        }
    };
}
