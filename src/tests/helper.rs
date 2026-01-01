use std::{env, fs};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use dotenv::dotenv as dotenv_linux;
use dotenvy::dotenv as dotenv_windows;
use serde_json::{json, Value};
use file_types::json;
use structs::data::GameThreshold;

pub static THRESHOLD_FILENAME: &str = "thresholds.json";
pub static SETTINGS_FILENAME: &str = "config.json";

pub fn get_data_path() -> String {
    if !json::get_test_mode() { json::set_test_mode(true); }
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }
    let mut data_path = env::var("TEST_PATH").unwrap_or_else(|_| String::from("."));
    let path_buf: PathBuf = [&data_path, "data"].iter().collect();
    data_path = path_buf.display().to_string();
    if !Path::new(&data_path).is_dir() {
        let _ = fs::create_dir(&data_path);
    }
    data_path
}

pub fn get_threshold_path() -> String {
    let mut threshold_path = get_data_path();
    let path_buf: PathBuf = [threshold_path, THRESHOLD_FILENAME.to_string()].iter().collect();
    threshold_path = path_buf.display().to_string();
    json::get_path(&threshold_path)
}

pub fn get_settings_path() -> String {
    let mut settings_path = get_data_path();
    let path_buf: PathBuf = [&settings_path, SETTINGS_FILENAME].iter().collect();
    settings_path = path_buf.display().to_string();
    json::get_path(&settings_path)
}

pub fn load_thresholds() -> Vec<GameThreshold> {
    let filepath = get_threshold_path();
    let data = read_to_string(filepath).unwrap();
    serde_json::from_str::<Vec<GameThreshold>>(&data).unwrap_or_default()
}

pub fn load_stores() -> Vec<String> {
    let filepath = get_settings_path();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get selected stores - could not convert to JSON");
    let selected = serde_json::to_string(&body["selected_stores"]).unwrap();
    serde_json::from_str::<Vec<String>>(&selected).unwrap_or_default()
}

pub fn load_alias_state() -> bool{
    let filepath = get_settings_path();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get alias state - could not convert to JSON");
    let alias_enabled =serde_json::to_string(&body["alias_enabled"]).unwrap();
    serde_json::from_str::<bool>(&alias_enabled).unwrap_or_else(|_|false)
}

pub fn clear_settings() {
    let settings = json!({"selected_stores": [], "alias_enabled": 1});
    let settings_str = serde_json::to_string_pretty(&settings);
    json::write_to_file(get_settings_path(), settings_str.expect("Clear settings."));
}

pub fn clear_thresholds(){
    let thresholds = json!([]);
    let thresholds_str = serde_json::to_string_pretty(&thresholds);
    json::write_to_file(get_threshold_path(), thresholds_str.expect("Clear thresholds."));
}