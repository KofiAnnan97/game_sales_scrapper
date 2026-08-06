use std::collections::HashMap;
use file_ops::settings;
use constants::operations::settings::{ENABLED_STATE, DISABLED_STATE, DEFAULT_ALIAS_STATE};
use properties;
use types::internal::store::GameStore;
use crate::utils::{tmp_setup};

static TMP_DIR_TITLE : &str = "settings";

#[test]
fn get_available_stores() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let _ = properties::load_properties();

    let available_stores = settings::get_available_stores();
    let mut all_stores_valid = true;
    for store in &available_stores {
        if store != &GameStore::STEAM &&
            store != &GameStore::GOOD_OLD_GAMES &&
            store!= &GameStore::MICROSOFT_STORE_PC {
            all_stores_valid = false;
            break;
        }
    }
    assert_eq!(true, all_stores_valid, "One of the follow are not valid: {:?}", &available_stores);

    _tmp_env.tear_down();
}

#[test]
fn get_proper_store_name() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let _ = properties::load_properties();

    let mut store_name = GameStore::STEAM.get_name();
    assert_eq!("Steam", store_name, "{} != {}", store_name, "Steam");
    store_name = GameStore::GOOD_OLD_GAMES.get_name();
    assert_eq!("Good Old Games (GOG)", store_name, "{} != {}", "Good Old Games (GOG)", store_name);
    store_name = GameStore::MICROSOFT_STORE_PC.get_name();
    assert_eq!("Microsoft Store (PC)", store_name, "{} != {}", "Microsoft Store (PC)", store_name);

    _tmp_env.tear_down();
}

#[test]
fn get_selected_stores() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let _ = properties::load_properties();
    
    let stores = vec![GameStore::STEAM, GameStore::GOOD_OLD_GAMES];
    settings::update_selected_stores(stores);
    let selected_stores = settings::get_selected_stores();
    let mut is_steam_selected = false;
    let mut is_gog_selected = false;
    let mut is_ms_store_selected = false;
    for store in &selected_stores {
        if store == &GameStore::STEAM { is_steam_selected = true }
        else if store == &GameStore::GOOD_OLD_GAMES { is_gog_selected = true }
        if store == &GameStore::MICROSOFT_STORE_PC { is_ms_store_selected = true }
    }
    assert_eq!(true, is_steam_selected, "{} should be selected", GameStore::STEAM);
    assert_eq!(true, is_gog_selected, "{} should be selected", GameStore::GOOD_OLD_GAMES);
    assert_eq!(false, is_ms_store_selected, "{} should not be selected", GameStore::MICROSOFT_STORE_PC);
    
    _tmp_env.tear_down();
}

#[test]
fn get_alias_state() {
    let _test_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());    
    let _ = properties::load_properties();
    
    let are_aliases_enabled = settings::get_alias_state();
    assert_eq!(true, are_aliases_enabled, "Aliases should be enabled.");
    
    _test_env.tear_down();
}

#[test]
fn update_selected_stores() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let _ = properties::load_properties();
    
    let mut selected_stores = settings::get_selected_stores();
    assert_eq!(0, selected_stores.len(), "No stores should be selected by default");
    // Check that stores are added to settings
    settings::update_selected_stores(vec![GameStore::STEAM,GameStore::GOOD_OLD_GAMES]);
    selected_stores = settings::get_selected_stores();
    assert_eq!(2, selected_stores.len(), "The number of selected stores should be 2 not {}", selected_stores.len());
    assert_eq!(GameStore::STEAM, selected_stores[0], "\'{}\' != \'{}\'", GameStore::STEAM, selected_stores[0]);
    assert_eq!(GameStore::GOOD_OLD_GAMES, selected_stores[1], "\'{}\' != \'{}\'", GameStore::GOOD_OLD_GAMES, selected_stores[1]);
    // Check that no duplicates exist
    settings::update_selected_stores(vec![GameStore::STEAM,
                                                    GameStore::STEAM,
                                                    GameStore::STEAM,
                                                    GameStore::GOOD_OLD_GAMES,
                                                    GameStore::GOOD_OLD_GAMES,
                                                    GameStore::MICROSOFT_STORE_PC]);
    selected_stores = settings::get_selected_stores();
    let mut store_count : HashMap<String, i32> = HashMap::new();
    for store in selected_stores {
        let val = store_count.entry(store.to_string()).or_insert(0);
        *val += 1;
    }
    let store_limit = 1;
    for store in store_count {
        let count = &store.1;
        assert_eq!(store_limit, *count, "\'{}\' should not have more than 1 entry.", store.0);
    }

    _tmp_env.tear_down();
}

#[test]
fn update_alias_state(){
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let _ = properties::load_properties();
    
    let mut are_aliases_enabled = settings::get_alias_state();
    // Check default alias state is true
    assert_eq!(DEFAULT_ALIAS_STATE, are_aliases_enabled, "Aliases should be enabled by default.");
    // Check that aliases are disabled
    settings::update_alias_state(DISABLED_STATE);
    are_aliases_enabled = settings::get_alias_state();
    assert_eq!(false, are_aliases_enabled, "Aliases should not be enabled.");
    // Check that behavior of values that are not 1 or 0
    for i in -10..10 {
        if i != ENABLED_STATE && i != DISABLED_STATE {
            settings::update_alias_state(i);
            let are_aliases_enabled = settings::get_alias_state();
            assert_eq!(false, are_aliases_enabled, "Aliases should not be enabled given input: {}.", i);
        }
    }
    
    _tmp_env.tear_down();
}