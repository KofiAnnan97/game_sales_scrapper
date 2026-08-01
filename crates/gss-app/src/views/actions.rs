use iced::widget::{Button, Checkbox, Scrollable, column, container, row, text};
use iced::{font, Element, Font, Length};

use file_ops::settings;
use file_types::general;
use constants::operations::settings::{STEAM_STORE_NAME, GOG_STORE_NAME, MICROSOFT_STORE_NAME};

use crate::components::custom_styles::bold_text;
use crate::components::custom_widgets::{game_comparison_row, game_store_card};
use crate::components::{custom_widgets};
use crate::{LOADING_FRAMES_SIZE, Message};
// use crate::utils::log_utils;

pub enum ActionDisplayed{
    NoAction,
    CheckPrices,
    ComparePrices,
    UpdateCache,
    TestEmail,
    Logs,
}

pub fn view_actions(app: &crate::App) -> Element<'_, Message> {
    let cache_loading = custom_widgets::text_loading_indicator("Retrieve games to cache", app.caching_loading_frame, LOADING_FRAMES_SIZE);
    let price_check_loading = custom_widgets::text_loading_indicator("Checking prices", app.price_check_loading_frame, LOADING_FRAMES_SIZE);
    let cmp_check_loading = custom_widgets::text_loading_indicator("Checking prices for comparison", app.price_check_loading_frame, LOADING_FRAMES_SIZE);

    let mut sales_by_store = column![];
    let current_sales: Vec<_> = app.sales_cache.by_store
        .iter()
        .filter(|(_, sales)| sales.len() > 0)
        .map(|(store_front,_)| store_front)
        .collect();
    if current_sales.is_empty() {
        sales_by_store = sales_by_store.push(text("No games are on sale for your desired prices."));
    } else {
        for (store, sale_idxs) in &app.sales_cache.by_store {
            if sale_idxs.is_empty() { continue; }
            let store_name = settings::get_proper_store_name(&store).unwrap_or_else(|| store.clone());
            sales_by_store = sales_by_store.push(
                container(text(store_name).size(32.0)
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
    ).width(Length::Fill);

    let mut sales_comparisons = column![];
    if current_sales.is_empty() {
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
    );

    let complete_logs = general::get_contents(&app.current_log_file) + &app.log_batch;
    let logs_display = Scrollable::new(text(complete_logs))
        .width(Length::Fill)
        .height(Length::Fill);

    let status_display = Scrollable::new(text(&app.status_message));

    let caching_display = Scrollable::new(
        if app.is_caching_in_progress {
            cache_loading
        } else {
            text(&app.status_message)
        }
    );

    let display_results = match app.action_displayed {
        ActionDisplayed::CheckPrices => price_check_scrollable,
        ActionDisplayed::ComparePrices => sales_comparisons_scrollable,
        ActionDisplayed::Logs => logs_display,
        ActionDisplayed::TestEmail => status_display,
        ActionDisplayed::UpdateCache => caching_display,
        ActionDisplayed::NoAction => Scrollable::new(text(""))
    };

    column![
        row![
            column![
                Button::new(text("Check Prices").center())
                    .on_press(Message::CheckPrices)
                    .width(Length::Fixed(160.))
                    .padding(10),
                Checkbox::new(app.cmp_sales_mode)
                    .label("Compare Sales")
                    .on_toggle(Message::ComparePrices)
                    .spacing(10),
            ],
            Button::new(text("Send Test Email"))
                .on_press(Message::SendEmail)
                .padding(10),
            Button::new(text("Update Game Cache"))
                .on_press(Message::UpdateCache)
                .padding(10),
            Button::new(text("Show Logs"))
                .on_press(Message::LogsShown)
                .padding(10)
        ]
        .spacing(10),
        display_results,
    ]
    .spacing(15)
    .padding(10)
    .into()
}