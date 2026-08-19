use std::fs::{metadata, read_to_string};
use std::path::{Path, PathBuf};
use serde_json::{json, Result, Value};
use properties;
use files::general;
use types::{
    internal::data::GameThreshold,
    response::steam::App
};
use constants::operations::thresholds::{ALIAS_MAP, THRESHOLDS, THRESHOLD_FILENAME};
use constants::operations::settings::{ALIASES_ENABLED, ALLOW_ALIAS_REUSE_AFTER_CREATION, 
                                      SELECTED_STORES, SETTINGS_FILENAME};
use constants::operations::properties::*;
use constants::stores::steam::{CACHE_FILENAME};

pub fn get_data_path() -> String {
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    let mut data_path = properties::get_test_path();
    let path_buf: PathBuf = [&data_path, DATA_DIR].iter().collect();
    data_path = path_buf.display().to_string();
    general::create_dir(&data_path);
    data_path
}

pub fn get_config_path() -> String {
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    let mut config_path = properties::get_test_path();
    let path_buf: PathBuf = [&config_path, CONFIG_DIR].iter().collect();
    config_path = path_buf.display().to_string();
    general::create_dir(&config_path);
    config_path
}

pub fn get_threshold_path() -> String {
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

pub fn get_settings_path() -> String {
    let mut settings_path = get_config_path();
    let path_buf: PathBuf = [&settings_path, SETTINGS_FILENAME].iter().collect();
    settings_path = path_buf.display().to_string();
    general::get_path(&settings_path)
}

pub fn clear_settings() {
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    let settings = json!({SELECTED_STORES: [], ALIASES_ENABLED: 1, ALLOW_ALIAS_REUSE_AFTER_CREATION: 1});
    let settings_str = serde_json::to_string_pretty(&settings);
    general::write_to_file(get_settings_path(), settings_str.expect("Clear settings."));
}

pub fn clear_thresholds(){
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    let thresholds = json!({
        THRESHOLDS.to_string(): [],
        ALIAS_MAP.to_string(): {}
    });
    let thresholds_str = serde_json::to_string_pretty(&thresholds);
    general::write_to_file(get_threshold_path(), thresholds_str.expect("Clear thresholds."));
}

pub fn load_steam_cache() -> Vec<App> {
    let path_buf: PathBuf = [get_data_path(), CACHE_FILENAME.to_string()].iter().collect();
    let filepath = path_buf.display().to_string();
    let data = read_to_string(filepath).unwrap();
    let body: Value = serde_json::from_str(&data).expect("Cannot parse steam cache for testing");
    let cache = serde_json::to_string(&body).unwrap();
    serde_json::from_str::<Vec<App>>(&cache).unwrap_or_default()
}

pub fn load_threshold_data() -> Result<Value> {
    let filepath = get_threshold_path();
    let data = read_to_string(filepath).unwrap();
    serde_json::from_str(&data)
}

pub fn load_thresholds() -> Vec<GameThreshold> {
    let filepath = get_threshold_path();
    let data = read_to_string(filepath).unwrap();
    let body: Value = serde_json::from_str(&data).expect("Cannot parse threshold for testing");
    let thresholds = serde_json::to_string(&body[THRESHOLDS]).unwrap();
    serde_json::from_str::<Vec<GameThreshold>>(&thresholds).unwrap_or_default()
}

pub fn load_stores() -> Vec<String> {
    let filepath = get_settings_path();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get selected stores - could not convert to JSON");
    let selected = serde_json::to_string(&body[SELECTED_STORES]).unwrap();
    serde_json::from_str::<Vec<String>>(&selected).unwrap_or_default()
}

pub fn load_alias_state() -> bool{
    let filepath = get_settings_path();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get alias state - could not convert to JSON");
    let alias_enabled =serde_json::to_string(&body[ALIASES_ENABLED]).unwrap();
    serde_json::from_str::<bool>(&alias_enabled).unwrap_or_else(|_|false)
}

pub fn create_env_str(steam_api_key_val: &str, recipient_email_val: &str, smtp_host_val: &str, smtp_port_val: u16, smtp_email_val: &str, 
    smtp_username_val: &str, smtp_password_val: &str, dir_path: &Path)-> String {
    let env_str = format!(
        "{steam_env}=\"{steam_api_key_val}\"\n{recipient_env}=\"{recipient_email_val}\"\n{host_env}=\"{smtp_host_val}\"\n{port_env}={smtp_port_val}\n{email_env}=\"{smtp_email_val}\"\n{user_env}=\"{smtp_username_val}\"\n{pwd_env}=\"{smtp_password_val}\"\n{project_env}=\"{project_path_val}\"\n{test_env}=\"{test_path_val}\"\n",
        steam_env = STEAM_API_KEY_ENV,
        recipient_env = RECIPIENT_EMAIL_ENV,
        host_env = SMTP_HOST_ENV,
        port_env = SMTP_PORT_ENV,
        email_env = SMTP_EMAIL_ENV,
        user_env = SMTP_USERNAME_ENV,
        pwd_env = SMTP_PASSWORD_ENV,
        project_env = PROJECT_PATH_ENV,
        test_env = TEST_PATH_ENV,
        steam_api_key_val = steam_api_key_val,
        recipient_email_val = recipient_email_val,
        smtp_host_val = smtp_host_val,
        smtp_port_val = smtp_port_val,
        smtp_email_val = smtp_email_val,
        smtp_username_val = smtp_username_val,
        smtp_password_val = smtp_password_val,
        project_path_val = dir_path.display().to_string(),
        test_path_val = dir_path.join(DEFAULT_TEST_DIR).display().to_string()
    );
    env_str
}

pub fn teardown(){
    properties::set_test_mode(false);
}