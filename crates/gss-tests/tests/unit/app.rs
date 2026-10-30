use types::internal::data::{GameThreshold, SimpleGameThreshold};
use types::internal::store::GameStore;

use gss_app::app::enums::{SortColumn, SortOrder, StoreSearchResult, Tab, View};
use gss_app::app::message::MainMessage;
use gss_app::app::state::App;
use gss_app::views::preview::PreviewMessage;
use gss_app::views::settings::Page;
use gss_tests::utils::tmp_setup;

const TMP_DIR_TITLE: &str = "gss_app";

fn is_message_in_batch(app: &App, log_msg: &str) -> bool {
    let mut is_present = false;
    for log in app.logger.get_batch_ref() {
        if log.contains(log_msg) {
            is_present = true;
            break;
        }
    }
    is_present
}

pub mod search {
    use super::*;

    #[test]
    fn store_search_result_ids_are_split_by_store() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let steam = StoreSearchResult::Steam {
            title: "Portal".into(),
            steam_id: 400,
        };
        let gog = StoreSearchResult::Gog {
            title: "Portal".into(),
            gog_id: 500,
        };
        let ms = StoreSearchResult::Microsoft {
            title: "Portal".into(),
            ms_id: "MS-123".into(),
        };

        assert_eq!(steam.ids(), (400, 0, String::new()));
        assert_eq!(gog.ids(), (0, 500, String::new()));
        assert_eq!(ms.ids(), (0, 0, "MS-123".to_string()));

        tmp_env.tear_down();
    }

    #[test]
    fn start_search_tracks_selected_stores_and_progress() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.selected_stores = vec![GameStore::STEAM, GameStore::GOOD_OLD_GAMES];
        app.0.search_query = "Portal 2".into();

        let _ = app.0.update(MainMessage::StartSearch.into());

        assert!(app.0.is_search_in_progress);
        assert_eq!(app.0.pending_searches, 2);
        assert_eq!(app.0.search_results_by_store.len(), 2);
        assert_eq!(app.0.search_results_by_store[0].0, GameStore::STEAM);
        assert_eq!(
            app.0.search_results_by_store[1].0,
            GameStore::GOOD_OLD_GAMES
        );
        assert!(
            app.0
                .search_results_by_store
                .iter()
                .all(|(_, results)| results.is_empty())
        );

        tmp_env.tear_down();
    }

    #[test]
    fn check_price_error_shows_dialog() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        let _ = app.0.update(
            PreviewMessage::GetSalesUpdated(Err("Could not find sales data".to_string())).into(),
        );
        assert!(app.0.preview_view.show_dialog);
        assert_eq!(
            app.0.preview_view.message_details,
            "An issue occurred while looking for game sales. Please check your internet connection or try again later."
        );

        tmp_env.tear_down();
    }

    #[test]
    fn selecting_result_without_auto_advance() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.auto_advance_enabled = false;
        app.0.current_store_search_idx = 1;
        app.0.search_results_by_store = vec![
            (
                GameStore::STEAM,
                vec![StoreSearchResult::Steam {
                    title: "Portal".into(),
                    steam_id: 400,
                }],
            ),
            (
                GameStore::GOOD_OLD_GAMES,
                vec![StoreSearchResult::Gog {
                    title: "Portal".into(),
                    gog_id: 500,
                }],
            ),
        ];

        let _ = app.0.update(MainMessage::SearchResultSelected(0).into());

        assert_eq!(
            app.0
                .selected_results_by_store
                .get(&GameStore::GOOD_OLD_GAMES),
            Some(&Some(0))
        );
        assert_eq!(app.0.current_store_search_idx, 1);

        tmp_env.tear_down();
    }

    #[test]
    fn selecting_result_with_auto_advance() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.auto_advance_enabled = true;
        app.0.current_store_search_idx = 0;
        app.0.search_results_by_store = vec![
            (
                GameStore::STEAM,
                vec![StoreSearchResult::Steam {
                    title: "Portal".into(),
                    steam_id: 400,
                }],
            ),
            (GameStore::GOOD_OLD_GAMES, Vec::new()),
        ];

        let _ = app.0.update(MainMessage::SearchResultSelected(0).into());

        assert_eq!(
            app.0.selected_results_by_store.get(&GameStore::STEAM),
            Some(&Some(0))
        );
        assert_eq!(app.0.current_store_search_idx, 0);

        tmp_env.tear_down();
    }
}

pub mod bulk_insert {
    use super::*;

    #[test]
    fn advance_to_next_game() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.bulk_simple_threshs = vec![
            SimpleGameThreshold {
                name: "Alpha".into(),
                price: 10.0,
            },
            SimpleGameThreshold {
                name: "Beta".into(),
                price: 20.0,
            },
        ];
        app.0.bulk_search_index = 0;

        let next_game = app.0.next_bulk_game();

        assert_eq!(next_game.map(|game| game.name), Some("Beta".into()));

        tmp_env.tear_down();
    }

    #[test]
    fn return_none_when_done() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.bulk_simple_threshs = vec![SimpleGameThreshold {
            name: "Alpha".into(),
            price: 10.0,
        }];
        app.0.bulk_search_index = 1;

        let next_game = app.0.next_bulk_game();

        assert!(next_game.is_none());

        tmp_env.tear_down();
    }

    #[test]
    fn returns_current_game() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.bulk_simple_threshs = vec![
            SimpleGameThreshold {
                name: String::from("Alpha"),
                price: 10.0,
            },
            SimpleGameThreshold {
                name: String::from("Beta"),
                price: 20.0,
            },
        ];
        app.0.bulk_search_index = 1;

        let current_game = app.0.current_bulk_game();

        assert_eq!(current_game.map(|game| game.name), Some("Beta".to_string()));

        tmp_env.tear_down();
    }

    #[test]
    fn get_next_search_from_csv() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.selected_stores = vec![GameStore::STEAM];
        app.0.bulk_simple_threshs = vec![
            SimpleGameThreshold {
                name: "Alpha".into(),
                price: 9.99,
            },
            SimpleGameThreshold {
                name: "Beta".into(),
                price: 14.5,
            },
        ];
        app.0.bulk_search_index = 0;
        app.0.bulk_search_used = true;

        let _ = app.0.update(MainMessage::ExecuteBulkInsert.into());

        assert_eq!(app.0.search_query, "Alpha");
        assert_eq!(app.0.add_price, "9.99");
        assert!(app.0.is_search_in_progress);
        assert_eq!(app.0.pending_searches, 1);
        assert_eq!(app.0.search_results_by_store.len(), 1);
        assert_eq!(app.0.search_results_by_store[0].0, GameStore::STEAM);

        tmp_env.tear_down();
    }

    #[test]
    fn no_games_loaded_from_csv() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.bulk_simple_threshs.clear();
        app.0.bulk_search_used = false;

        let _ = app.0.update(MainMessage::ExecuteBulkInsert.into());

        assert!(!app.0.is_search_in_progress);
        assert!(app.0.search_query.is_empty());
        assert!(
            is_message_in_batch(&app.0, "No games loaded from CSV."),
            "The expected warning could not be found in the batch {:?}",
            app.0.logger.get_batch_ref()
        );

        tmp_env.tear_down();
    }
}

pub mod thresholds {
    use super::*;

    #[test]
    fn insert() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut thresholds_list = Vec::new();

        let threshold_added = App::insert_threshold(
            &mut thresholds_list,
            "Example Game",
            "Example Alias",
            9.99,
            12345,
            0,
            "".to_string(),
        );

        assert!(threshold_added);
        assert_eq!(thresholds_list.len(), 1);
        assert_eq!(thresholds_list[0].title, "Example Game");
        assert_eq!(thresholds_list[0].alias, "Example Alias");
        assert_eq!(thresholds_list[0].desired_price, 9.99);
        assert_eq!(thresholds_list[0].steam_id, 12345);

        tmp_env.tear_down();
    }

    #[test]
    fn update() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut thresholds_list = vec![GameThreshold {
            title: "Example Game".into(),
            alias: "Old Alias".into(),
            steam_id: 100,
            gog_id: 0,
            microsoft_store_id: String::new(),
            currency: String::from("USD"),
            desired_price: 19.99,
        }];

        let threshold_added = App::insert_threshold(
            &mut thresholds_list,
            "Example Game",
            "New Alias",
            8.99,
            200,
            10,
            "MS123".to_string(),
        );

        assert!(!threshold_added);
        assert_eq!(thresholds_list.len(), 1);
        assert_eq!(thresholds_list[0].alias, "New Alias");
        assert_eq!(thresholds_list[0].desired_price, 8.99);
        assert_eq!(thresholds_list[0].steam_id, 200);
        assert_eq!(thresholds_list[0].gog_id, 10);
        assert_eq!(thresholds_list[0].microsoft_store_id, "MS123");

        tmp_env.tear_down();
    }

    #[test]
    fn sync_to_alias_and_price_edits() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.thresholds = vec![
            GameThreshold {
                title: "Example Game".into(),
                alias: "Alias1".into(),
                steam_id: 0,
                gog_id: 0,
                microsoft_store_id: String::new(),
                currency: String::from("USD"),
                desired_price: 5.5,
            },
            GameThreshold {
                title: "Example 2".into(),
                alias: String::new(),
                steam_id: 0,
                gog_id: 0,
                microsoft_store_id: String::new(),
                currency: String::from("USD"),
                desired_price: 12.0,
            },
        ];

        app.0.sync_threshold_edits();

        assert_eq!(app.0.threshold_alias_edits, vec!["Alias1", ""]);
        assert_eq!(app.0.threshold_price_edits, vec!["5.5", "12"]);

        tmp_env.tear_down();
    }

    #[test]
    fn sort_column_order_cycle() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.threshold_sort_column = None;
        app.0.threshold_sort_order = SortOrder::Original;

        let _ = app
            .0
            .update(MainMessage::SortThresholds(SortColumn::Title).into());
        assert_eq!(app.0.threshold_sort_column, Some(SortColumn::Title));
        assert_eq!(app.0.threshold_sort_order, SortOrder::Ascending);

        let _ = app
            .0
            .update(MainMessage::SortThresholds(SortColumn::Title).into());
        assert_eq!(app.0.threshold_sort_order, SortOrder::Descending);

        let _ = app
            .0
            .update(MainMessage::SortThresholds(SortColumn::Title).into());
        assert_eq!(app.0.threshold_sort_column, None);
        assert_eq!(app.0.threshold_sort_order, SortOrder::Original);

        tmp_env.tear_down();
    }

    #[test]
    fn threshold_edits_capture_user_changes() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.threshold_alias_edits = vec!["Original Alias".into()];
        app.0.threshold_price_edits = vec!["19.99".into()];

        let _ = app
            .0
            .update(MainMessage::ThresholdAliasChanged(0, "Updated Alias".into()).into());
        let _ = app
            .0
            .update(MainMessage::ThresholdPriceChanged(0, "12.5".into()).into());

        assert_eq!(app.0.threshold_alias_edits[0], "Updated Alias");
        assert_eq!(app.0.threshold_price_edits[0], "12.5");

        tmp_env.tear_down();
    }

    #[test]
    fn alias_and_price_changes_for_new_entries_saved_in_buffer() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());

        let _ = app.0.update(
            MainMessage::ThresholdAliasChanged(usize::MAX, "Bulk Alias".to_string()).into(),
        );
        let _ = app
            .0
            .update(MainMessage::ThresholdPriceChanged(usize::MAX, "12.5".to_string()).into());

        assert_eq!(app.0.add_alias, "Bulk Alias");
        assert_eq!(app.0.add_price, "12.5");

        tmp_env.tear_down();
    }
}

pub mod caching {
    use super::*;

    #[test]
    fn update_cache_error_shows_dialog() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        let _ = app
            .0
            .update(MainMessage::UpdateCacheResult(Err("cache update failed".to_string())).into());
        assert!(app.0.show_dialog);
        assert_eq!(
            app.0.message_details,
            "An issue occurred trying to cache game titles. Please check your internet connection or try again later."
        );

        tmp_env.tear_down();
    }
}

pub mod logging {
    use super::*;

    #[test]
    fn log_start_search_with_no_stores() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.selected_stores.clear();
        app.0.search_query = "Example".into();

        let _ = app.0.update(MainMessage::StartSearch.into());

        assert!(!app.0.is_search_in_progress);
        assert!(
            is_message_in_batch(&app.0, "No stores to search"),
            "The expected log could not be found in the batch {:?}",
            app.0.logger.get_batch_ref()
        );
        assert!(app.0.search_results_by_store.is_empty());

        tmp_env.tear_down();
    }

    #[test]
    fn logs_start_search_with_no_query() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.search_query.clear();

        let _ = app.0.update(MainMessage::StartSearch.into());

        assert!(!app.0.is_search_in_progress);
        assert!(
            is_message_in_batch(&app.0, "Search query was empty so no game could be found."),
            "The expected log could not be found in the batch {:?}",
            app.0.logger.get_batch_ref()
        );

        tmp_env.tear_down();
    }
}

pub mod interface {
    use super::*;

    #[test]
    fn tab_labels_match_expected_values() {
        assert_eq!(Tab::Search.label(), "Search");
        assert_eq!(Tab::Thresholds.label(), "Thresholds");
    }

    #[test]
    fn opening_and_closing_settings_page() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());

        let _ = app.0.update(MainMessage::OpenSettings.into());
        assert_eq!(app.0.active_view, View::Settings);
        assert!(app.0.view_bar.contains(&View::Settings));
        assert_eq!(app.0.settings_page, Page::General);

        let _ = app.0.update(MainMessage::CloseSettings.into());
        assert_eq!(app.0.active_view, View::Base);
        assert!(!app.0.view_bar.contains(&View::Settings));

        tmp_env.tear_down();
    }

    #[test]
    fn opening_and_closing_logging_page() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());

        let _ = app.0.update(MainMessage::OpenLogsView.into());
        assert_eq!(app.0.active_view, View::Logs);
        assert!(app.0.view_bar.contains(&View::Logs));

        let _ = app.0.update(MainMessage::CloseLogsView.into());
        assert_eq!(app.0.active_view, View::Base);
        assert!(!app.0.view_bar.contains(&View::Logs));

        tmp_env.tear_down();
    }

    #[test]
    fn opening_and_closing_sales_preview_page() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());

        let _ = app.0.update(MainMessage::OpenSalesPreview.into());
        assert_eq!(app.0.active_view, View::Preview);
        assert!(app.0.view_bar.contains(&View::Preview));

        let _ = app.0.update(MainMessage::CloseSalesPreview.into());
        assert_eq!(app.0.active_view, View::Base);
        assert!(!app.0.view_bar.contains(&View::Preview));

        tmp_env.tear_down();
    }

    #[test]
    fn opens_email_settings() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());

        let _ = app.0.update(MainMessage::PageSelected(Page::Email).into());

        assert_eq!(app.0.active_view, View::Settings);
        assert!(app.0.view_bar.contains(&View::Settings));
        assert_eq!(app.0.settings_page, Page::Email);

        tmp_env.tear_down();
    }

    // Fix this test (also make these tests not update the actual settings, config or thresholds files)
    #[test]
    fn toggle_store_options() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.available_stores = vec![GameStore::STEAM, GameStore::GOOD_OLD_GAMES];

        let _ = app
            .0
            .update(MainMessage::ToggleStore(GameStore::STEAM, true).into());
        let _ = app
            .0
            .update(MainMessage::ToggleStore(GameStore::GOOD_OLD_GAMES, true).into());
        let _ = app
            .0
            .update(MainMessage::ToggleStore(GameStore::STEAM, false).into());
        // let _ = app.0.

        assert_eq!(app.0.selected_stores, vec![GameStore::GOOD_OLD_GAMES]);

        tmp_env.tear_down();
    }

    #[test]
    fn hide_dialog_clears_error_state() {
        let tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
        let _ = properties::load_properties();
        properties::set_test_mode(true);

        let mut app = App::new(String::new());
        app.0.show_dialog = true;
        app.0.status_message = "ERROR".into();
        app.0.message_details = "Something went wrong".into();

        let _ = app.0.update(MainMessage::HideDialog.into());

        assert!(!app.0.show_dialog);
        assert!(app.0.status_message.is_empty());
        assert!(app.0.message_details.is_empty());

        tmp_env.tear_down();
    }
}
