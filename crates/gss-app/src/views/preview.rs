use iced::widget::{Button, Checkbox, Scrollable, TextInput, column, container, row, text, pick_list, button, scrollable};
use iced::{Alignment, Element, Font, Length, font};

use constants::icons::RETRY;
use constants::operations::settings::{STEAM_STORE_NAME, GOG_STORE_NAME, MICROSOFT_STORE_NAME};
use types::internal::filtering::{PriceOptions, SortOptions, StoreOptions};
use types::internal::store::GameStore;

use crate::components::custom_styles::bold_text;
use crate::components::custom_widgets::{game_comparison_row, game_sale_row, game_store_card};
use crate::components::{custom_widgets};
use crate::utils::pricing_utils::StoreSale;
use crate::views::settings::SettingsPage;
use crate::{LOADING_FRAMES_SIZE, Message, STATUS_ERR};

#[derive(PartialEq)]
pub enum PreviewDisplayed {
    Sales,
    SalesCompare,
    EmailPreview,
}

pub fn sale_preview_view(app: &crate::App) -> Element<'_, Message> {
    let get_sales_loading = custom_widgets::text_loading_indicator("Retrieving sales", app.price_check_loading_frame, LOADING_FRAMES_SIZE);
    let price_check_loading = custom_widgets::text_loading_indicator("Checking prices", app.price_check_loading_frame, LOADING_FRAMES_SIZE);
    let cmp_check_loading = custom_widgets::text_loading_indicator("Checking prices for comparison", app.price_check_loading_frame, LOADING_FRAMES_SIZE);

    // Sales Preview

    let filters = container(
        column![
            row![
                bold_text("Filters: "),
                text("Store:"),
                pick_list(
                    &StoreOptions::LIST[..],
                    app.preview_selected_store.clone(),
                    Message::PreviewStoreChanged
                ),
                text("Price:"),
                if app.preview_price_filter != Some(PriceOptions::Custom) {
                    column![
                        pick_list(
                            &PriceOptions::LIST[..],
                            app.preview_price_filter.clone(),
                            Message::PreviewPriceChanged
                        ),
                    ]
                } else {
                    column![
                        pick_list(
                            &PriceOptions::LIST[..],
                            app.preview_price_filter.clone(),
                            Message::PreviewPriceChanged
                        ),
                        row![
                            TextInput::new("25.00", &app.preview_custom_price_lower)
                                .on_input(Message::PreviewCustomLowerPriceChanged)
                                .width(Length::Fixed(50.))
                                .padding(5), 
                            TextInput::new("80.00", &app.preview_custom_price_higher)
                                .on_input(Message::PreviewCustomUpperPriceChanged)
                                .width(Length::Fixed(50.))
                                .padding(5),
                        ]
                    ]
                },   
                text("Sort by:"),
                pick_list(
                    &SortOptions::LIST[..],
                    app.preview_sort_by.clone(),
                    Message::PreviewSortByChanged
                ),

                button("Apply")
                    .on_press(Message::PreviewApplyFilters)
                    .padding(6),

                button("Reset")
                    .on_press(Message::PreviewResetFilters)
                    .padding(6),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        ]
    )
    .padding(5)
    .width(Length::Fill);

    let sales_header = container(
        row![
            text("Game").width(Length::FillPortion(2)),
            text("Store").width(Length::FillPortion(1)),
            text("Price").width(Length::FillPortion(1)),
        ]
    )
    .padding(5);

    let mut sales_rows = column![];
    if !app.message_details.is_empty() && app.status_message == STATUS_ERR {
        sales_rows = sales_rows.push(text("Retrieving sales failed."));
    } else if app.sales_cache.store_sales.is_empty() {
        sales_rows = sales_rows.push(text("No games are on sale for your desired prices."));
    } else if app.filtered_sale_idxs.is_empty() {
        sales_rows = sales_rows.push(text("No games could be found with applied filters."));
    } else {
        // if app.filtered_sale_idxs.is_empty() {
        // for (idx, sale) in app.sales_cache.store_sales.iter().enumerate() {
        //     sales_rows = sales_rows.push(game_sale_row(sale, idx));
        // }
        //}
        for idx in app.filtered_sale_idxs.clone() {
            sales_rows = sales_rows.push(game_sale_row(&app.sales_cache.store_sales[idx], idx));
        }
    }

    let product_list = if app.is_price_check_in_progress {
        container(get_sales_loading)
            .height(Length::Fill)
    } else {
        container(
            column![
                sales_header,
                scrollable(sales_rows)
                    .height(Length::Fill)
            ]
        )
        .padding(5)
        .width(Length::Fill)
    };

    let footer = container(
        row![
            button("Email Preview")
                .on_press(Message::CheckPrices)
                .padding(6),
            button("Close")
                .on_press(Message::CloseSalesPreview)
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

    // Email Preview

    let mut sales_by_store = column![];
    let current_sales: Vec<_> = app.sales_cache.by_store
        .iter()
        .filter(|(_, sales)| sales.len() > 0)
        .map(|(store_front,_)| store_front)
        .collect();
    if !app.message_details.is_empty() && app.status_message == STATUS_ERR {
        sales_by_store = sales_by_store.push(text("Retrieving sales failed."));
    } else if current_sales.is_empty() {
        sales_by_store = sales_by_store.push(text("No games are on sale for your desired prices."));
    } else {
        for (store, sale_idxs) in &app.sales_cache.by_store {
            if sale_idxs.is_empty() { continue; }
            sales_by_store = sales_by_store.push(
                container(text(store.get_name()).size(24.0)
                    .font(Font{
                        weight: font::Weight::Bold,
                        ..Font::DEFAULT
                    }))
                    .padding(10.0)
            );
            for idx in sale_idxs {
                let game_sale = &app.sales_cache.store_sales[*idx];
                sales_by_store = sales_by_store.push(game_store_card(&app.copied_link, &game_sale));
            }
        }
    }

    let price_check_scrollable = Scrollable::new(
        if app.is_price_check_in_progress {
            column![price_check_loading]
        } else {
            column![sales_by_store]
        }
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let mut sales_comparisons = column![];
    if !app.message_details.is_empty() && app.status_message == STATUS_ERR  {
        sales_comparisons = sales_comparisons.push(text("Retrieving sales failed."));
    } else if current_sales.is_empty() {
        sales_comparisons = sales_comparisons.push(text("No games to compare sale prices against."));
    } else {
        let cmp_header = row![
            bold_text("Game").size(20).width(Length::FillPortion(2)),
            bold_text(STEAM_STORE_NAME).size(20).width(Length::FillPortion(1)).center(),
            bold_text(GOG_STORE_NAME).size(20).width(Length::FillPortion(1)).center(),
            bold_text(MICROSOFT_STORE_NAME).size(20).width(Length::FillPortion(1)).center(),
        ];
        sales_comparisons = sales_comparisons.push(container(cmp_header).width(Length::Fill));
        for idx in 0..app.sales_cache.comparisons.len() {
            let game = &app.sales_cache.comparisons[idx];
            sales_comparisons = sales_comparisons.push(game_comparison_row(game, idx).width(Length::Fill));
        }
    }

    let sales_comparisons_scrollable = Scrollable::new(
        if app.is_price_check_in_progress {
            column![cmp_check_loading]
        } else {
            column![sales_comparisons]
        }
    ).height(Length::Fill);
    
    let email_space = container(
        column![
            row![
                bold_text("Email Preview").size(36),
                Button::new("(Go to email settings)")
                    .on_press(Message::SettingsPageSelected(SettingsPage::Email))
                    .style(button::text)
            ]
            .align_y(Alignment::Center)
            .spacing(10),
            Checkbox::new(app.cmp_sales_mode)
                .label("Compare Sales")
                .on_toggle(Message::ComparePrices)
                .spacing(10),
            if app.preview_displayed == PreviewDisplayed::SalesCompare {
                sales_comparisons_scrollable
            } else {
                price_check_scrollable
            },
            container(
                row![
                    Button::new("Send Manual Email")
                        .on_press(Message::SendEmail)
                        .padding(6),
                    Button::new("Exit Email Preview")
                        .on_press(Message::ExitEmailPreview)
                        .padding(6),
                ]
                .spacing(10)
                .padding(8)
            )
            .align_right(Length::Fill)
            .height(50)
        ]
    );

    let screen_displayed = match app.preview_displayed {
        PreviewDisplayed::Sales => preview_space,
        _ => email_space
    };

    column![
        container(
            button(text(format!("Refresh Sales {}", RETRY)))
                .on_press(Message::RefreshSales)
        )
        .align_right(Length::Fill)
        .padding(5),
        screen_displayed
    ]
    .height(Length::Fill)
    .spacing(5)
    .padding(5)
    .into()
}


pub fn store_filter(games: &Vec<StoreSale>, filter_type: StoreOptions) -> Vec<usize> {
    let mut filtered: Vec<usize> = Vec::new();
    match filter_type {
        StoreOptions::Steam => {
            for (idx, game) in games.iter().enumerate() {
                if game.store == GameStore::STEAM {
                    filtered.push(idx);
                }
            }
        },
        StoreOptions::GOG => {
            for (idx, game) in games.iter().enumerate() {
                if game.store == GameStore::GOOD_OLD_GAMES {
                    filtered.push(idx);
                }
            }
        },
        StoreOptions::MicrosoftStore => {
            for (idx, game) in games.iter().enumerate() {
                if game.store == GameStore::MICROSOFT_STORE_PC {
                    filtered.push(idx);
                }
            }
        },
        StoreOptions::All => filtered = (0..games.len()).collect(),
    };
    filtered
}

pub fn price_filter(games: &Vec<StoreSale>, idxs: Vec<usize>, filter_type: PriceOptions, low_price: Option<f64>, high_price: Option<f64>) -> Vec<usize> {
    let mut filtered: Vec<usize> = Vec::new();
    match filter_type {
        PriceOptions::None => filtered = idxs,
        PriceOptions::Under5 => {
            for i in idxs {
                if games[i].info.current_price < 5. {
                    filtered.push(i);
                }
            }
        },
        PriceOptions::Under10 => {
            for i in idxs {
                if games[i].info.current_price < 10. {
                    filtered.push(i);
                }
            }
        },
        PriceOptions::Under25 => {
            for i in idxs {
                if games[i].info.current_price < 25. {
                    filtered.push(i);
                }
            }
        },
        PriceOptions::Custom => {
            let lowest_price = low_price.unwrap_or(0.);
            let highest_price = high_price.unwrap_or(0.);
            for i in idxs {
                if games[i].info.current_price >= lowest_price && 
                    games[i].info.current_price < highest_price {
                        filtered.push(i);
                }
            }
        },
    }
    filtered
}

pub fn sort_by_filter(games: &Vec<StoreSale>, idxs: Vec<usize>, filter_type: SortOptions) -> Vec<usize> {
    let filtered;
    let mut tmp: Vec<(StoreSale, usize)> = Vec::new();
    for i in idxs.iter() {
        tmp.push((games[*i].clone(),*i));
    }
    match filter_type {
        SortOptions::None => filtered = idxs,
        SortOptions::AToZ => {
            tmp.sort_by(|a, b| a.0.info.title.cmp(&b.0.info.title));
            filtered = tmp.iter().map(|(_, idx)|  *idx).collect();
        },
        SortOptions::ZToA => {
            tmp.sort_by(|a, b| a.0.info.title.cmp(&b.0.info.title));
            filtered = tmp.iter().rev().map(|(_, idx)|  *idx).collect();
        },
        SortOptions::LowToHigh => {
            tmp.sort_by(|a, b| a.0.info.current_price.total_cmp(&b.0.info.current_price));
            filtered = tmp.iter().map(|(_, idx)|  *idx).collect();
        },
        SortOptions::HighToLow => {
            tmp.sort_by(|a, b| a.0.info.current_price.total_cmp(&b.0.info.current_price));
            filtered = tmp.iter().rev().map(|(_, idx)|  *idx).collect();
        },
    }
    filtered
}