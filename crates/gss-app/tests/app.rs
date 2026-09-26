use types::internal::data::{GameThreshold, SimpleGameThreshold};
use types::internal::store::GameStore;

use gss_app::app::enums::{SortColumn, SortOrder, StoreSearchResult, Tab, View};
use gss_app::app::message::MainMessage;
use gss_app::app::state::App;
use gss_app::views::preview::PreviewMessage;
use gss_app::views::settings::Page;

#[cfg(test)]
pub mod unit {
    use super::*;

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

    #[test]
    fn bulk_search_advances_to_next_game() {
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
    }

    #[test]
    fn bulk_search_return_none_when_done() {
        let mut app = App::new(String::new());
        app.0.bulk_simple_threshs = vec![SimpleGameThreshold {
            name: "Alpha".into(),
            price: 10.0,
        }];
        app.0.bulk_search_index = 1;

        let next_game = app.0.next_bulk_game();

        assert!(next_game.is_none());
    }

    #[test]
    fn bulk_search_returns_current_game() {
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
    }

    #[test]
    fn insert_threshold() {
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
    }

    #[test]
    fn update_existing_threshold() {
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
    }

    #[test]
    fn thresholds_sync_to_alias_and_price_edits() {
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
    }

    #[test]
    fn sort_thresholds_column_order_cycle() {
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
    }

    #[test]
    fn log_start_search_with_no_stores() {
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
    }

    #[test]
    fn logs_start_search_with_no_query() {
        let mut app = App::new(String::new());
        app.0.search_query.clear();

        let _ = app.0.update(MainMessage::StartSearch.into());

        assert!(!app.0.is_search_in_progress);
        assert!(
            is_message_in_batch(&app.0, "Search query was empty so no game could be found."),
            "The expected log could not be found in the batch {:?}",
            app.0.logger.get_batch_ref()
        );
    }

    #[test]
    fn update_cache_error_shows_dialog() {
        let mut app = App::new(String::new());
        let _ = app
            .0
            .update(MainMessage::UpdateCacheResult(Err("cache update failed".to_string())).into());
        assert!(app.0.show_dialog);
        assert_eq!(
            app.0.message_details,
            "An issue occurred trying to cache game titles. Please check your internet connection or try again later."
        );
    }

    #[test]
    fn check_price_error_shows_dialog() {
        let mut app = App::new(String::new());
        let _ = app.0.update(
            PreviewMessage::GetSalesUpdated(Err("Could not find sales data".to_string())).into(),
        );
        assert!(app.0.preview_view.show_dialog);
        assert_eq!(
            app.0.preview_view.message_details,
            "An issue occurred while looking for game sales. Please check your internet connection or try again later."
        );
    }

    #[test]
    fn tab_labels_match_expected_values() {
        assert_eq!(Tab::Search.label(), "Search");
        assert_eq!(Tab::Thresholds.label(), "Thresholds");
    }

    #[test]
    fn store_search_result_ids_are_split_by_store() {
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
    }

    #[test]
    fn opening_and_closing_settings_updates_view_stack() {
        let mut app = App::new(String::new());

        let _ = app.0.update(MainMessage::OpenSettings.into());
        assert_eq!(app.0.active_view, View::Settings);
        assert!(app.0.view_bar.contains(&View::Settings));
        assert_eq!(app.0.settings_page, Page::General);

        let _ = app.0.update(MainMessage::CloseSettings.into());
        assert_eq!(app.0.active_view, View::Base);
        assert!(!app.0.view_bar.contains(&View::Settings));
    }

    #[test]
    fn opening_and_closing_logs_updates_view_stack() {
        let mut app = App::new(String::new());

        let _ = app.0.update(MainMessage::OpenLogsView.into());
        assert_eq!(app.0.active_view, View::Logs);
        assert!(app.0.view_bar.contains(&View::Logs));

        let _ = app.0.update(MainMessage::CloseLogsView.into());
        assert_eq!(app.0.active_view, View::Base);
        assert!(!app.0.view_bar.contains(&View::Logs));
    }

    #[test]
    fn start_search_tracks_selected_stores_and_progress() {
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
    }
}
