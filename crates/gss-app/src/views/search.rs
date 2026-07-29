use iced::widget::{Button, Radio, Scrollable, TextInput, column, container, row, text};
use iced::{Element, Length};

use file_ops::{settings, thresholds};

use crate::{LOADING_FRAMES_SIZE, Message};
use crate::components::custom_widgets;

pub const SKIP_STORE_SELECTION: usize = usize::MAX;

pub fn search_tab(app: &crate::App) -> Element<'_, Message> {
    let current_store_idx: usize = if app.pending_searches > 0 {
        app.selected_stores.len()-app.pending_searches+1
    } else { 0 };

    let search_index = format!("Searching Stores({}/{})", 
        current_store_idx, 
        app.selected_stores.len()
    );

    let search_loading = custom_widgets::text_loading_indicator(&search_index, app.search_loading_frame, LOADING_FRAMES_SIZE);

    let store_selection = if app.selected_stores.is_empty() {
        column![text("No stores selected. Please go to 'Settings'.")]
    } else if app.search_query.is_empty() && app.search_results_by_store.is_empty() {
        column![text("No search in progress. Enter a query and press Search.")]
    } else if !app.alias_reuse_enabled && thresholds::does_alias_exist(&app.add_alias) {
        //app.log_batch.push_str(&log_utils::message_builder("L", log_utils::LogLevel::WARN));
        column![text(format!("The alias \'{}\' is already is use. Please enable alias reuse in Settings to use this alias again.", &app.add_alias))]
    } else if app.search_results_by_store.len() == 0 {
        column![text("")]
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
                ).width(Length::Fill)
            ].spacing(5);
        } else {
            search_list = search_list.push(
                Radio::new(
                    String::from("Skip this store"),
                    SKIP_STORE_SELECTION,
                    selected_index_opt,
                    Message::SearchResultSelected,
                )
                .width(Length::Fill)
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
                    .width(Length::Fill)
                    .spacing(5),
                );
            }
        }

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
                text(progress).size(18),
                if app.current_store_search_idx > 0 {
                    Button::new(text("←").center())
                        .on_press(Message::PreviousStore)
                        .height(18)
                        .padding(4)
                } else {
                    Button::new(text("←").center())
                        .height(18)
                        .padding(4)
                },
                if app.current_store_search_idx < app.search_results_by_store.len() - 1 {
                    Button::new(text("→").center())
                        .on_press(Message::NextStore)
                        .height(18)
                        .padding(4)
                } else {
                    Button::new(text("→").center())
                        .height(18)
                        .padding(4)
                },
            ],
            Scrollable::new(search_list).height(400).width(Length::Fill),
            column![
                add_threshold_button,
            ]
            .padding(8)
            .spacing(10),
        ]
    };
    
    let bulk_insert_button = if app.bulk_simple_threshs.is_empty() {
        let mut button = Button::new(text("Load Multiple Searches")).padding(6);
        if !app.is_search_in_progress {
            button = button.on_press(Message::OpenCsv);
        }
        button
    } else {
        let mut button = Button::new(text("Search All")).padding(6);
        if !app.is_search_in_progress {
            button = button.on_press(Message::ExecuteBulkInsert);
        }
        button
    };

    let search_button = if app.bulk_search_used {
        Button::new(text("Search")).padding(6)
    } else {
        let mut button = Button::new(text("Search")).padding(6);
        if !app.is_search_in_progress {
            button = button.on_press(Message::StartSearch);
        }
        button
    };

    let reset_button = if !app.is_search_in_progress {
        Button::new(text("Reset"))
            .on_press(Message::SearchReset)
            .padding(6)
    } else {
        Button::new(text("Reset"))
            .padding(6)
    };

    let search_controls = column![
        row![
            TextInput::new("Search games", &app.search_query)
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::StartSearch)
                .padding(5)
                .width(Length::Fill),
            search_button,
            reset_button
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
                bulk_insert_button
            ].width(Length::Fill)
            .spacing(20)
        } else {
            row![
                TextInput::new("Desired price", &app.add_price)
                    .on_input(|value| Message::ThresholdPriceChanged(usize::MAX, value))
                    .padding(5)
                    .width(Length::Fixed(150.0)),
                bulk_insert_button
            ].spacing(20)
        },
        if app.is_search_in_progress {
            column![search_loading]
        } else {
            store_selection
        }        
    ]
    .spacing(10)
    .padding(8);

    let combined = column![search_controls].spacing(12).width(Length::Fill);

    container(combined).width(Length::Fill).center_x(Length::Fill).into()
}