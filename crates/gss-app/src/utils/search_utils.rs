use reqwest::Client;

use stores::pc::{gog, microsoft_store, steam};
use types::internal::store::GameStore;

use crate::StoreSearchResult;

static MAX_RESULTS: usize = 20;

pub async fn perform_store_search(query: String, game_store: GameStore) -> Result<Vec<StoreSearchResult>, String> {
    if query.trim().is_empty() {
        return Err(String::from("Please enter a search query."));
    }
    let http_client = Client::new();
    let mut results = Vec::new();

    match game_store {
        GameStore::STEAM => {
            match steam::search_by_keyphrase(&query).await {
                Ok(list) => {
                    for title in list.into_iter().take(MAX_RESULTS) {
                        let steam_id = steam::check_game(&title).await.map(|app| app.app_id).unwrap_or(0);
                        results.push(StoreSearchResult::Steam { title, steam_id });                    
                    }
                }
                Err(e) => return Err(format!("Steam search error: {}", e)),
            }
        },
        GameStore::GOOD_OLD_GAMES => {
            match gog::search_game_by_title_v2(&query, &http_client).await {
                Ok(list) => {
                    for game_info in list.into_iter().take(MAX_RESULTS) {
                        if game_info.price.is_some() 
                            && game_info.price.unwrap().final_money.amount.parse::<f64>().unwrap_or(0.) > 0. {
                                let gog_id = game_info.id.parse::<u32>().unwrap_or(0);
                                results.push(StoreSearchResult::Gog { title: game_info.title, gog_id });
                        }
                    }     
                }
                Err(e) => return Err(format!("GOG search error: {}", e))
            }
        },
        GameStore::MICROSOFT_STORE_PC => {
            match microsoft_store::search_game_by_title(&query, &http_client).await {
                Ok(list) => {
                    for info in list.into_iter().take(MAX_RESULTS) {
                        if info.price_info.price.is_some() && info.price_info.price.unwrap() > 0. {
                            results.push(StoreSearchResult::Microsoft { title: info.title, ms_id: info.product_id.clone() });
                        }
                    }
                }
                Err(e) => return Err(format!("Microsoft Store search error: {}", e)),
            }
        },
    }

    if results.is_empty() {
        Err(String::from("No results found."))
    } else {
        Ok(results)
    }
}