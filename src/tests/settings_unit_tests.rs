use std::collections::HashMap;
use crate::settings::{self, STEAM_STORE_ID, STEAM_STORE_NAME,
                          GOG_STORE_ID, GOG_STORE_NAME,
                          MICROSOFT_STORE_ID, MICROSOFT_STORE_NAME};

// Constants
static DEFAULT_ALIAS_STATE : bool = true;
static ALIAS_DISABLED : i32 = 0;
static ALIAS_ENABLED : i32 = 1;

fn default_settings() {
    settings::update_selected_stores(Vec::new());
    settings::update_alias_state(ALIAS_ENABLED);
}

#[test]
fn get_available_stores() {
    let available_stores = settings::get_available_stores();
    let mut all_stores_valid = true;
    let mut invalid_store = "";
    for store in &available_stores {
        if store.as_str() != STEAM_STORE_ID &&
            store.as_str() != GOG_STORE_ID &&
            store.as_str() != MICROSOFT_STORE_ID {
            invalid_store = store.as_str();
            all_stores_valid = false;
            break;
        }
    }
    assert_eq!(true, all_stores_valid, "Could not find store: {}", invalid_store);
    invalid_store = "fake_store";
    for store in available_stores {
        assert_ne!(store, invalid_store, "\'{}\' should not be a valid store", invalid_store);
    }
}

#[test]
fn get_proper_store_name() {
    // Test valid store ids
    let mut store_name = settings::get_proper_store_name(STEAM_STORE_ID).unwrap();
    assert_eq!(STEAM_STORE_NAME, store_name, "{} != {}", store_name, STEAM_STORE_NAME);
    store_name = settings::get_proper_store_name(GOG_STORE_ID).unwrap();
    assert_eq!(GOG_STORE_NAME, store_name, "{} != {}", GOG_STORE_NAME, store_name);
    store_name = settings::get_proper_store_name(MICROSOFT_STORE_ID).unwrap();
    assert_eq!(MICROSOFT_STORE_NAME, store_name, "{} != {}", MICROSOFT_STORE_NAME, store_name);
    // Test invalid store id
    store_name = settings::get_proper_store_name("fake_store").unwrap_or_default();
    assert_eq!("", store_name, "\'{}\' is not a valid store id", store_name);
}

#[test]
fn get_selected_stores() {
    default_settings();
    let stores = vec![String::from(STEAM_STORE_ID),
                                  String::from(GOG_STORE_ID)];
    settings::update_selected_stores(stores);
    let mut selected_stores = settings::get_selected_stores();
    let mut is_steam_selected = false;
    let mut is_gog_selected = false;
    let mut is_ms_store_selected = false;
    for store in &selected_stores {
        if store == STEAM_STORE_ID { is_steam_selected = true }
        else if store == GOG_STORE_ID { is_gog_selected = true }
        if store == MICROSOFT_STORE_ID { is_ms_store_selected = true }
    }
    assert_eq!(true, is_steam_selected, "{} should be selected", STEAM_STORE_ID);
    assert_eq!(true, is_gog_selected, "{} should be selected", GOG_STORE_ID);
    assert_eq!(false, is_ms_store_selected, "{} should not be selected", MICROSOFT_STORE_ID);
}

#[test]
fn get_alias_state() {
    default_settings();
    let are_aliases_enabled = settings::get_alias_state();
    assert_eq!(true, are_aliases_enabled, "Aliases should be enabled.");
}

#[test]
fn update_selected_stores() {
    default_settings();
    let mut selected_stores = settings::get_selected_stores();
    assert_eq!(0, selected_stores.len(), "No stores should be selected by default");
    // Check that stores are added to settings
    settings::update_selected_stores(vec![String::from(STEAM_STORE_ID),
                                                      String::from(GOG_STORE_ID)]);
    selected_stores = settings::get_selected_stores();
    assert_eq!(2, selected_stores.len(), "The number of selected stores should be 2 not {}", selected_stores.len());
    assert_eq!(STEAM_STORE_ID, selected_stores[0], "\'{}\' != \'{}\'", STEAM_STORE_ID, selected_stores[0]);
    assert_eq!(GOG_STORE_ID, selected_stores[1], "\'{}\' != \'{}\'", GOG_STORE_ID, selected_stores[1]);
    // Check that no duplicates exist
    settings::update_selected_stores(vec![String::from(STEAM_STORE_ID),
                                          String::from(STEAM_STORE_ID),
                                          String::from(STEAM_STORE_ID),
                                          String::from(GOG_STORE_ID),
                                          String::from(GOG_STORE_ID),
                                          String::from(MICROSOFT_STORE_ID)]);
    selected_stores = settings::get_selected_stores();
    let mut store_count : HashMap<String, i32> = HashMap::new();
    for store in selected_stores {
        let val = store_count.entry(store.clone()).or_insert(0);
        *val += 1;
    }
    let store_limit = 1;
    for store in store_count {
        let count = &store.1;
        assert_eq!(store_limit, *count, "\'{}\' should not have more than 1 entry.", store.0);
    }
}

#[test]
fn update_alias_state(){
    default_settings();
    let mut are_aliases_enabled = settings::get_alias_state();
    // Check default alias state is true
    assert_eq!(DEFAULT_ALIAS_STATE, are_aliases_enabled, "Aliases should be enabled by default.");
    // Check that aliases are disabled
    settings::update_alias_state(ALIAS_DISABLED);
    are_aliases_enabled = settings::get_alias_state();
    assert_eq!(false, are_aliases_enabled, "Aliases should not be enabled.");
    // Check that behavior of values that are not 1 or 0
    for i in -10..10 {
        if i != ALIAS_ENABLED && i != ALIAS_DISABLED {
            settings::update_alias_state(i);
            let are_aliases_enabled = settings::get_alias_state();
            assert_eq!(false, are_aliases_enabled, "Aliases should not be enabled given input: {}.", i);
        }
    }
}