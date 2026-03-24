use std::fs::{metadata, read_to_string};
use std::path::{PathBuf};
use serde_json::{json, Result, Value};
use properties;
use file_types::general;
use structs::internal::data::GameThreshold;
use constants::operations::thresholds::{ALIAS_MAP, THRESHOLDS, THRESHOLD_FILENAME};
use constants::operations::settings::{ALIASES_ENABLED, ALLOW_ALIAS_REUSE_AFTER_CREATION, 
                                      SELECTED_STORES, SETTINGS_FILENAME};
use constants::operations::properties::{CONFIG_DIR, DATA_DIR};

pub(in crate::tests) fn get_data_path() -> String {
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    let mut data_path = properties::get_test_path();
    let path_buf: PathBuf = [&data_path, DATA_DIR].iter().collect();
    data_path = path_buf.display().to_string();
    general::create_dir(&data_path);
    data_path
}

pub(in crate::tests) fn get_config_path() -> String {
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    let mut config_path = properties::get_test_path();
    let path_buf: PathBuf = [&config_path, CONFIG_DIR].iter().collect();
    config_path = path_buf.display().to_string();
    general::create_dir(&config_path);
    config_path
}

pub(in crate::tests) fn get_threshold_path() -> String {
    let path_buf: PathBuf = [get_data_path(), THRESHOLD_FILENAME.to_string()].iter().collect();
    let threshold_path = path_buf.display().to_string();
    let path_str = general::get_path(&threshold_path);
    match metadata(&path_str){
        Ok(md) => {
            if md.len() == 0 {
                let data = json!({
                    THRESHOLDS.to_string(): [],
                    ALIAS_MAP.to_string(): {},
                });
                let data_str = serde_json::to_string_pretty(&data);
                general::write_to_file(threshold_path.clone(), data_str.expect("Initial settings could not be created."));
            }
        },
        Err(e) => eprintln!("Error: {}", e)
    }
    path_str
}

pub(in crate::tests) fn get_settings_path() -> String {
    let mut settings_path = get_config_path();
    let path_buf: PathBuf = [&settings_path, SETTINGS_FILENAME].iter().collect();
    settings_path = path_buf.display().to_string();
    general::get_path(&settings_path)
}

pub(in crate::tests) fn clear_settings() {
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    let settings = json!({SELECTED_STORES: [], ALIASES_ENABLED: 1, ALLOW_ALIAS_REUSE_AFTER_CREATION: 1});
    let settings_str = serde_json::to_string_pretty(&settings);
    general::write_to_file(get_settings_path(), settings_str.expect("Clear settings."));
}

pub(in crate::tests) fn clear_thresholds(){
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    let thresholds = json!({
        THRESHOLDS.to_string(): [],
        ALIAS_MAP.to_string(): {}
    });
    let thresholds_str = serde_json::to_string_pretty(&thresholds);
    general::write_to_file(get_threshold_path(), thresholds_str.expect("Clear thresholds."));
}

pub(in crate::tests) fn load_threshold_data() -> Result<Value> {
    let filepath = get_threshold_path();
    let data = read_to_string(filepath).unwrap();
    serde_json::from_str(&data)
}

pub(in crate::tests) fn load_thresholds() -> Vec<GameThreshold> {
    let filepath = get_threshold_path();
    let data = read_to_string(filepath).unwrap();
    let body: Value = serde_json::from_str(&data).expect("Cannot parse threshold for testing");
    let thresholds = serde_json::to_string(&body[THRESHOLDS]).unwrap();
    serde_json::from_str::<Vec<GameThreshold>>(&thresholds).unwrap_or_default()
}

pub(in crate::tests) fn load_stores() -> Vec<String> {
    let filepath = get_settings_path();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get selected stores - could not convert to JSON");
    let selected = serde_json::to_string(&body[SELECTED_STORES]).unwrap();
    serde_json::from_str::<Vec<String>>(&selected).unwrap_or_default()
}

pub(in crate::tests) fn load_alias_state() -> bool{
    let filepath = get_settings_path();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get alias state - could not convert to JSON");
    let alias_enabled =serde_json::to_string(&body[ALIASES_ENABLED]).unwrap();
    serde_json::from_str::<bool>(&alias_enabled).unwrap_or_else(|_|false)
}

pub(in crate::tests) fn teardown(){
    properties::set_test_mode(false);
}