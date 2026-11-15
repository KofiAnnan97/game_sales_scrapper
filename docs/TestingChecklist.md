# Testing Checklist

This checklist covers the `gss-cli` and `gss-app` executables. Status text uses
dark green (`#006400`) for automated coverage and dark orange (`#a34000`) for
manual checks. Each label is highlighted with a lighter green or peach
background. Automated coverage is identified by its `gss-tests` test name;
manual entries list the steps needed to verify the functionality.

For the automated test suite run the following:
```sh
cargo test -p gss-tests
```

For manual checks run the executables and go through the flow for validation
```sh
# CLI
cargo run -p gss-cli
# App
cargo run -p gss-app
```

## Command Line (`gss-cli`)

### Command parsing and configuration

| Description | Manual/Automated | Notes |
|---|---|---|
| 1. Check top-level and subcommand help. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `cargo run -p gss-cli -- --help`, `cargo run -p gss-cli -- config --help`, and each subcommand's `--help`. |
| 2. Check required and invalid arguments. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Try omitting required arguments and supplying invalid values. |
| 3. Check conflicting flags. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli` with conflicting top-level flags and verify argument validation. |
| 4. Configure selected stores. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::config_cmd`; covers selecting Steam and GOG. |
| 5. Configure alias enablement. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::config_cmd` checks aliases disabled. |
| 6. Select all stores. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli config settings --all_stores` and confirm all stores are persisted. |
| 7. Enable alias reuse. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli config settings --allow_alias_reuse 1` and confirm the setting is persisted. |
| 8. Check invalid alias setting values. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Try values other than 0 or 1 for alias enablement/reuse. |
| 9. Check store mapping and selected-store persistence. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/settings.rs`: `get_available_stores`, `get_selected_stores`, `update_selected_stores`. |
| 10. Check alias state. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/settings.rs`: `get_alias_state`, `update_alias_state`. |
| 11. Check store names. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/settings.rs::get_proper_store_name`. |
| 12. Check property file creation and loading. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/properties.rs`. |
| 13. Check environment-variable handling. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/env_vars.rs`; `.env` update coverage is in `properties.rs::update_properties_from_env`. |
| 14. Check SMTP configuration prompts and recipient. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli config properties --set-smtp` and `--set-recipient` with disposable test values; verify saved values. |
| 15. Check Steam API key and project/test paths. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run with `--set-api-key`, `--set-project-path`, and `--set-test-path`; verify saved values. |
| 16. Check CLI `.env` import. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli config properties --from-env` with a disposable `.env` file and verify imported values. |
| 17. Check listing masked secrets. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli config properties --list-properties`; confirm secrets are hidden. |
| 18. Check secret reveal confirmation and key flow. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run with `--reveal-secrets`; verify confirmation and decryption-key checks. |

### Threshold

| Description | Manual/Automated | Notes |
|---|---|---|
| 1. Add a game in each configured store. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Configure stores, then run `gss-cli add --title "<game>" --price <price>` and follow prompts. `add_cmd` is ignored as not yet implemented. |
| 2. Check alias handling and result selection. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | During `gss-cli add`, select a store result and enter an alias; inspect saved thresholds. |
| 3. Check skip and invalid input. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | During `gss-cli add`, skip a result and enter an invalid selection; verify behavior. |
| 4. Verify saved title, store ID, alias, and desired price. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Inspect threshold data after completing `gss-cli add`. |
| 5. Bulk-insert a valid CSV. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli bulk-insert --file <csv-path>` and follow prompts. `bulk_insert_cmd` is ignored as not yet implemented. |
| 6. Bulk-insert empty and malformed CSV files. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli bulk-insert --file <csv-path>` with empty and malformed files; verify errors and saved data. |
| 7. Verify bulk-insert store selection and saved thresholds. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Complete a bulk insert and inspect thresholds. |
| 8. Update a threshold price by title. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::update_price_cmd`. |
| 9. Update a threshold price by fuzzy title. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::update_price_cmd`. |
| 10. Update a threshold price by alias. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::update_price_cmd`. |
| 11. Update an alias by title. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::update_alias_cmd`. |
| 12. Update an alias by fuzzy title. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::update_alias_cmd`. |
| 13. Remove a threshold by title. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::remove_cmd`. |
| 14. Remove a threshold by fuzzy title. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::remove_cmd`. |
| 15. Remove a threshold by alias. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::remove_cmd`. |
| 16. Add per-store thresholds. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/thresholds.rs`. |
| 17. Update threshold aliases, prices, and store IDs. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/thresholds.rs`. |
| 18. Remove thresholds and update alias maps. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/thresholds.rs`. |
| 19. List thresholds. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::list_thresholds_cmd`. |
| 20. List selected stores. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::list_selected_stores_cmd`. |

### Price checks, cache, and email

| Description | Manual/Automated | Notes |
|---|---|---|
| 1. Check a threshold against Steam results. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::check_prices`; live store request, not deterministic mock coverage. |
| 2. Check a threshold against GOG results. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::check_prices`; live store request, not deterministic mock coverage. |
| 3. Check a threshold against Microsoft Store results. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::check_prices`; live store request, not deterministic mock coverage. |
| 4. Check Steam store search and price API calls. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/api/steam.rs`; API-level tests do not cover the full CLI workflow. |
| 5. Check GOG store search and price API calls. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/api/gog.rs`; API-level tests do not cover the full CLI workflow. |
| 6. Check Microsoft Store search and price API calls. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/api/microsoft_store.rs`; API-level tests do not cover the full CLI workflow. |
| 7. Check `--check-prices` matching and non-matching prices. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Configure thresholds/stores and run `gss-cli --check-prices` with sale and no-sale threshold cases. |
| 8. Check `--check-prices` no-sale output. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli --check-prices` with thresholds that are not on sale. |
| 9. Check `--check-prices` store error handling. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli --check-prices` when a store request fails; verify error reporting. |
| 10. Check cache update success. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli --update-cache` online and verify the cache updates. |
| 11. Check cache update failure reporting. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli --update-cache` without network access or with an unavailable service. |
| 12. Check `--send-email` with valid settings and a sale. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Configure disposable email/SMTP settings and run `gss-cli --send-email` with a threshold that is on sale. |
| 13. Check `--send-email` no-sale behavior. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli --send-email` with no matching sales; verify no email is sent. |
| 14. Check invalid email settings and delivery failure reporting. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli --send-email` with invalid settings or unavailable mail service. |
| 15. Verify generated email content and delivery. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Use disposable SMTP settings, run `gss-cli --send-email`, then verify the message and delivery. No automated CLI email test exists. |

## Application (`gss-app`)

### Startup, navigation, and search

| Description | Manual/Automated | Notes |
|---|---|---|
| 1. Check app startup and main window sizing/resizing. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `cargo run -p gss-app` and exercise the main window. |
| 2. Check the initial Search tab. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Launch with `cargo run -p gss-app`; verify Search is initially selected. |
| 3. Check Search/Thresholds navigation. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Switch between the Search and Thresholds tabs. |
| 4. Check clean close. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Close the app normally and verify it exits cleanly. |
| 5. Navigate to/from Settings, Logs, and Sales Preview. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs`: `opening_and_closing_settings_page`, `opening_and_closing_logging_page`, `opening_and_closing_sales_preview_page`. |
| 6. Select settings pages. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::interface::opens_email_settings`. |
| 7. Dismiss dialogs. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::interface::hide_dialog_clears_error_state`. |
| 8. Toggle stores. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::interface::toggle_store_options`. |
| 9. Check selected stores and pending-search tracking. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::search::start_search_tracks_selected_stores_and_progress`. |
| 10. Check per-store IDs and result selection. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs`: `store_search_result_ids_are_split_by_store`, `selecting_result_without_auto_advance`. |
| 11. Check auto-advance. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::search::selecting_result_with_auto_advance`. |
| 12. Check empty query/no stores and search-error dialog state. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs`: `logging::logs_start_search_with_no_query`, `logging::log_start_search_with_no_stores`, `search::check_price_error_shows_dialog`. |
| 13. Search live stores and check progress/results. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | In Search, enter a game title, select stores, click Search, and verify progress/results. |
| 14. Check no results and store failures. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Search for an unmatched title and repeat with unavailable network. |
| 15. Check previous/next navigation and skipping. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Navigate between stores/results and select Skip for a store. |
| 16. Check manual result selection and reset. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Select a result manually, then reset the search. |
| 17. Add selected search results as thresholds. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Search for a game, select results, enter alias/price, and add the threshold. |
| 18. Check price validation and aliases. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Add a threshold with a valid alias/price, then retry with an invalid price. |
| 19. Check matching titles across stores and alias reuse. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Select matching results from multiple stores and try a reused alias. |
| 20. Verify threshold persistence after refresh/restart. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Refresh or restart the app and confirm the threshold remains. |

### CSV import and thresholds

| Description | Manual/Automated | Notes |
|---|---|---|
| 1. Check bulk-search current/next item tracking. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs` (`bulk_insert` tests). |
| 2. Check empty bulk-search input. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::bulk_insert::no_games_loaded_from_csv`. |
| 3. Check starting the next CSV game. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::bulk_insert::get_next_search_from_csv`. |
| 4. Select valid CSV files. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | In Search, choose **Load Multiple Searches** and select a valid CSV. |
| 5. Select empty and malformed CSV files. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Repeat **Load Multiple Searches** with empty and malformed files; verify handling. |
| 6. Run the multi-game search workflow. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run **Search All**, select results, and add thresholds for imported games. |
| 7. Verify CSV workflow progression and completion/errors. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Confirm the workflow advances through entries and reports completion/errors. |
| 8. Check threshold insertion and update. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::thresholds::insert` and `thresholds::update`. |
| 9. Check threshold edit-buffer synchronization and edit capture. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs`: `sync_to_alias_and_price_edits`, `threshold_edits_capture_user_changes`, `alias_and_price_changes_for_new_entries_saved_in_buffer`. |
| 10. Check sort-order cycling. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::thresholds::sort_column_order_cycle`. |
| 11. Check threshold filtering and refresh. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | In Thresholds, filter by title/alias and refresh the list. |
| 12. Check threshold sorting. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Cycle all sortable columns and sort directions. |
| 13. Save alias/price edits and try invalid prices. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Edit a row's alias and price, save, and repeat with an invalid price. |
| 14. Remove a row and check empty-list display. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Remove a threshold; verify the row and empty-list state. |
| 15. Verify threshold persistence. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Restart the app and confirm threshold changes persist. |

### Settings, sales, cache, and logs

| Description | Manual/Automated | Notes |
|---|---|---|
| 1. Check settings navigation. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::interface::opening_and_closing_settings_page`. |
| 2. Check store-selection state. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::interface::toggle_store_options`. |
| 3. Check cache failure dialog state. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::caching::update_cache_error_shows_dialog`. |
| 4. Save General settings. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Open Settings; update project/test paths, test mode, stores, and alias options; save. |
| 5. Save Email settings. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Update email/SMTP fields in Settings and save. |
| 6. Save Steam settings. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Update the Steam API key in Settings and save. |
| 7. Verify settings persistence and masked secrets. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Close/reopen Settings or restart; confirm saved values and secret masking. |
| 8. Check app cache update success. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | In Settings, trigger cache update online and verify progress/success. |
| 9. Check app cache update network-error behavior. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Trigger cache update with network unavailable and verify error reporting. |
| 10. Check sales lookup error dialog. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::search::check_price_error_shows_dialog`. |
| 11. Check Sales Preview lookup and refresh. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Open Sales Preview and run its initial lookup and refresh. |
| 12. Check empty and populated results. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Verify Sales Preview with no matching sales and with populated results. |
| 13. Check filters, sorting, and reset. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Exercise store/price/custom-range filters, sort, apply, and reset. |
| 14. Check comparison and email preview. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Compare sales and inspect the email preview. |
| 15. Check link copying and closing/reopening. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Copy a game link, verify clipboard/feedback, then close and reopen preview. |
| 16. Send email from the preview with valid settings. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Use **Send Manual Email** with valid disposable SMTP settings; verify delivery and success feedback. |
| 17. Send email with invalid settings or unavailable network. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Use invalid settings or unavailable network; verify error feedback. |
| 18. Check opening the Logs view. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::interface::opening_and_closing_logging_page`. |
| 19. Check closing the Logs view. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/app.rs::interface::opening_and_closing_logging_page`. |
| 20. Check log refresh and level/screen/file filters. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Open Logs; refresh and exercise level, screen, and file filters. |
| 21. Check paging and current/historical logs. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Page through current and historical log entries. |
| 22. Check manual pruning and selection constraints. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Open Manual Prune, select historical logs, and verify current-log selection constraints. |
| 23. Check deletion behavior and log display refresh. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Delete selected/all-but-current logs and verify the current log remains and display refreshes. |

## Shared Libraries (`gss-core`)

### Files, configuration, and data

| Description | Manual/Automated | Notes |
|---|---|---|
| 1. Load and update selected stores. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/settings.rs`: `get_available_stores`, `get_selected_stores`, `update_selected_stores`. |
| 2. Load and update alias options. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/settings.rs`: `get_alias_state`, `update_alias_state`. |
| 3. Persist settings changes. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/functional/commands.rs::config_cmd`. |
| 4. Add and load per-store thresholds. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/thresholds.rs`: `add_steam_game`, `add_gog_game`, `add_microsoft_store_game`. |
| 5. Update threshold aliases, prices, and IDs. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/thresholds.rs`: `update_alias`, `update_price`, `update_id`, `update_id_str`. |
| 6. Remove thresholds and update alias maps. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/thresholds.rs::remove_game`. |
| 7. Parse game/price CSV input from text. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No direct CSV tests exist in `gss-tests`; import CSV through the app and verify valid, empty, malformed, and quoted/multiword rows. |
| 8. Parse game/price CSV input from a file. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Run `gss-cli bulk-insert --file <csv-path>` and verify accepted rows. |
| 9. Generate CSV output. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No direct CSV tests exist; check generated output from `files::csv::generate_csv`. |
| 10. Create directories and files through core file helpers. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No direct `files::general` tests; exercise configuration/log file creation through the CLI/app and inspect the configured project path. |
| 11. Read, write, and append file contents. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Exercise configuration/log file updates through the CLI/app and verify expected contents. |
| 12. Delete files. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Delete a historical log in the app and verify file removal. |
| 13. Create and load properties. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/properties.rs`: `create_properties_file`, `load_properties_from_file`. |
| 14. Import environment variables. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/properties.rs::update_properties_from_env` and `gss-tests/tests/unit/env_vars.rs`. |
| 15. Encrypt and decrypt stored secrets. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/passwords.rs`: encryption, successful decryption, failed decryption. |
| 16. Serialize and use core data models. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No direct model tests; configure/list stores and thresholds in the CLI/app and verify persisted data. |
| 17. Check store IDs and names. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | Store-name coverage in `gss-tests/tests/unit/settings.rs::get_proper_store_name`; threshold store IDs covered in `gss-tests/tests/unit/thresholds.rs`. |
| 18. Check filtering option labels and behavior. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No direct filtering tests; exercise Sales Preview store/price/sort filters. |
| 19. Check constants used in CLI/app configuration. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No dedicated `constants` tests; verify documented flag and store labels in CLI/app. |
| 20. Check file, API, and email error reporting. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No dedicated `errors` tests; trigger missing/corrupt config, unavailable store APIs, and invalid email settings; verify messages. |

### Store search, pricing, and notifications

| Description | Manual/Automated | Notes |
|---|---|---|
| 1. Calculate Levenshtein similarity. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/algorithms.rs::test_levenshtein_dist`. |
| 2. Calculate Damerau-Levenshtein similarity. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/algorithms.rs::test_damerau_levenshtein`. |
| 3. Calculate Smith-Waterman similarity. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/algorithms.rs::test_smith_waterman`. |
| 4. Check very long inputs. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/unit/algorithms.rs`: long-input tests for all three algorithms. |
| 5. Search games and retrieve prices from Steam. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/api/steam.rs`; live API requests. |
| 6. Search games and retrieve prices from GOG. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/api/gog.rs`; live API requests. |
| 7. Search games and retrieve prices from Microsoft Store. | <span style="color:#006400"><mark style="background-color:#b7e4c7">Automated</mark></span> | `gss-tests/tests/api/microsoft_store.rs`; live API requests. |
| 8. Validate allowed and disallowed URLs. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No direct `alerting::email` URL validation tests; preview links with allowed and disallowed domains. |
| 9. Check generated email HTML. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | No direct HTML-generation tests; generate and inspect an email preview. |
| 10. Check escaped game fields and allowed image/store domains. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Use special characters and valid/invalid URLs in game/store data; confirm HTML escaping and link restrictions. |
| 11. Validate required SMTP properties. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Omit required fields one at a time and verify parameter errors. |
| 12. Send plain-text email. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Use valid disposable SMTP settings; verify delivery. |
| 13. Send HTML email. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Use valid disposable SMTP settings; inspect rendered content and verify delivery. |
| 14. Check unavailable SMTP and send failures. | <span style="color:#a34000"><mark style="background-color:#ffe0b2">Manual</mark></span> | Test with unavailable SMTP and verify errors are reported. |

## Coverage notes

- Existing app tests exercise `App` message/state transitions, not rendered UI
  interaction or full executable flows.
- The CLI `check_prices` test uses live services. App search, cache, sales, and
  email workflows also require manual end-to-end verification.