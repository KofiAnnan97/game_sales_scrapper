use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use serde_json::json;
use file_types::general;
use constants::operations::thresholds::{ALIAS_MAP, THRESHOLDS, THRESHOLD_FILENAME};
use constants::operations::settings::{ALIASES_ENABLED, ALLOW_ALIAS_REUSE_AFTER_CREATION, 
                                      SELECTED_STORES, SETTINGS_FILENAME};
use constants::operations::properties::{CONFIG_DIR, DATA_DIR};
use constants::stores::steam::{CACHE_FILENAME};
use structs::response::steam::App;


fn get_dir(title: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("gss_test_{}_{}", title, timestamp));
    fs::create_dir_all(&path).unwrap();
    path
}

pub fn create(title: &str, steam_cache: Vec<App>) -> PathBuf {
    let base_dir = get_dir(title);
    let data_dir = base_dir.join(DATA_DIR);
    let config_dir = base_dir.join(CONFIG_DIR);
    general::create_dir(&data_dir.display().to_string());
    general::create_dir(&config_dir.display().to_string());
    let tmp_thresholds_file = data_dir.join(THRESHOLD_FILENAME).with_extension("json").display().to_string();
    let tmp_settings_file = config_dir.join(SETTINGS_FILENAME).with_extension("json").display().to_string();
    let tmp_steam_cache_file = data_dir.join(CACHE_FILENAME).with_extension("json").display().to_string();
    general::write_to_file(tmp_steam_cache_file, serde_json::to_string_pretty(&steam_cache).expect("Failed to write steam cache for testing"));
    general::write_to_file(tmp_settings_file, serde_json::to_string_pretty(&json!({
        SELECTED_STORES.to_string(): [],
        ALIASES_ENABLED.to_string(): 1,
        ALLOW_ALIAS_REUSE_AFTER_CREATION.to_string(): 1
    })).expect("Failed to write settings for testing"));
    general::write_to_file(tmp_thresholds_file, serde_json::to_string_pretty(&json!({
        THRESHOLDS.to_string(): [],
        ALIAS_MAP.to_string(): {}
    })).expect("Failed to write thresholds for testing"));
    base_dir
}

pub fn clean_up(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
        println!("Cleaned up temporary directory: {}", path.display());
    }
}