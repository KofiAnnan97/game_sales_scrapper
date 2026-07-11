use file_ops::settings;
use file_types::general;
use iced::widget::{Button, Scrollable, column, container, row, text};
use iced::{font, Element, Font, Length};

use crate::components::{custom_widgets};

use crate::{LOADING_FRAMES_SIZE, Message};
// use crate::utils::log_utils;

pub enum ActionDisplayed{
    NoAction,
    CheckPrices,
    UpdateCache,
    TestEmail,
    Logs,
}

pub fn view_actions(app: &crate::App) -> Element<'_, Message> {
    let cache_loading = custom_widgets::text_loading_indicator("Retrieve games to cache", app.caching_loading_frame, LOADING_FRAMES_SIZE);
    let price_check_loading = custom_widgets::text_loading_indicator("Checking prices", app.price_check_loading_frame, LOADING_FRAMES_SIZE);

    let mut sales_by_store = column![];
    for (store, games) in app.sales_info_by_store.iter() {
        if games.is_empty() { continue; }
        let store_name = settings::get_proper_store_name(store).unwrap_or_else(|| store.clone());
        sales_by_store = sales_by_store.push(
            container(text(store_name).size(40.0)
                .font(Font{
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                }))
                .center_x(Length::Fill)
                .padding(10.0)
        );
        sales_by_store = sales_by_store.push(container(custom_widgets::create_sales_table( &games)).center_x(Length::Fill));
    }

    let stores_with_sales: Vec<_> = app.sales_info_by_store
        .iter()
        .filter(|(_, sales)| sales.len() > 0)
        .map(|(store_front,_)| store_front)
        .collect();
    if stores_with_sales.is_empty() {
        sales_by_store = sales_by_store.push(text("No games are on sale for your desired prices."));
    }

    let price_check_scrollable = Scrollable::new(
        if app.is_price_check_in_progress {
            column![price_check_loading]
        } else {
            column![sales_by_store]
        }
    ).width(Length::Fill);

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
        ActionDisplayed::Logs => logs_display,
        ActionDisplayed::TestEmail => status_display,
        ActionDisplayed::UpdateCache => caching_display,
        ActionDisplayed::NoAction => Scrollable::new(text(""))
    };

    column![
        row![
            Button::new(text("Check prices"))
                .on_press(Message::CheckPrices)
                .padding(10),
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