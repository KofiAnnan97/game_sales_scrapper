use iced::widget::{Button, Radio, Scrollable, TextInput, column, container, row, text};
use iced::{Element, Length};

use crate::Message;
use file_ops::settings;

pub const SKIP_STORE_SELECTION: usize = usize::MAX;

pub fn search_tab(app: &crate::App) -> Element<'_, Message> {
    let store_section = if app.search_results_by_store.is_empty() {
        column![text("No search in progress. Enter a query and press Search.")]
    } else {
        let (current_store_id, current_results) = &app.search_results_by_store[app.current_store_search_idx];
        let store_name = settings::get_proper_store_name(current_store_id)
            .unwrap_or_else(|| current_store_id.clone());
        let progress = format!("Store {}/{}: {}  ",
            app.current_store_search_idx + 1,
            app.search_results_by_store.len(),
            store_name
        );

        let mut search_list = column![];
        let selected_index_opt = match app.selected_results_by_store.get(current_store_id) {
            Some(Some(index)) => Some(*index),
            Some(None) => Some(SKIP_STORE_SELECTION),
            None => None,
        };

        if current_results.is_empty() {
            search_list = column![
                text("No results for this store."),
                Radio::new(
                    String::from("Skip this store"),
                    SKIP_STORE_SELECTION,
                    selected_index_opt,
                    Message::SearchResultSelected,
                )
            ].spacing(5);
        } else {
            search_list = search_list.push(
                Radio::new(
                    String::from("Skip this store"),
                    SKIP_STORE_SELECTION,
                    selected_index_opt,
                    Message::SearchResultSelected,
                )
                .spacing(5),
            );
            for (index, result) in current_results.iter().enumerate() {
                search_list = search_list.push(
                    Radio::new(
                        result.title().to_string(),
                        index,
                        selected_index_opt,
                        Message::SearchResultSelected,
                    )
                    .spacing(5),
                );
            }
        }

        // let selected_text = match app.selected_results_by_store.get(current_store_id) {
        //     Some(Some(idx)) => current_results.get(*idx).map(|result| result.title().to_string()).unwrap_or_else(|| String::from("None")),
        //     Some(None) => String::from("Skipped"),
        //     None => String::from("None"),
        // };

        let add_reqs_meet = app.search_results_by_store.iter().all(|(id, _)| app.selected_results_by_store.contains_key(id)) && !app.add_price.is_empty();
        let add_threshold_button = if add_reqs_meet {
            Button::new(text("Add Threshold"))
                .on_press(Message::AddThreshold)
                .padding(8)
        } else {
            Button::new(text("Add Threshold"))
                .padding(8)
        };

        column![
            row![
                text(progress).size(14),
                if app.current_store_search_idx > 0 {
                    Button::new(text("←").center())
                        .on_press(Message::PreviousStore)
                        .height(14)
                        .padding(4)
                } else {
                    Button::new(text("←").center())
                        .height(14)
                        .padding(4)
                },
                if app.current_store_search_idx < app.search_results_by_store.len() - 1 {
                    Button::new(text("→").center())
                        .on_press(Message::NextStore)
                        .height(14)
                        .padding(4)
                } else {
                    Button::new(text("→").center())
                        .height(14)
                        .padding(4)
                },
            ],
            Scrollable::new(search_list).height(400).width(Length::Fill),
            column![
                // text(format!("Selected: {}", selected_text)).size(14),
                add_threshold_button,
            ]
            .spacing(10),
        ]
    };

    let search_controls = column![
        row![
            TextInput::new("Search games", &app.search_query)
                .on_input(Message::SearchQueryChanged)
                .padding(5)
                .width(Length::Fill),
            Button::new(text("Search")).on_press(Message::StartSearch).padding(6),
        ]
        .spacing(8),
        if settings::get_alias_state() {
            row![
                TextInput::new("Add alias for game", &app.add_alias)
                    .on_input(|value| Message::ThresholdAliasChanged(usize::MAX, value))
                    .padding(5)
                    .width(Length::Fixed(400.0)),
                TextInput::new("Desired price", &app.add_price)
                    .on_input(|value| Message::ThresholdPriceChanged(usize::MAX, value))
                    .padding(5)
                    .width(Length::Fixed(150.0)),
            ]
        } else {
            row![
                TextInput::new("Desired price", &app.add_price)
                    .on_input(|value| Message::ThresholdPriceChanged(usize::MAX, value))
                    .padding(5)
                    .width(Length::Fixed(150.0)),
            ]
        },
        store_section,
    ]
    .spacing(10)
    .padding(8);

    let combined = column![search_controls].spacing(12).width(Length::Fill);

    container(combined).width(Length::Fill).center_x(Length::Fill).into()
}