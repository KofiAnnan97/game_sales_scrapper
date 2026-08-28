use std::time::Duration;

use iced::widget::{
    Button, Checkbox, Column, Container, Scrollable, TextInput, button, center, column, container, pick_list, row, scrollable, stack, text,
};
use iced::{Alignment, Element, Font, Length, Task, clipboard, font};

use constants::icons::RETRY;
use constants::operations::settings::{GOG_STORE_NAME,MICROSOFT_STORE_NAME,STEAM_STORE_NAME};
use types::internal::filtering::*;
use types::internal::store::GameStore;

use crate::components::custom_styles::{self as cs, bold_text};
use crate::components::custom_widgets::{self as cw, game_comparison_row, game_sale_row, game_store_card};
use crate::utils::log_utils::{LogLevel};
use crate::utils::pricing_utils::{SalesCache, StoreSale, check_prices_for_display, compare_prices, get_sales};
use crate::{STATUS_ERR, LOADING_FRAMES_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewDisplayed {
    Sales,
    SalesCompare,
    EmailPreview,
}

impl std::fmt::Display for PreviewDisplayed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreviewDisplayed::Sales => write!(f, "Sales"),
            PreviewDisplayed::SalesCompare => write!(f, "Sales Comparison"),
            PreviewDisplayed::EmailPreview => write!(f, "Email Preview"),
        }
    }
}

impl Default for PreviewDisplayed {
    fn default() -> Self {
        Self::Sales
    }
}

impl Default for SalesCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum PreviewMessage {
    CheckPrices,
    ComparePrices(bool),
    PreviewStoreChanged(StoreOptions),
    PreviewPriceChanged(PriceOptions),
    PreviewSortByChanged(SortOptions),
    PreviewCustomLowerPriceChanged(String),
    PreviewCustomUpperPriceChanged(String),
    PreviewApplyFilters,
    PreviewResetFilters,
    RefreshSales,
    Tick,
    GetSales(Result<Vec<StoreSale>, String>),
    GetSalesUpdated(Result<SalesCache, String>),
    CopyLinkToClipboard(String, String),
    ResetCopyMessage,
    ResetToSales,
    // Message(s) to communicate up to App
    Exit,
    SendEmail,
    GetEmailResult(String, String),
    SendLogEvent(LogLevel, String),
    OpenEmailSettings,
    HideDialog,
}
pub struct PreviewView {
    active_view: PreviewDisplayed,
    sales_cache: SalesCache,
    cmp_sales_mode: bool,
    copied_link: Option<String>,
    preview_selected_store: Option<StoreOptions>,
    preview_price_filter: Option<PriceOptions>,
    preview_custom_price_lower: String,
    preview_custom_price_higher: String,
    preview_sort_by: Option<SortOptions>,
    filtered_sale_idxs: Vec<usize>,
    is_price_check_in_progress: bool,
    price_check_loading_frame: usize,
    status_message: String,
    pub message_details: String,
    pub show_dialog: bool,
}

impl Default for PreviewView {
    fn default() -> Self {
        Self {
            active_view: PreviewDisplayed::Sales,
            sales_cache: SalesCache::default(),
            cmp_sales_mode: false,
            copied_link: None,
            preview_selected_store: Some(StoreOptions::All),
            preview_price_filter: Some(PriceOptions::None),
            preview_custom_price_lower: String::new(),
            preview_custom_price_higher: String::new(),
            preview_sort_by: Some(SortOptions::None),
            filtered_sale_idxs: Vec::new(),
            is_price_check_in_progress: false,
            price_check_loading_frame: 0,
            status_message: String::new(),
            message_details: String::new(),
            show_dialog: false,
        }
    }
}

impl PreviewView {
    pub fn update(&mut self, message: PreviewMessage) -> Task<PreviewMessage> {
        match message {
            PreviewMessage::CheckPrices => {
                self.show_dialog = false;
                if self.cmp_sales_mode {
                    self.active_view = PreviewDisplayed::SalesCompare;
                } else {
                    self.active_view = PreviewDisplayed::EmailPreview;
                }
                Task::done(PreviewMessage::SendLogEvent(LogLevel::DEBUG,format!("Viewing {}", self.active_view)))
            }
            PreviewMessage::ComparePrices(toggled) => {
                self.show_dialog = false;
                self.cmp_sales_mode = toggled;
                if toggled {
                    self.active_view = PreviewDisplayed::SalesCompare;
                } else {
                    self.active_view = PreviewDisplayed::EmailPreview;
                }
                Task::done(PreviewMessage::SendLogEvent(LogLevel::DEBUG,format!("Comparison toggle set to {}", self.cmp_sales_mode)))
            }
            PreviewMessage::GetSales(sales_results) => {
                Task::perform(
                    async move {
                        match sales_results {
                            Ok(store_sales) => {
                                let by_store = check_prices_for_display(&store_sales);
                                let comparisons = compare_prices(&store_sales);
                                Ok(SalesCache {
                                    store_sales,
                                    comparisons,
                                    by_store,
                                })
                            }
                            Err(error) => Err(error),
                        }
                    },
                    PreviewMessage::GetSalesUpdated,
                )
            }
            PreviewMessage::GetSalesUpdated(cache_result) => {
                self.is_price_check_in_progress = false;
                self.price_check_loading_frame = 0;
                let (level, log_msg) = match cache_result {
                    Ok(new_sales_cache) => {
                        self.sales_cache = new_sales_cache;
                        let _ = self.filter_games();
                        self.status_message.clear();
                        self.message_details.clear();
                        (LogLevel::INFO, format!("Game(s) found on sale: {}", self.sales_cache.store_sales.len()))
                    }
                    Err(err_msg) => {
                        self.show_dialog = true;
                        self.status_message = STATUS_ERR.into();
                        self.message_details = "An issue occurred while looking for game sales. \
                            Please check your internet connection or try again later.".into();
                        (LogLevel::ERROR, format!("An error occurred while updating sales: {}", err_msg))
                    }
                };
                Task::done(PreviewMessage::SendLogEvent(level, log_msg))
            }
            PreviewMessage::ResetToSales => {
                self.active_view = PreviewDisplayed::Sales;
                if self.sales_cache.store_sales.is_empty() {
                    self.is_price_check_in_progress = true;
                    self.price_check_loading_frame = 0;
                    Task::batch(vec![
                        Task::perform(get_sales(),PreviewMessage::GetSales),
                        Task::done(PreviewMessage::SendLogEvent(LogLevel::INFO, "Check games for potential sales".into()))
                    ])
                } else {
                    Task::none()
                }
                // Task::none()
            }
            PreviewMessage::PreviewStoreChanged(choice) => {
                self.preview_selected_store = Some(choice);
                Task::none()
            }
            PreviewMessage::PreviewPriceChanged(choice) => {
                self.preview_price_filter = Some(choice);
                Task::none()
            }
            PreviewMessage::PreviewSortByChanged(choice) => {
                self.preview_sort_by = Some(choice);
                Task::none()
            }
            PreviewMessage::PreviewCustomLowerPriceChanged(lower) => {
                self.preview_custom_price_lower = lower;
                Task::none()
            }
            PreviewMessage::PreviewCustomUpperPriceChanged(upper) => {
                self.preview_custom_price_higher = upper;
                Task::none()
            }
            PreviewMessage::PreviewApplyFilters => {
                let (store, price, sort) = self.filter_games();
                let log_msg = format!("Sales Preview - applying the following filters: {}, {}, {}", store, price, sort);
                Task::done(PreviewMessage::SendLogEvent(LogLevel::DEBUG, log_msg))
            }
            PreviewMessage::PreviewResetFilters => {
                self.reset_sales_page();
                self.filtered_sale_idxs = (0..self.sales_cache.store_sales.len()).collect();
                Task::done(PreviewMessage::SendLogEvent(LogLevel::DEBUG, "Reset filters".into()))
            }
            PreviewMessage::RefreshSales => {
                self.is_price_check_in_progress = true;
                self.price_check_loading_frame = 0;
                self.sales_cache.clear();
                Task::batch(vec![
                    Task::perform(get_sales(),PreviewMessage::GetSales),
                    Task::done(PreviewMessage::SendLogEvent(LogLevel::DEBUG, "Refreshing sales".into()))
                ])
            }
            PreviewMessage::Tick => {
                if self.is_price_check_in_progress {
                    self.price_check_loading_frame = (self.price_check_loading_frame + 1) % LOADING_FRAMES_SIZE;
                }
                Task::none()
            }
            PreviewMessage::CopyLinkToClipboard(id, url) => {
                self.copied_link = Some(id);
                Task::batch([
                    clipboard::write(url),
                    Task::perform(
                        async {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                        },
                        |_| PreviewMessage::ResetCopyMessage,
                    ),
                ])
            }
            PreviewMessage::ResetCopyMessage => {
                self.copied_link = None;
                Task::none()
            }
            PreviewMessage::HideDialog => {
                self.show_dialog = false;
                if self.status_message.eq_ignore_ascii_case(STATUS_ERR) {
                    self.status_message.clear();
                    self.message_details.clear();
                }
                Task::none()
            }
            PreviewMessage::SendLogEvent(_, _) => Task::none(),
            PreviewMessage::Exit => Task::none(),
            PreviewMessage::SendEmail => Task::none(),
            PreviewMessage::GetEmailResult(level, msg) => {
                self.status_message = level;
                self.message_details = msg;
                self.show_dialog = true;
                Task::none()
            }
            PreviewMessage::OpenEmailSettings => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, PreviewMessage> {
        let get_sales_loading = cw::text_loading_indicator("Retrieving sales",self.price_check_loading_frame,LOADING_FRAMES_SIZE);
        let price_check_loading = cw::text_loading_indicator("Checking prices",self.price_check_loading_frame,LOADING_FRAMES_SIZE);
        let cmp_check_loading = cw::text_loading_indicator("Checking prices for comparison",self.price_check_loading_frame,LOADING_FRAMES_SIZE);

        let filters = container(
            row![
                bold_text("Filters: "),
                text("Store:"),
                pick_list(
                    &StoreOptions::LIST[..],
                    self.preview_selected_store.clone(),
                    PreviewMessage::PreviewStoreChanged,
                ),
                text("Price:"),
                if self.preview_price_filter != Some(PriceOptions::Custom) {
                    column![
                        pick_list(
                            &PriceOptions::LIST[..],
                            self.preview_price_filter.clone(),
                            PreviewMessage::PreviewPriceChanged,
                        )
                    ]
                } else {
                    column![
                        pick_list(
                            &PriceOptions::LIST[..],
                            self.preview_price_filter.clone(),
                            PreviewMessage::PreviewPriceChanged,
                        ),
                        row![
                            TextInput::new("25.00", &self.preview_custom_price_lower)
                                .on_input(PreviewMessage::PreviewCustomLowerPriceChanged)
                                .width(Length::Fixed(70.0))
                                .padding(5),
                            TextInput::new("80.00", &self.preview_custom_price_higher)
                                .on_input(PreviewMessage::PreviewCustomUpperPriceChanged)
                                .width(Length::Fixed(70.0))
                                .padding(5),
                        ]
                    ]
                },
                text("Sort by:"),
                pick_list(
                    &SortOptions::LIST[..],
                    self.preview_sort_by.clone(),
                    PreviewMessage::PreviewSortByChanged,
                ),
                button("Apply")
                    .on_press(PreviewMessage::PreviewApplyFilters)
                    .padding(6),
                button("Reset")
                    .on_press(PreviewMessage::PreviewResetFilters)
                    .padding(6),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(5)
        .width(Length::Fill);

        let sales_header: Container<'_, PreviewMessage> = container(
            row![
                text("Game").width(Length::FillPortion(2)),
                text("Store").width(Length::FillPortion(1)),
                text("Price").width(Length::FillPortion(1)),
            ]
        )
        .padding(5);

        let mut sales_rows: Column<'_, PreviewMessage> = column![];

        if self.status_message.contains(STATUS_ERR) {
            sales_rows = sales_rows.push(text("Retrieving sales failed."));
        } else if self.sales_cache.store_sales.is_empty() {
            sales_rows = sales_rows.push(text("No games are on sale for your desired prices."));
        } else if self.filtered_sale_idxs.is_empty() {
            sales_rows = sales_rows.push(text("No games could be found with applied filters."));
        } else {
            // if self.filtered_sale_idxs.is_empty() {
            // for (idx, sale) in self.sales_cache.store_sales.iter().enumerate() {
            //     sales_rows = sales_rows.push(game_sale_row(sale, idx));
            // }
            //}
            for (i, sale_idx) in self.filtered_sale_idxs.iter().enumerate() {
                let sale = &self.sales_cache.store_sales[*sale_idx];
                sales_rows = sales_rows.push(game_sale_row(sale, i));
            }
        }

        let product_list =
            if self.is_price_check_in_progress {
                container(get_sales_loading).height(Length::Fill)
            } else {
                container(
                    column![
                        sales_header,
                        scrollable(sales_rows).height(Length::Fill),
                    ]
                )
                .padding(5)
                .width(Length::Fill)
            };

        let footer = container(
            row![
                button("Email Preview")
                    .on_press(PreviewMessage::CheckPrices)
                    .padding(6),
                button("Close")
                    .on_press(PreviewMessage::Exit)
                    .padding(6),
            ]
            .spacing(10)
        )
        .padding(2)
        .width(Length::Fill);

        let preview_space = container(
            column![
                filters,
                product_list,
                footer.height(50),
            ]
            .spacing(5)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .padding(10);

        // EMAIL PREVIEW

        let mut sales_by_store: Column<'_, PreviewMessage> = column![];

        let current_sales_exist = self.sales_cache.by_store.values()
            .any(|sales| !sales.is_empty());

        if self.status_message.contains(STATUS_ERR) {
            sales_by_store = sales_by_store.push(text("Retrieving sales failed."));
        } else if !current_sales_exist {
            sales_by_store = sales_by_store.push(text("No games are on sale for your desired prices."));
        } else {
            for (store, sale_idxs) in &self.sales_cache.by_store {
                sales_by_store = sales_by_store.push(
                    container(
                        text(store.get_name())
                            .size(24.0)
                            .font(Font {
                                weight: font::Weight::Bold,
                                ..Font::DEFAULT
                            })
                    )
                    .padding(10.0)
                );

                for idx in sale_idxs {
                    let game_sale = &self.sales_cache.store_sales[*idx];
                    sales_by_store = sales_by_store.push(game_store_card(&self.copied_link,game_sale, PreviewMessage::CopyLinkToClipboard));
                }
            }
        }

        let price_check_scrollable =
            Scrollable::new(
                if self.is_price_check_in_progress {
                    column![price_check_loading]
                } else {
                    column![sales_by_store]
                }
            )
            .width(Length::Fill)
            .height(Length::Fill);

        // COMPARISON PREVIEW

        let mut sales_comparisons: Column<'_, PreviewMessage> = column![];

        if self.status_message.contains(STATUS_ERR) {
            sales_comparisons = sales_comparisons.push(text("Retrieving sales failed."));
        } else if !current_sales_exist {
            sales_comparisons =
                sales_comparisons.push(text("No games to compare sale prices against."));
        } else {
            let cmp_header = row![
                bold_text("Game")
                    .size(20)
                    .width(Length::FillPortion(2)),
                bold_text(STEAM_STORE_NAME)
                    .size(20)
                    .width(Length::FillPortion(1))
                    .center(),
                bold_text(GOG_STORE_NAME)
                    .size(20)
                    .width(Length::FillPortion(1))
                    .center(),
                bold_text(MICROSOFT_STORE_NAME)
                    .size(20)
                    .width(Length::FillPortion(1))
                    .center(),
            ];

            sales_comparisons = sales_comparisons.push(container(cmp_header).width(Length::Fill));
            for (idx, game) in self.sales_cache.comparisons.iter().enumerate() {
                sales_comparisons = sales_comparisons.push(game_comparison_row(game,idx).width(Length::Fill));
            }
        }

        let sales_comparisons_scrollable = Scrollable::new(
            if self.is_price_check_in_progress {
                column![cmp_check_loading]
            } else {
                column![sales_comparisons]
            }
        )
        .height(Length::Fill);

        // EMAIL SPACE

        let email_space = container(
            column![
                row![
                    bold_text("Email Preview").size(36),
                    Button::new("(Go to email settings)")
                        .on_press(PreviewMessage::OpenEmailSettings)
                        .style(button::text),
                ]
                .align_y(Alignment::Center)
                .spacing(10),
                Checkbox::new(self.cmp_sales_mode)
                    .label("Compare Sales")
                    .on_toggle(PreviewMessage::ComparePrices)
                    .spacing(10),
                if self.active_view == PreviewDisplayed::SalesCompare {
                    sales_comparisons_scrollable
                } else {
                    price_check_scrollable
                },
                container(
                    row![
                        Button::new("Send Manual Email")
                            .on_press(PreviewMessage::SendEmail)
                            .padding(6),
                        Button::new("Exit Email Preview")
                            .on_press(PreviewMessage::ResetToSales)
                            .padding(6),
                    ]
                    .spacing(10)
                    .padding(8)
                )
                .align_right(Length::Fill)
                .height(50),
            ]
        );

        let screen = match self.active_view {
            PreviewDisplayed::Sales => preview_space,
            PreviewDisplayed::SalesCompare | PreviewDisplayed::EmailPreview => email_space,
        };

        let content = column![
            container(
                button(text(format!("Refresh Sales {}", RETRY)))
                    .on_press(PreviewMessage::RefreshSales)
            )
            .align_right(Length::Fill)
            .padding(5),
            screen,
        ]
        .height(Length::Fill)
        .spacing(5)
        .padding(5);

        if self.show_dialog {
            stack![
                content,
                cs::backdrop(PreviewMessage::HideDialog),
                center(
                    cw::message_dialog(
                        &self.status_message,
                        &self.message_details,
                        PreviewMessage::HideDialog
                    )
                )
            ]
            .into()
        } else {
            content.into()
        }
    }
    
    fn reset_sales_page(&mut self) {
        self.preview_selected_store = Some(StoreOptions::All);
        self.preview_price_filter = Some(PriceOptions::None);
        self.preview_sort_by = Some(SortOptions::None);
        self.preview_custom_price_lower.clear();
        self.preview_custom_price_higher.clear();
    }

    fn filter_games(&mut self) -> (String, String, String) {
        let store_filter_type = self.preview_selected_store.unwrap_or(StoreOptions::All);
        let price_filter_type = self.preview_price_filter.unwrap_or(PriceOptions::None);
        let sort_filter_type = self.preview_sort_by.unwrap_or(SortOptions::None);

        if store_filter_type == StoreOptions::All && price_filter_type == PriceOptions::None && sort_filter_type == SortOptions::None {
            self.filtered_sale_idxs = (0..self.sales_cache.store_sales.len()).collect();
        } else {
            let mut filtered_idxs = store_filter(&self.sales_cache.store_sales, store_filter_type);
            let low_price =  self.preview_custom_price_lower.parse::<f64>().ok();
            let high_price = self.preview_custom_price_higher.parse::<f64>().ok();
            filtered_idxs = price_filter(&self.sales_cache.store_sales, filtered_idxs, price_filter_type, low_price, high_price);
            filtered_idxs = sort_by_filter(&self.sales_cache.store_sales, filtered_idxs, sort_filter_type);
            self.filtered_sale_idxs = filtered_idxs;
        }
        (store_filter_type.to_string(), price_filter_type.to_string(), sort_filter_type.to_string())        
    }
}

pub fn store_filter(games: &[StoreSale],filter_type: StoreOptions) -> Vec<usize> {
    match filter_type {
        StoreOptions::Steam => games.iter().enumerate().filter_map(|(idx, game)| {
            (game.store == GameStore::STEAM).then_some(idx)
        })
        .collect(),
        StoreOptions::GOG => games.iter().enumerate().filter_map(|(idx, game)| {
            (game.store == GameStore::GOOD_OLD_GAMES).then_some(idx)
        })
        .collect(),
        StoreOptions::MicrosoftStore => games.iter().enumerate().filter_map(|(idx, game)| {
            (game.store == GameStore::MICROSOFT_STORE_PC).then_some(idx)
        })
        .collect(),
        StoreOptions::All => (0..games.len()).collect(),
    }
}

pub fn price_filter(games: &[StoreSale], idxs: Vec<usize>, filter_type: PriceOptions, low_price: Option<f64>, high_price: Option<f64>) -> Vec<usize> {
    match filter_type {
        PriceOptions::None => idxs,
        PriceOptions::Under5 => idxs.into_iter().filter(|&i| {
            games[i].info.current_price < 5.0
        })
        .collect(),
        PriceOptions::Under10 => idxs.into_iter().filter(|&i| {
            games[i].info.current_price < 10.0
        })
        .collect(),
        PriceOptions::Under25 => idxs.into_iter().filter(|&i| {
            games[i].info.current_price < 25.0
        })
        .collect(),
        PriceOptions::Custom => {
            let lowest = low_price.unwrap_or(0.);
            let highest = high_price.unwrap_or(0.);
            idxs.into_iter().filter(|&i| {
                let price = games[i].info.current_price;
                price >= lowest && price < highest
            })
            .collect()
        }
    }
}

pub fn sort_by_filter(games: &[StoreSale],idxs: Vec<usize>,filter_type: SortOptions) -> Vec<usize> {
    if filter_type == SortOptions::None {
        return idxs;
    }

    let mut tmp: Vec<(usize, &StoreSale)> =idxs.iter().map(|&idx| {
        (idx, &games[idx])
    })
    .collect();

    match filter_type {
        SortOptions::None => {},
        SortOptions::AToZ => {
            tmp.sort_by(|a, b| {
                a.1.info.title.cmp(&b.1.info.title)
            });
        },
        SortOptions::ZToA => {
            tmp.sort_by(|a, b| {
                b.1.info.title.cmp(&a.1.info.title)
            });
        },
        SortOptions::LowToHigh => {
            tmp.sort_by(|a, b| {
                a.1.info.current_price.total_cmp(&b.1.info.current_price)
            });
        },
        SortOptions::HighToLow => {
            tmp.sort_by(|a, b| {
                b.1.info.current_price.total_cmp(&a.1.info.current_price)
            });
        }
    }

    tmp.into_iter().map(|(idx, _)| idx).collect()
}