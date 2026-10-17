use constants::operations::properties::*;
use constants::operations::settings::{ALIASES_ENABLED, SELECTED_STORES};
use constants::operations::thresholds::THRESHOLDS;
use serde_json::{Result, Value};
use std::fs::read_to_string;
use std::path::Path;
use types::{internal::data::GameThreshold, response::steam::App};

pub fn load_steam_cache(path: &Path) -> Vec<App> {
    let filepath = path.display().to_string();
    let data = read_to_string(filepath).unwrap();
    let body: Value = serde_json::from_str(&data).expect("Cannot parse steam cache for testing");
    let cache = serde_json::to_string(&body).unwrap();
    serde_json::from_str::<Vec<App>>(&cache).unwrap_or_default()
}

pub fn load_threshold_data(path: &Path) -> Result<Value> {
    let filepath = path.display().to_string();
    let data = read_to_string(filepath).unwrap();
    serde_json::from_str(&data)
}

pub fn load_thresholds(path: &Path) -> Vec<GameThreshold> {
    let filepath = path.display().to_string();
    let data = read_to_string(filepath).unwrap();
    let body: Value = serde_json::from_str(&data).expect("Cannot parse threshold for testing");
    let thresholds = serde_json::to_string(&body[THRESHOLDS]).unwrap();
    serde_json::from_str::<Vec<GameThreshold>>(&thresholds).unwrap_or_default()
}

pub fn load_stores(path: &Path) -> Vec<String> {
    let filepath = path.display().to_string();
    let data = read_to_string(filepath).unwrap();
    let body: Value =
        serde_json::from_str(&data).expect("Get selected stores - could not convert to JSON");
    let selected = serde_json::to_string(&body[SELECTED_STORES]).unwrap();
    serde_json::from_str::<Vec<String>>(&selected).unwrap_or_default()
}

pub fn load_alias_state(path: &Path) -> bool {
    let filepath = path.display().to_string();
    let data = read_to_string(filepath).unwrap();
    let body: Value =
        serde_json::from_str(&data).expect("Get alias state - could not convert to JSON");
    let alias_enabled = serde_json::to_string(&body[ALIASES_ENABLED]).unwrap();
    serde_json::from_str::<bool>(&alias_enabled).unwrap_or(false)
}

#[allow(clippy::too_many_arguments)]
pub fn create_env_str(
    steam_api_key_val: &str,
    recipient_email_val: &str,
    smtp_host_val: &str,
    smtp_port_val: u16,
    smtp_email_val: &str,
    smtp_username_val: &str,
    smtp_password_val: &str,
    dir_path: &Path,
) -> String {
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
        project_path_val = dir_path.display(),
        test_path_val = dir_path.join(DEFAULT_TEST_DIR).display()
    );
    env_str
}
