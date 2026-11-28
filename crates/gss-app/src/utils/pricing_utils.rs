use iced::widget::image::Handle;
use std::collections::HashMap;
use std::sync::Arc;

use errors::api::ApiError;
use notifications::email;
use operations::thresholds;
use stores::pc::{gog, microsoft_store, steam};
use stores::search::provider::{SearchError, SearchProvider};
use types::internal::{data::SaleInfo, store::GameStore};

use crate::utils::file_utils::load_url_image_with_fallback;

pub static SALES_FAILURE_SEPARATOR: &str = "; ";

type StoreSaleOption = Option<StoreSale>;
type FetchError = SearchError<ApiError>;

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

#[derive(Debug, Clone, Default)]
pub struct StoreFailures {
    pub store_call: usize,
    pub semaphore: usize,
    pub task: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SalesFetchResult {
    pub sales: Vec<StoreSale>,
    pub failures: HashMap<GameStore, StoreFailures>,
    pub errors: Vec<String>,
    pub attempted_lookups: usize,
}

impl SalesFetchResult {
    pub fn all_lookups_failed(&self) -> bool {
        let failed_lookups: usize = self
            .failures
            .values()
            .map(|failures| failures.store_call + failures.semaphore + failures.task)
            .sum();

        self.attempted_lookups > 0 && failed_lookups == self.attempted_lookups
    }

    pub fn failure_summary(&self) -> Option<String> {
        let stores = GameStore::get_all_stores();
        let has_failures = self
            .failures
            .values()
            .any(|failures| failures.store_call + failures.semaphore + failures.task > 0);
        if !has_failures {
            None
        } else {
            Some(
                stores
                    .into_iter()
                    .map(|store| {
                        let failures = self.failures.get(&store).cloned().unwrap_or_default();
                        let total = failures.store_call + failures.semaphore + failures.task;
                        let mut causes = format!(
                            "{} store calls, {} search capacity",
                            failures.store_call, failures.semaphore
                        );
                        if failures.task > 0 {
                            causes.push_str(&format!(", {} task", failures.task));
                        }
                        format!(
                            "{}: {} game(s) failed ({})",
                            store.get_name(),
                            total,
                            causes
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(SALES_FAILURE_SEPARATOR),
            )
        }
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

fn format_email_output(sales: Vec<SaleInfo>, store_type: GameStore, use_html: bool) -> String {
    if !sales.is_empty() {
        if use_html {
            email::create_store_cards(store_type, sales)
        } else {
            get_simple_prices_str(store_type, sales)
        }
    } else {
        String::new()
    }
}

pub async fn check_prices_for_email(
    use_html: bool,
    search_provider: SearchProvider,
) -> Result<String, String> {
    let mut steam_sales: Vec<SaleInfo> = Vec::new();
    let mut gog_sales: Vec<SaleInfo> = Vec::new();
    let mut microsoft_store_sales: Vec<SaleInfo> = Vec::new();

    let fetch_sales = get_sales(&Arc::new(search_provider))
        .await
        .map_err(|e| e.to_string())?;
    let all_failed = fetch_sales.all_lookups_failed();
    let fetch_summary = fetch_sales.failure_summary();
    for sale in fetch_sales.sales {
        match sale.store {
            GameStore::STEAM => steam_sales.push(sale.info),
            GameStore::GOOD_OLD_GAMES => gog_sales.push(sale.info),
            GameStore::MICROSOFT_STORE_PC => microsoft_store_sales.push(sale.info),
        }
    }

    let mut output = String::new();
    output.push_str(&format_email_output(
        steam_sales,
        GameStore::STEAM,
        use_html,
    ));
    output.push_str(&format_email_output(
        gog_sales,
        GameStore::GOOD_OLD_GAMES,
        use_html,
    ));
    output.push_str(&format_email_output(
        microsoft_store_sales,
        GameStore::MICROSOFT_STORE_PC,
        use_html,
    ));
    if all_failed && let Some(summary) = fetch_summary {
        return Err(format!(
            "All game lookups failed:\n{}",
            summary.replace(SALES_FAILURE_SEPARATOR, "\n")
        ));
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
            sale.lowest_price_stores
                .push(GameStore::STEAM.get_id().into());
        }
        if sale.gog_price == Some(lowest_price) {
            sale.lowest_price_stores
                .push(GameStore::GOOD_OLD_GAMES.get_id().into());
        }
        if sale.microsoft_store_price == Some(lowest_price) {
            sale.lowest_price_stores
                .push(GameStore::MICROSOFT_STORE_PC.get_id().into());
        }
    }
    compiled_sales
}

async fn has_desired_sale_price(
    store_type: GameStore,
    store_id: &str,
    info: SaleInfo,
    desired_price: f64,
    alias: String,
) -> Option<StoreSale> {
    if desired_price >= info.current_price {
        let icon_handler = Some(load_url_image_with_fallback(&info.icon_link).await);
        Some(StoreSale {
            store: store_type,
            info,
            alias,
            game_id: format!("{}_{}", store_type.get_id(), store_id),
            icon_handler,
        })
    } else {
        None
    }
}

pub async fn get_sales(provider: &Arc<SearchProvider>) -> Result<SalesFetchResult, String> {
    let thresholds = thresholds::load_thresholds().map_err(|e| e.to_string())?;

    let mut tasks: Vec<(
        GameStore,
        tokio::task::JoinHandle<Result<StoreSaleOption, FetchError>>,
    )> = Vec::new();

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

                let info = result?;
                let sale = has_desired_sale_price(
                    GameStore::STEAM,
                    &steam_id.to_string(),
                    info,
                    desired_price,
                    alias,
                )
                .await;
                Ok(sale)
            });
            tasks.push((GameStore::STEAM, task));
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
                    .result_search(|| async {
                        gog::get_price_details_v2(&title, &gog_provider.client).await
                    })
                    .await;

                let info = result?;
                let sale = has_desired_sale_price(
                    GameStore::GOOD_OLD_GAMES,
                    &gog_id.to_string(),
                    info,
                    desired_price,
                    alias,
                )
                .await;
                Ok(sale)
            });
            tasks.push((GameStore::GOOD_OLD_GAMES, task));
        }
    }

    for game in thresholds.iter() {
        if !game.microsoft_store_id.is_empty() {
            let store_id = game.microsoft_store_id.clone();
            let alias = game.alias.clone();
            let desired_price = game.desired_price;

            let ms_provider = Arc::clone(provider);
            let task = tokio::spawn(async move {
                let result = ms_provider
                    .result_search(|| async {
                        microsoft_store::get_price_details(&store_id, &ms_provider.client).await
                    })
                    .await;

                let info = result?;
                let sale = has_desired_sale_price(
                    GameStore::MICROSOFT_STORE_PC,
                    &store_id,
                    info,
                    desired_price,
                    alias,
                )
                .await;
                Ok(sale)
            });
            tasks.push((GameStore::MICROSOFT_STORE_PC, task));
        }
    }

    let mut fetch_result = SalesFetchResult {
        attempted_lookups: tasks.len(),
        ..SalesFetchResult::default()
    };

    for (store, result) in futures::future::join_all(
        tasks
            .into_iter()
            .map(|(store, task)| async move { (store, task.await) }),
    )
    .await
    {
        match result {
            Ok(Ok(Some(sale))) => fetch_result.sales.push(sale),
            Ok(Ok(None)) => {}
            Ok(Err(failure)) => {
                let counts = fetch_result.failures.entry(store).or_default();
                match failure {
                    SearchError::StoreError(e) => {
                        counts.store_call += 1;
                        fetch_result.errors.push(format!(
                            "{} game search failed: {}",
                            store.get_name(),
                            e
                        ));
                    }
                    SearchError::SemaphoreError(e) => {
                        counts.semaphore += 1;
                        fetch_result.errors.push(format!(
                            "{} search capacity failure: {}",
                            store.get_name(),
                            e
                        ));
                    }
                }
            }
            Err(e) => {
                fetch_result.failures.entry(store).or_default().task += 1;
                fetch_result
                    .errors
                    .push(format!("{} lookup task failed: {}", store.get_name(), e));
            }
        }
    }
    Ok(fetch_result)
}
