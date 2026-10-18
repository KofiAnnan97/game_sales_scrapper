use regex::Regex;
use serde::Deserialize;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
// use mockall::automock;
use async_trait::async_trait;
use tokio::sync::Semaphore;
use tokio::time::Duration;

use crate::search::fuzzy;
use constants::operations::properties::PROP_STEAM_API_KEY;
use constants::operations::thresholds::{
    LEVENSTEIN_DIST_PERCENTAGE, SMITH_WATERMAN_DIST_PERCENTAGE,
};
use constants::stores::steam::*;
use errors::api::ApiError;
use files::general;
use properties;
use types::internal::data::SaleInfo;
use types::internal::store::DEFAULT_CONCURRENCY_LIMIT;
use types::response::steam::{App, AppDetails, AppPrices, PriceOverview};

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum SearchPattern {
    Exact,
    FuzzyLevenshtein,
    FuzzySmithWatrerman,
}

// #[automock]
#[async_trait]
pub trait SteamApi {
    async fn search_game(&self, keyphrase: &str) -> Option<String>;
    async fn search_by_keyphrase(&self, keyphrase: &str) -> Result<Vec<String>, ApiError>;
    async fn check_game(&self, name: &str) -> Option<App>;
    async fn get_price(&self, app_id: u32) -> Result<PriceOverview, ApiError>;
    async fn get_price_details(&self, app_id: u32) -> Result<SaleInfo, ApiError>;
    async fn get_bulk_filtered_price_details(
        &self,
        app_ids: &[u32],
        desired_prices: &HashMap<String, f64>,
    ) -> Result<Vec<SaleInfo>, ApiError>;
}

pub struct SteamClient<'a> {
    http_client: &'a reqwest::Client,
}

impl<'a> SteamClient<'a> {
    pub fn new(http_client: &'a reqwest::Client) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl<'a> SteamApi for SteamClient<'a> {
    async fn search_game(&self, keyphrase: &str) -> Option<String> {
        match self.search_by_keyphrase(keyphrase).await {
            Ok(search_list) => {
                if !search_list.is_empty() {
                    println!("Steam search results:");
                    for (idx, game_title) in search_list.iter().enumerate() {
                        println!("  [{}] {}", idx, game_title);
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
                                    let title = search_list[idx].clone();
                                    return Some(title);
                                } else if idx >= search_list.len() {
                                    eprintln!(
                                        "Integer \"{}\" is invalid. Request terminated.",
                                        idx
                                    );
                                }
                            }
                            Err(e) => println!("Invalid input: {}\nError: {}", input, e),
                        }
                    }
                } else {
                    println!(
                        "Could not find a game title matching \"{}\" on Steam.",
                        keyphrase
                    );
                }
            }
            Err(e) => println!("Error: {}", e),
        }
        None
    }

    async fn search_by_keyphrase(&self, keyphrase: &str) -> Result<Vec<String>, ApiError> {
        let mut games_list: Vec<App> = load_cached_games().unwrap_or_default();
        if games_list.is_empty() {
            let _ = update_cached_games(self.http_client).await;
            games_list = load_cached_games().unwrap_or_default();
        }
        let mut search_list: Vec<String> = Vec::new();
        let search_type = SearchPattern::FuzzySmithWatrerman;

        let semaphore = Arc::new(Semaphore::new(DEFAULT_CONCURRENCY_LIMIT));
        let cache_length = &games_list.len();
        let cache_chunk = cache_length / DEFAULT_CONCURRENCY_LIMIT;
        if cache_chunk > 0 {
            let game_chunks = games_list.chunks(cache_chunk);
            match search_type {
                SearchPattern::Exact => {
                    let mut tasks: Vec<tokio::task::JoinHandle<Vec<String>>> = Vec::new();
                    for chunk in game_chunks {
                        let sem = Arc::clone(&semaphore);
                        let chunk = chunk.to_vec();
                        let keyphrase_owned = keyphrase.to_string();
                        let task = tokio::spawn(async move {
                            let _permit = sem.acquire().await.ok();
                            let mut temp_list: Vec<String> = Vec::new();
                            let keyphrase_ignore_case = format!("(?i){}", keyphrase_owned);
                            let re = Regex::new(&keyphrase_ignore_case).unwrap();
                            for game in chunk.iter() {
                                let caps = re.captures(&game.name);
                                if !caps.is_none() {
                                    temp_list.push(game.name.clone());
                                }
                            }
                            temp_list
                        });
                        tasks.push(task);
                    }
                    let results = futures::future::join_all(tasks).await;
                    for game_names in results.into_iter().take(SEARCH_SIZE_LIMIT).flatten() {
                        search_list.extend(game_names);
                    }
                }
                SearchPattern::FuzzyLevenshtein | SearchPattern::FuzzySmithWatrerman => {
                    let mut tasks: Vec<tokio::task::JoinHandle<HashMap<String, f32>>> = Vec::new();
                    for chunk in game_chunks {
                        let sem = Arc::clone(&semaphore);
                        let chunk = chunk.to_vec();
                        let keyphrase_owned = keyphrase.to_string();
                        let task = tokio::spawn(async move {
                            let _permit = sem.acquire().await.ok();
                            let mut fuzzy_thresholds: HashMap<String, f32> = HashMap::new();
                            for app in chunk {
                                if search_type == SearchPattern::FuzzyLevenshtein {
                                    let dist =
                                        fuzzy::levenshtein_distance(&keyphrase_owned, &app.name);
                                    if 1.0 - (dist / keyphrase_owned.len() as f32)
                                        >= LEVENSTEIN_DIST_PERCENTAGE
                                    {
                                        if fuzzy_thresholds.len() > SEARCH_SIZE_LIMIT / 2 {
                                            break;
                                        }
                                        fuzzy_thresholds.insert(app.name.clone(), dist);
                                    }
                                } else if search_type == SearchPattern::FuzzySmithWatrerman {
                                    let score = fuzzy::smith_waterman(&keyphrase_owned, &app.name);
                                    let percentage = score / (2 * keyphrase_owned.len()) as f32;
                                    if percentage > SMITH_WATERMAN_DIST_PERCENTAGE {
                                        if fuzzy_thresholds.len() > SEARCH_SIZE_LIMIT / 2 {
                                            break;
                                        }
                                        fuzzy_thresholds.insert(app.name.clone(), percentage);
                                    }
                                }
                            }
                            fuzzy_thresholds
                        });
                        tasks.push(task);
                    }
                    let mut combined_fuzzy_scores: HashMap<String, f32> = HashMap::new();
                    let results = futures::future::join_all(tasks).await;
                    for fuzzy_scores in results.into_iter().flatten() {
                        combined_fuzzy_scores.extend(fuzzy_scores);
                    }
                    search_list = combined_fuzzy_scores.keys().cloned().collect();
                    search_list.sort_by(|a, b| {
                        combined_fuzzy_scores[a].total_cmp(&combined_fuzzy_scores[b])
                    });
                    combined_fuzzy_scores.clear();

                    if !keyphrase.is_empty() && !search_list.is_empty() {
                        prioritize_fuzzy_results(&mut search_list, keyphrase);
                    }
                }
            };
        }
        Ok(search_list)
    }

    async fn check_game(&self, name: &str) -> Option<App> {
        let mut games_list: Vec<App> = load_cached_games().unwrap_or_default();
        if games_list.is_empty() {
            let _ = update_cached_games(self.http_client).await;
            games_list = load_cached_games().unwrap_or_default();
        }
        for elem in games_list.iter() {
            if name == elem.name {
                return Some(App {
                    name: name.to_owned(),
                    app_id: elem.app_id,
                    last_modified: elem.last_modified,
                    price_change_number: elem.price_change_number,
                });
            }
        }
        None
    }

    async fn get_price(&self, app_id: u32) -> Result<PriceOverview, ApiError> {
        match get_game_data(app_id, true, self.http_client).await {
            Ok(app_details) => {
                let success = app_details.success;
                let data = app_details.app_data;
                if success && let Some(game_data) = data {
                    if let Some(price_overview) = game_data.price_overview {
                        Ok(PriceOverview {
                            currency: price_overview.currency,
                            discount_percent: price_overview.discount_percent,
                            initial: price_overview.initial / 100.0,
                            final_price: price_overview.final_price / 100.0,
                        })
                    } else {
                        Err(ApiError::Message(format!(
                            "Could not retrieve pricing information for game with id {}.",
                            app_id
                        )))
                    }
                } else {
                    Err(ApiError::Message(format!(
                        "No data available for game with id {}.",
                        app_id
                    )))
                }
            }
            Err(e) => Err(ApiError::Message(format!(
                "An issue was found for Steam game with id: {}, {}",
                app_id, e
            ))),
        }
    }

    async fn get_price_details(&self, app_id: u32) -> Result<SaleInfo, ApiError> {
        match get_game_data(app_id, true, self.http_client).await {
            Ok(app_details) => {
                let success = app_details.success;
                if success {
                    if let Some(data) = app_details.app_data {
                        let price_overview = data.price_overview.unwrap();
                        Ok(SaleInfo {
                            icon_link: data.header_image,
                            title: data.name,
                            original_price: price_overview.initial / 100.0,
                            current_price: price_overview.final_price / 100.0,
                            discount_percentage: format!("{}", price_overview.discount_percent),
                            store_page_link: format!("{}{}", STORE_PAGE_URL, app_id),
                        })
                    } else {
                        Err(ApiError::Message(format!(
                            "Game with id {} was present but info was not available",
                            app_id
                        )))
                    }
                } else {
                    Err(ApiError::Message(format!(
                        "No data available for game with id {}.",
                        app_id
                    )))
                }
            }
            Err(e) => Err(ApiError::Message(format!(
                "An issue was found for Steam game with id: {}, {}",
                app_id, e
            ))),
        }
    }

    async fn get_bulk_filtered_price_details(
        &self,
        app_ids: &[u32],
        desired_prices: &HashMap<String, f64>,
    ) -> Result<Vec<SaleInfo>, ApiError> {
        let mut sales_info: Vec<SaleInfo> = Vec::new();
        match get_bulk_prices(app_ids, self.http_client).await {
            Ok(app_prices) => {
                for (app_id, response) in app_prices.prices {
                    let id = app_id.parse::<u32>().unwrap_or_default();
                    if let Some(po) = response.data.price_overview
                        && let Some(price) = desired_prices.get(&app_id)
                        && *price >= po.final_price / 100.0
                        && let Ok(app_details) = get_game_data(id, false, self.http_client).await
                        && let Some(data) = app_details.app_data
                    {
                        sales_info.push(SaleInfo {
                            icon_link: data.header_image,
                            title: data.name,
                            original_price: po.initial / 100.0,
                            current_price: po.final_price / 100.0,
                            discount_percentage: format!("{}", po.discount_percent),
                            store_page_link: format!("{}{}", STORE_PAGE_URL, app_id),
                        });
                    }
                }
            }
            Err(e) => return Err(e),
        }
        Ok(sales_info)
    }
}

// Caching Functions
fn get_cache_path() -> String {
    let path_buf: PathBuf = [properties::get_data_path(), CACHE_FILENAME.to_string()]
        .iter()
        .collect();
    let cache_file_path = path_buf.display().to_string();
    general::get_path(&cache_file_path)
}

fn get_valid_apps(raw_app_list: &Value) -> Vec<App> {
    let mut valid_apps: Vec<App> = Vec::new();
    if let Some(games) = raw_app_list.as_array() {
        for game in games.iter() {
            match serde_json::from_value::<App>(game.clone()) {
                Ok(app) => valid_apps.push(app),
                Err(_) => continue,
            }
        }
    }
    valid_apps
}

fn load_cached_games() -> serde_json::Result<Vec<App>> {
    let filepath = get_cache_path();
    let data = read_to_string(filepath).unwrap();
    let raw_cached_games: Value = serde_json::from_str(&data)?;
    Ok(get_valid_apps(&raw_cached_games))
}

fn get_last_appid(cached_games: &[App]) -> u32 {
    if !cached_games.is_empty()
        && let Some(app) = cached_games.last()
    {
        app.app_id
    } else {
        0
    }
}

// Adds/updates entries in cache then empties the list contains potentially new games
fn add_entries_to_cache(new_games: &mut Vec<App>, cached_games: &mut Vec<App>) -> bool {
    let mut changed = false;
    // let mut game_idx = 0;
    for ng in new_games.iter() {
        let mut unique = true;
        for i in 0..cached_games.len() {
            let cached_game = cached_games.get_mut(i).unwrap();
            if ng.app_id == cached_game.app_id {
                if ng.last_modified > cached_game.last_modified {
                    cached_game.name = ng.name.to_string();
                    cached_game.last_modified = ng.last_modified;
                    cached_game.price_change_number = ng.price_change_number;
                    changed = true;
                }
                // game_idx = i;
                unique = false;
                break;
            } else if ng.app_id < cached_game.app_id {
                // game_idx = i;
                break;
            }
        }
        if unique && !ng.name.is_empty() {
            cached_games.push(App {
                name: ng.name.clone(),
                app_id: ng.app_id,
                last_modified: ng.last_modified,
                price_change_number: ng.price_change_number,
            });
            changed = true;
        }
    }
    new_games.clear();
    changed
}

// Updated to return a boolean and propagate error
pub async fn update_cached_games(client: &reqwest::Client) -> Result<String, ApiError> {
    let mut games_list: Vec<App> = load_cached_games().unwrap_or_default();
    let last_appid = get_last_appid(&games_list);
    let mut temp: Vec<App> = get_games(client, NUM_OF_RESULTS, last_appid).await?;
    let mut cache_changed = add_entries_to_cache(&mut temp, &mut games_list);
    let sliding_last_appid = properties::get_sliding_steam_appid();
    if temp.is_empty()
        || (sliding_last_appid < last_appid && games_list.len() > SLIDING_UPDATE_START_SIZE)
    {
        temp = get_games(client, NUM_OF_RESULTS, sliding_last_appid).await?;
        if let Some(last_app) = temp.last() {
            properties::set_sliding_steam_appid(last_app.app_id);
        } else {
            properties::set_sliding_steam_appid(0);
        }
        cache_changed |= add_entries_to_cache(&mut temp, &mut games_list);
    } else {
        properties::set_sliding_steam_appid(0);
    }
    if !cache_changed {
        return Ok("Cache already up to date".into());
    }
    games_list.sort_by_key(|a| a.app_id);
    let data_str = serde_json::to_string_pretty(&games_list)?;
    general::write_to_file(get_cache_path(), data_str);
    Ok("Cache update complete".into())
}

// API Functions
// TODO: If request fails don't prove url with steam key
async fn get_games(
    client: &reqwest::Client,
    max_results: u32,
    last_appid: u32,
) -> Result<Vec<App>, ApiError> {
    let steam_api_key = properties::get_steam_api_key(false);
    if steam_api_key.is_empty() {
        return Err(ApiError::Message(format!(
            "Missing '{}' property.",
            PROP_STEAM_API_KEY
        )));
    }
    let query_string = [
        ("key", steam_api_key.as_str()),
        ("max_results", &max_results.to_string()),
        ("last_appid", &last_appid.to_string()),
        ("format", "json"),
    ];
    let url = format!("{}{}", API_BASE_URL, APP_LIST_ENDPOINT);
    let resp = match client
        .get(url)
        .timeout(Duration::from_secs(GAME_LIST_TIMEOUT_IN_SECS))
        .query(&query_string)
        .send()
        .await
    {
        Ok(response) => response.text().await?,
        Err(_) => {
            return Err(ApiError::Message(format!(
                "Request failed - error sending request for url {}{}",
                API_BASE_URL, APP_LIST_ENDPOINT
            )));
        }
    };
    let body: Value = serde_json::from_str(&resp)?;
    let app_list = match body
        .get("response")
        .and_then(|response| response.get("apps"))
    {
        Some(apps) if !apps.is_null() => get_valid_apps(&body),
        _ => Vec::new(),
    };
    Ok(app_list)
}

async fn get_game_data(
    app_id: u32,
    with_price: bool,
    client: &reqwest::Client,
) -> Result<AppDetails, ApiError> {
    let app_id_str = app_id.to_string();
    let mut filters = String::from("basic");
    if with_price {
        filters.push_str(",price_overview");
    }
    let query_string = [
        ("appids", app_id_str.as_str()),
        ("filters", filters.as_str()),
    ];
    let url = format!("{}{}", STORE_BASE_URL, DETAILS_ENDPOINT);
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
        .query(&query_string)
        .send()
        .await?
        .text()
        .await?;

    let body: Value = serde_json::from_str(&resp)?;

    let response: HashMap<String, AppDetails> = serde_json::from_value(body)?;
    if response.is_empty() {
        return Err(ApiError::Message("The response was empty.".into()));
    }
    if response.len() > 1 {
        return Err(ApiError::Message(
            "More than one game was provided in response.".into(),
        ));
    }
    if let Some((_, details)) = response.into_iter().next()
        && let Some(app_data) = &details.app_data
        && app_data.steam_appid == app_id
    {
        Ok(details)
    } else {
        Err(ApiError::Message(
            "The game data is empty or there was some mismatch/missing data.".into(),
        ))
    }
}

async fn get_bulk_prices(app_ids: &[u32], client: &reqwest::Client) -> Result<AppPrices, ApiError> {
    let app_ids_str = app_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let query_string = [
        ("appids", app_ids_str.as_str()),
        ("filters", "price_overview"),
    ];
    let url = format!("{}{}", STORE_BASE_URL, DETAILS_ENDPOINT);
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))
        .query(&query_string)
        .send()
        .await?
        .text()
        .await?;

    let body: Value = serde_json::from_str(&resp)?;
    let app_prices: AppPrices = AppPrices::deserialize(&body)?;
    Ok(app_prices)
}

pub async fn get_price(app_id: u32, client: &reqwest::Client) -> Result<PriceOverview, ApiError> {
    SteamClient::new(client).get_price(app_id).await
}
pub async fn get_price_details(
    app_id: u32,
    client: &reqwest::Client,
) -> Result<SaleInfo, ApiError> {
    SteamClient::new(client).get_price_details(app_id).await
}

// Search Functions
pub async fn check_game(name: &str, client: &reqwest::Client) -> Option<App> {
    SteamClient::new(client).check_game(name).await
}

pub async fn search_by_keyphrase(
    keyphrase: &str,
    client: &reqwest::Client,
) -> Result<Vec<String>, ApiError> {
    SteamClient::new(client)
        .search_by_keyphrase(keyphrase)
        .await
}

pub async fn search_game(keyphrase: &str, client: &reqwest::Client) -> Option<String> {
    SteamClient::new(client).search_game(keyphrase).await
}

pub async fn get_bulk_filtered_price_details(
    app_ids: &[u32],
    desired_prices: &HashMap<String, f64>,
    client: &reqwest::Client,
) -> Result<Vec<SaleInfo>, ApiError> {
    SteamClient::new(client)
        .get_bulk_filtered_price_details(app_ids, desired_prices)
        .await
}

fn prioritize_fuzzy_results(search_list: &mut Vec<String>, keyphrase: &str) {
    let mut exact_matches: Vec<String> = Vec::new();
    let mut substring_matches: Vec<String> = Vec::new();
    let mut remaining_matches: Vec<String> = Vec::new();

    for title in search_list.drain(..) {
        if title.eq_ignore_ascii_case(keyphrase) {
            exact_matches.push(title);
        } else if title.contains(keyphrase) || keyphrase.contains(&title) {
            substring_matches.push(title);
        } else {
            remaining_matches.push(title);
        }
    }

    search_list.extend(exact_matches);
    search_list.extend(substring_matches);
    search_list.extend(remaining_matches);
}
