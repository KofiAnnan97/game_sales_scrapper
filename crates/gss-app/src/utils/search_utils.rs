use reqwest::Client;
use tokio::time::Duration;

use stores::pc::{gog, microsoft_store, steam};
use constants::operations::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};

use crate::StoreSearchResult;

static MAX_RESULTS: usize = 20;

pub async fn perform_store_search(query: String, store_id: String) -> Result<Vec<StoreSearchResult>, String> {
    if query.trim().is_empty() {
        return Err(String::from("Please enter a search query."));
    }
    let http_client = Client::new();
    let mut results = Vec::new();

    if store_id == STEAM_STORE_ID {
        match steam::search_by_keyphrase(&query).await {
            Ok(list) => {
                for title in list.into_iter().take(MAX_RESULTS) {
                    let steam_id = steam::check_game(&title).await.map(|app| app.app_id).unwrap_or(0);
                    // println!("Added Steam game: {}, {}", &title, &steam_id);
                    results.push(StoreSearchResult::Steam { title, steam_id });                    
                }
            }
            Err(e) => return Err(format!("Steam search error: {}", e)),
        }
    }
    else if store_id == GOG_STORE_ID {
        match gog::search_game_by_title_v2(&query, &http_client).await {
            Ok(list) => {
                for g in list.into_iter().take(MAX_RESULTS) {
                    let gog_id = g.id.parse::<u32>().unwrap_or(0);
                    // println!("Added GOG game: {}, {}", &g.title, &gog_id);
                    results.push(StoreSearchResult::Gog { title: g.title, gog_id });
                }
            }
            Err(e) => return Err(format!("GOG search error: {}", e)),
        }
    }
    else if store_id == MICROSOFT_STORE_ID {
        match microsoft_store::search_game_by_title(&query, &http_client).await {
            Ok(list) => {
                for info in list.into_iter().take(MAX_RESULTS) {
                    // println!("Added Microsoft game: {}, {}", &info.title, &info.product_id);
                    results.push(StoreSearchResult::Microsoft { title: info.title, ms_id: info.product_id.clone() });
                }
            }
            Err(e) => return Err(format!("Microsoft Store search error: {}", e)),
        }
    }

    if results.is_empty() {
        Err(String::from("No results found."))
    } else {
        Ok(results)
    }
}