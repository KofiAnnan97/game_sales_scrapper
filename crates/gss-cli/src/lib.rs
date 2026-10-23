use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;

use alerting::email;
use constants::stores::gog::VERSION as GOG_VERSION;
use file_ops::{settings, thresholds};
use stores::pc::{gog, microsoft_store, steam};
use stores::search::provider::{SearchError, SearchProvider};
use types::internal::{
    data::SaleInfo,
    store::{BULK_SEARCH_LIMIT, GameStore, StorePriceResult},
};
use types::response::gog::GameInfo as GOGGameInfo;
use types::response::microsoft_store::ProductInfo;

pub fn storefront_check() -> Vec<GameStore> {
    let selected_stores = settings::get_selected_stores();
    if selected_stores.is_empty() {
        panic!(
            "Please configure which stores to query. Run \'game_sales_scrapper config --help\' for more info."
        );
    }
    selected_stores
}

pub fn print_test_flag() {
    if properties::is_testing_enabled() {
        println!("------------------------\n* TEST MODE IS ENABLED *\n------------------------");
    }
}

pub fn get_simple_prices_str(store_name: GameStore, sales: Vec<SaleInfo>) -> String {
    let mut prices_str = String::new();
    for game in sales.iter() {
        prices_str.push_str(&format!(
            "\n\t- {} : {} -> {} ({}% off)",
            game.title, game.original_price, game.current_price, game.discount_percentage
        ));
    }
    if !prices_str.is_empty() {
        let header_str = format!("\n{} game(s) that met your desired price:", store_name);
        prices_str = header_str + &prices_str;
    }
    prices_str
}

pub async fn check_prices(use_html: bool, provider: &Arc<SearchProvider>) -> String {
    let thresholds = thresholds::load_thresholds().unwrap_or_else(|_e| Vec::new());

    let mut tasks: Vec<tokio::task::JoinHandle<(GameStore, StorePriceResult)>> = Vec::new();

    let mut steam_app_ids: Vec<u32> = Vec::new();
    let mut desired_prices: HashMap<String, f64> = HashMap::new();
    let mut steam_iter = thresholds.iter().peekable();

    while let Some(elem) = steam_iter.next() {
        if elem.steam_id != 0 {
            steam_app_ids.push(elem.steam_id);
            desired_prices
                .entry(elem.steam_id.to_string())
                .or_insert(elem.desired_price);
        }
        if steam_app_ids.len() >= BULK_SEARCH_LIMIT || steam_iter.peek().is_none() {
            let ids = steam_app_ids.clone();
            let prices = desired_prices.clone();
            let steam_provider = Arc::clone(provider);

            let task = tokio::spawn(async move {
                let result = steam_provider
                    .result_search(|| async {
                        steam::get_bulk_filtered_price_details(
                            &ids,
                            &prices,
                            &steam_provider.client,
                        )
                        .await
                    })
                    .await;
                match result {
                    Ok(sales) => (GameStore::STEAM, StorePriceResult::Success(sales)),
                    Err(SearchError::SemaphoreError(e)) => {
                        (GameStore::STEAM, StorePriceResult::Error(e.to_string()))
                    }
                    Err(SearchError::StoreError(e)) => {
                        (GameStore::STEAM, StorePriceResult::Error(e.to_string()))
                    }
                }
            });
            tasks.push(task);

            steam_app_ids.clear();
            desired_prices.clear();
        }
    }

    for elem in thresholds.iter() {
        if elem.gog_id != 0 && GOG_VERSION == 2 {
            let title = elem.title.clone();
            let desired_price = elem.desired_price;
            let gog_provider = Arc::clone(provider);

            let task = tokio::spawn(async move {
                let result = gog_provider
                    .option_search(|| async {
                        gog::get_price_details_v2(&title, &gog_provider.client).await
                    })
                    .await;

                match result {
                    Some(info) if desired_price >= info.current_price => (
                        GameStore::GOOD_OLD_GAMES,
                        StorePriceResult::Success(vec![info]),
                    ),
                    _ => (GameStore::GOOD_OLD_GAMES, StorePriceResult::Success(vec![])),
                }
            });
            tasks.push(task);
        }

        if !elem.microsoft_store_id.is_empty() {
            let store_id = elem.microsoft_store_id.clone();
            let desired_price = elem.desired_price;
            let ms_provider = Arc::clone(provider);

            let task = tokio::spawn(async move {
                let result = ms_provider
                    .option_search(|| async {
                        microsoft_store::get_price_details(&store_id, &ms_provider.client).await
                    })
                    .await;

                match result {
                    Some(info) if desired_price >= info.current_price => (
                        GameStore::MICROSOFT_STORE_PC,
                        StorePriceResult::Success(vec![info]),
                    ),
                    _ => (
                        GameStore::MICROSOFT_STORE_PC,
                        StorePriceResult::Success(vec![]),
                    ),
                }
            });
            tasks.push(task);
        }
    }

    let mut steam_sales: Vec<SaleInfo> = Vec::new();
    let mut gog_sales: Vec<SaleInfo> = Vec::new();
    let mut microsoft_store_sales: Vec<SaleInfo> = Vec::new();

    let results = futures::future::join_all(tasks).await;
    for (store, price_result) in results.into_iter().flatten() {
        match price_result {
            StorePriceResult::Success(mut sales) => match store {
                GameStore::STEAM => steam_sales.append(&mut sales),
                GameStore::GOOD_OLD_GAMES => gog_sales.append(&mut sales),
                GameStore::MICROSOFT_STORE_PC => microsoft_store_sales.append(&mut sales),
            },
            StorePriceResult::Error(e) => {
                eprintln!("Error from {}: {}", store, e);
            }
        }
    }

    // Format output
    let mut output = String::new();
    if !steam_sales.is_empty() {
        if use_html {
            output.push_str(&email::create_store_cards(GameStore::STEAM, steam_sales));
        } else {
            output.push_str(&get_simple_prices_str(GameStore::STEAM, steam_sales));
        }
    }
    if !gog_sales.is_empty() {
        if use_html {
            output.push_str(&email::create_store_cards(
                GameStore::GOOD_OLD_GAMES,
                gog_sales,
            ));
        } else {
            output.push_str(&get_simple_prices_str(GameStore::GOOD_OLD_GAMES, gog_sales));
        }
    }
    if !microsoft_store_sales.is_empty() {
        if use_html {
            output.push_str(&email::create_store_cards(
                GameStore::MICROSOFT_STORE_PC,
                microsoft_store_sales,
            ));
        } else {
            output.push_str(&get_simple_prices_str(
                GameStore::MICROSOFT_STORE_PC,
                microsoft_store_sales,
            ));
        }
    }
    output
}

pub async fn steam_insert_sequence(
    alias: &str,
    title: &str,
    price: f64,
    provider: &SearchProvider,
) {
    match steam::check_game(title, &provider.client).await {
        Some(data) => {
            thresholds::add_steam_game(alias.to_string(), data, price, &provider.client).await
        }
        None => {
            if let Some(t) = steam::search_game(title, &provider.client).await {
                match steam::check_game(&t, &provider.client).await {
                    Some(data) => {
                        thresholds::add_steam_game(alias.to_string(), data, price, &provider.client)
                            .await
                    }
                    None => eprintln!("Something went wrong"),
                }
            }
        }
    }
}

pub async fn gog_insert_sequence(alias: &str, title: &str, price: f64, provider: &SearchProvider) {
    let mut search_list: Vec<GOGGameInfo> = Vec::new();
    match gog::search_game_by_title_v2(title, &provider.client).await {
        Ok(data) => search_list = data,
        Err(e) => println!("Search GOG Game Error: {}", e),
    }
    if !search_list.is_empty() {
        println!("GOG search results:");
        for (i, game) in search_list.iter().enumerate() {
            let price = match &game.price {
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
        } else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < search_list.len() {
                        //title = &search_list[idx].title;
                        let game = &search_list[idx];
                        thresholds::add_gog_game(alias.to_string(), game, price);
                    } else if idx >= search_list.len() {
                        eprintln!("Integer \"{}\" is invalid. Request terminated.", idx);
                    }
                }
                Err(e) => println!("Invalid input: {}\nError: {}", input, e),
            }
        }
    } else {
        println!("Could not find a game title matching \"{}\" on GOG.", title);
    }
}

pub async fn microsoft_store_insert_sequence(
    alias: &str,
    title: &str,
    price: f64,
    provider: &SearchProvider,
) {
    let mut search_list: Vec<ProductInfo> = Vec::new();
    match microsoft_store::search_game_by_title(title, &provider.client).await {
        Ok(data) => search_list = data,
        Err(e) => println!("Search Microsoft Store Error: {}", e),
    }
    if !search_list.is_empty() {
        println!("Microsoft Store search results:");
        for (i, game) in search_list.iter().enumerate() {
            println!(
                "  [{}] {} - ${}",
                i,
                game.title,
                game.price_info.msrp.unwrap_or_default()
            );
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
        } else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < search_list.len() {
                        let game = &search_list[idx];
                        thresholds::add_microsoft_store_game(alias.to_string(), game, price);
                    } else if idx >= search_list.len() {
                        eprintln!("Integer \"{}\" is invalid. Request terminated.", idx);
                    }
                }
                Err(e) => println!("Invalid input: {}\nError: {}", input, e),
            }
        }
    } else {
        println!(
            "Could not find a game title matching \"{}\" on the Microsoft Store.",
            title
        );
    }
}
