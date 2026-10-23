use iced::widget::image::Handle;
use std::collections::HashMap;
use std::sync::Arc;

use crate::utils::file_utils::load_url_image_with_fallback;
use alerting::email;
use constants::operations::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};
use file_ops::thresholds;
use stores::pc::{gog, microsoft_store, steam};
use stores::search::provider::SearchProvider;
use types::internal::{data::SaleInfo, store::GameStore};

#[derive(Debug, Clone)]
pub struct StoreSale {
    pub store: GameStore,
    pub info: SaleInfo,
    pub alias: String,
    pub game_id: String,
    pub icon_handler: Option<Handle>,
}

#[derive(Debug, Clone)]
pub struct SaleInfoCompare {
    pub icon_handler: Option<Handle>,
    pub title: String,
    pub steam_price: Option<f64>,
    pub gog_price: Option<f64>,
    pub microsoft_store_price: Option<f64>,
    pub lowest_price_stores: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SalesCache {
    pub store_sales: Vec<StoreSale>,
    pub comparisons: Vec<SaleInfoCompare>,
    pub by_store: HashMap<GameStore, Vec<usize>>,
}

impl SalesCache {
    pub fn new() -> Self {
        SalesCache {
            store_sales: Vec::new(),
            comparisons: Vec::new(),
            by_store: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.store_sales.clear();
        self.comparisons.clear();
        self.by_store.clear();
    }
}

pub fn get_simple_prices_str(store_name: GameStore, sales: Vec<SaleInfo>) -> String {
    let mut prices_str = String::new();
    sales.iter().for_each(|game| {
        prices_str.push_str(&format!(
            "\n\t- {} : {} -> {} ({}% off)",
            game.title, game.original_price, game.current_price, game.discount_percentage
        ));
    });
    if !prices_str.is_empty() {
        let header_str = format!("\n{} game(s) that met your desired price:", store_name);
        prices_str = header_str + &prices_str;
    }
    prices_str
}

pub fn check_prices_for_display(store_sales: &[StoreSale]) -> HashMap<GameStore, Vec<usize>> {
    let mut by_store: HashMap<GameStore, Vec<usize>> = HashMap::new();

    for i in 0..store_sales.len() {
        let sale: &StoreSale = store_sales.get(i).unwrap();
        by_store.entry(sale.store).or_default().push(i);
    }
    by_store
}

pub async fn check_prices_for_email(
    use_html: bool,
    search_provider: SearchProvider,
) -> Result<String, String> {
    let mut steam_sales: Vec<SaleInfo> = Vec::new();
    let mut gog_sales: Vec<SaleInfo> = Vec::new();
    let mut microsoft_store_sales: Vec<SaleInfo> = Vec::new();

    let store_sales = get_sales(&Arc::new(search_provider))
        .await
        .unwrap_or_default();
    for sale in store_sales {
        match sale.store {
            GameStore::STEAM => steam_sales.push(sale.info),
            GameStore::GOOD_OLD_GAMES => gog_sales.push(sale.info),
            GameStore::MICROSOFT_STORE_PC => microsoft_store_sales.push(sale.info),
        }
    }

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

    if output.is_empty() {
        Ok(String::from(
            "No thresholded games are currently below desired price.",
        ))
    } else {
        Ok(output)
    }
}

pub fn compare_prices(store_sales: &[StoreSale]) -> Vec<SaleInfoCompare> {
    let mut compared_sales: HashMap<String, SaleInfoCompare> = HashMap::new();
    for sale in store_sales.iter() {
        let cmp_id = if !sale.alias.is_empty() {
            sale.alias.clone()
        } else {
            sale.info.title.clone()
        };
        let entry = compared_sales
            .entry(cmp_id)
            .or_insert_with(|| SaleInfoCompare {
                icon_handler: sale.icon_handler.clone(),
                title: sale.info.title.clone(),
                steam_price: None,
                gog_price: None,
                microsoft_store_price: None,
                lowest_price_stores: Vec::new(),
            });

        let price = sale.info.current_price;
        match sale.store {
            GameStore::STEAM => entry.steam_price = Some(price),
            GameStore::GOOD_OLD_GAMES => entry.gog_price = Some(price),
            GameStore::MICROSOFT_STORE_PC => entry.microsoft_store_price = Some(price),
        }
    }
    let mut compiled_sales: Vec<SaleInfoCompare> = compared_sales.into_values().collect();
    for sale in compiled_sales.iter_mut() {
        let lowest_price = sale
            .steam_price
            .unwrap_or(f64::MAX)
            .min(sale.gog_price.unwrap_or(f64::MAX))
            .min(sale.microsoft_store_price.unwrap_or(f64::MAX));
        if sale.steam_price == Some(lowest_price) {
            sale.lowest_price_stores.push(STEAM_STORE_ID.into());
        }
        if sale.gog_price == Some(lowest_price) {
            sale.lowest_price_stores.push(GOG_STORE_ID.into());
        }
        if sale.microsoft_store_price == Some(lowest_price) {
            sale.lowest_price_stores.push(MICROSOFT_STORE_ID.into());
        }
    }
    compiled_sales
}

pub async fn get_sales(provider: &Arc<SearchProvider>) -> Result<Vec<StoreSale>, String> {
    let mut sales: Vec<StoreSale> = Vec::new();
    let thresholds = thresholds::load_thresholds().unwrap_or_else(|_e| Vec::new());

    let mut tasks: Vec<tokio::task::JoinHandle<(GameStore, Option<StoreSale>)>> = Vec::new();

    for game in thresholds.iter() {
        if game.steam_id != 0 {
            let steam_id = game.steam_id;
            let alias = game.alias.clone();
            let desired_price = game.desired_price;
            let steam_provider = Arc::clone(provider);

            let task = tokio::spawn(async move {
                let result = steam_provider
                    .result_search(|| async {
                        steam::get_price_details(steam_id, &steam_provider.client).await
                    })
                    .await;

                match result {
                    Ok(info) if desired_price >= info.current_price => {
                        let icon_handler =
                            Some(load_url_image_with_fallback(&info.icon_link).await);
                        (
                            GameStore::STEAM,
                            Some(StoreSale {
                                store: GameStore::STEAM,
                                info,
                                alias,
                                game_id: format!("{}_{}", STEAM_STORE_ID, steam_id),
                                icon_handler,
                            }),
                        )
                    }
                    _ => (GameStore::STEAM, None),
                }
            });
            tasks.push(task);
        }
    }

    for game in thresholds.iter() {
        if game.gog_id != 0 {
            let title = game.title.clone();
            let gog_id = game.gog_id;
            let alias = game.alias.clone();
            let desired_price = game.desired_price;

            let gog_provider = Arc::clone(provider);
            let task = tokio::spawn(async move {
                let result = gog_provider
                    .option_search(|| async {
                        gog::get_price_details_v2(&title, &gog_provider.client).await
                    })
                    .await;

                match result {
                    Some(info) if desired_price >= info.current_price => {
                        let icon_handler =
                            Some(load_url_image_with_fallback(&info.icon_link).await);
                        (
                            GameStore::GOOD_OLD_GAMES,
                            Some(StoreSale {
                                store: GameStore::GOOD_OLD_GAMES,
                                info,
                                alias,
                                game_id: format!("{}_{}", GOG_STORE_ID, gog_id),
                                icon_handler,
                            }),
                        )
                    }
                    _ => (GameStore::GOOD_OLD_GAMES, None),
                }
            });
            tasks.push(task);
        }

        if !game.microsoft_store_id.is_empty() {
            let store_id = game.microsoft_store_id.clone();
            let alias = game.alias.clone();
            let desired_price = game.desired_price;

            let ms_provider = Arc::clone(provider);
            let task = tokio::spawn(async move {
                let result = ms_provider
                    .option_search(|| async {
                        microsoft_store::get_price_details(&store_id, &ms_provider.client).await
                    })
                    .await;

                match result {
                    Some(info) if desired_price >= info.current_price => {
                        let icon_handler =
                            Some(load_url_image_with_fallback(&info.icon_link).await);
                        (
                            GameStore::MICROSOFT_STORE_PC,
                            Some(StoreSale {
                                store: GameStore::MICROSOFT_STORE_PC,
                                info,
                                alias,
                                game_id: format!("{}_{}", MICROSOFT_STORE_ID, store_id),
                                icon_handler,
                            }),
                        )
                    }
                    _ => (GameStore::MICROSOFT_STORE_PC, None),
                }
            });
            tasks.push(task);
        }
    }

    // Execute all tasks and return list of sales
    let results = futures::future::join_all(tasks).await;
    for (_store, sale_option) in results.into_iter().flatten() {
        if let Some(sale) = sale_option {
            sales.push(sale);
        }
    }
    Ok(sales)
}
