use std::env;
use std::fs::{read_to_string};
use std::path::{Path};
use std::sync::{Mutex, OnceLock};

use file_types::general;
use properties;
use serde_json::Value;

use constants::operations::properties::*;
use crate::utils::{file_operations, tmp_setup};


const TMP_DIR_TITLE: &str = "properties";
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn retrieve_env_var(env_var: &str) -> Option<String> {
    env::var(env_var).ok()
}

fn restore_env_var(env_var: &str, value: Option<String>) {
    unsafe {
        match value {
            Some(data) => env::set_var(env_var, data),
            None => env::remove_var(env_var),
        }
    }
}

fn create_env_str(steam_api_key_val: &str, recipient_email_val: &str, smtp_host_val: &str, smtp_port_val: &str, smtp_email_val: &str, 
    smtp_username_val: &str, smtp_password_val: &str, dir_path: &Path)-> String {
    format!(
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
    )
}

#[test]
fn create_properties_file() {
    let _guard = test_lock().lock().unwrap();
    let temp_dir = tmp_setup::create(TMP_DIR_TITLE, file_operations::load_steam_cache());
    let prev_project_path = retrieve_env_var(PROJECT_PATH_ENV);
    let prev_test_path = retrieve_env_var(TEST_PATH_ENV);
    let prev_dir = env::current_dir().unwrap();

    unsafe {
        env::set_var(PROJECT_PATH_ENV, temp_dir.display().to_string());
        env::remove_var(TEST_PATH_ENV);
    }
    env::set_current_dir(&temp_dir).unwrap();

    let properties_path = properties::get_properties_path();
    assert!(Path::new(&properties_path).is_file());
    assert!(properties_path.ends_with(&format!("{}{}{}", CONFIG_DIR, std::path::MAIN_SEPARATOR, PROPERTIES_FILENAME)));

    let contents = read_to_string(&properties_path).unwrap();
    let json: Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(json[PROP_STEAM_API_KEY].as_str().unwrap(), "", "Expected {} to be empty string in properties file", PROP_STEAM_API_KEY);
    assert_eq!(json[PROP_RECIPIENT_EMAIL].as_str().unwrap(), "", "Expected {} to be empty string in properties file", PROP_RECIPIENT_EMAIL);
    assert_eq!(json[PROP_SMTP_HOST].as_str().unwrap(), "", "Expected {} to be empty string in properties file", PROP_SMTP_HOST);
    assert_eq!(json[PROP_SMTP_EMAIL].as_str().unwrap(), "", "Expected {} to be empty string in properties file", PROP_SMTP_EMAIL);
    assert_eq!(json[PROP_SMTP_USERNAME].as_str().unwrap(), "", "Expected {} to be empty string in properties file", PROP_SMTP_USERNAME);
    assert_eq!(json[PROP_SMTP_PASSWORD].as_str().unwrap(), "", "Expected {} to be empty string in properties file", PROP_SMTP_PASSWORD);
    assert_eq!(json[PROP_SMTP_PORT].as_i64().unwrap(), 0, "Expected {} to be 0 in properties file", PROP_SMTP_PORT);
    assert_eq!(json[PROP_PROJECT_PATH].as_str().unwrap(), temp_dir.display().to_string(), "Expected {} to be {} in properties file", PROP_PROJECT_PATH, temp_dir.display().to_string());
    assert_eq!(json[PROP_TEST_PATH].as_str().unwrap(), temp_dir.join(DEFAULT_TEST_DIR).display().to_string(), "Expected {} to be {} in properties file", PROP_TEST_PATH, temp_dir.join(DEFAULT_TEST_DIR).display().to_string());

    env::set_current_dir(prev_dir).unwrap();
    restore_env_var(PROJECT_PATH_ENV, prev_project_path);
    restore_env_var(TEST_PATH_ENV, prev_test_path);
    tmp_setup::clean_up(&temp_dir);
}

#[test]
fn load_properties_from_file() {
    let _guard = test_lock().lock().unwrap();
    let temp_dir = tmp_setup::create(TMP_DIR_TITLE, file_operations::load_steam_cache());
    let prev_project_path = retrieve_env_var(PROJECT_PATH_ENV);
    let prev_test_path = retrieve_env_var(TEST_PATH_ENV);
    let prev_dir = env::current_dir().unwrap();

    unsafe {
        env::set_var(PROJECT_PATH_ENV, temp_dir.display().to_string());
        env::remove_var(TEST_PATH_ENV);
    }
    env::set_current_dir(&temp_dir).unwrap();

    let _ = properties::get_properties_path();
    let loaded_properties = properties::load_properties().expect("Properties should load");

    assert_eq!(loaded_properties[PROP_STEAM_API_KEY].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_STEAM_API_KEY);
    assert_eq!(loaded_properties[PROP_RECIPIENT_EMAIL].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_RECIPIENT_EMAIL);
    assert_eq!(loaded_properties[PROP_SMTP_HOST].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_SMTP_HOST);
    assert_eq!(loaded_properties[PROP_SMTP_PORT].as_i64().unwrap(), 0);
    assert_eq!(loaded_properties[PROP_SMTP_EMAIL].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_SMTP_EMAIL);
    assert_eq!(loaded_properties[PROP_SMTP_USERNAME].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_SMTP_USERNAME);
    assert_eq!(loaded_properties[PROP_SMTP_PASSWORD].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_SMTP_PASSWORD);
    assert_eq!(loaded_properties[PROP_PROJECT_PATH].as_str().unwrap(), temp_dir.display().to_string());
    assert_eq!(loaded_properties[PROP_TEST_PATH].as_str().unwrap(), temp_dir.join(DEFAULT_TEST_DIR).display().to_string());

    env::set_current_dir(prev_dir).unwrap();
    restore_env_var(PROJECT_PATH_ENV, prev_project_path);
    restore_env_var(TEST_PATH_ENV, prev_test_path);
    tmp_setup::clean_up(&temp_dir);
}

#[test]
fn sub_directories_created() {
    let _guard = test_lock().lock().unwrap();
    let temp_dir = tmp_setup::create(TMP_DIR_TITLE, file_operations::load_steam_cache());
    let prev_project_path = retrieve_env_var(PROJECT_PATH_ENV);
    let prev_test_path = retrieve_env_var(TEST_PATH_ENV);
    let prev_dir = env::current_dir().unwrap();

    unsafe {
        env::set_var(PROJECT_PATH_ENV, temp_dir.display().to_string());
        env::remove_var(TEST_PATH_ENV);
    }
    env::set_current_dir(&temp_dir).unwrap();

    let _ = properties::get_properties_path();

    let data_path = properties::get_data_path();
    let config_path = properties::get_config_path();

    assert_eq!(data_path, temp_dir.join(DATA_DIR).display().to_string());
    assert_eq!(config_path, temp_dir.join(CONFIG_DIR).display().to_string());
    assert!(Path::new(&data_path).is_dir());
    assert!(Path::new(&config_path).is_dir());

    env::set_current_dir(prev_dir).unwrap();
    restore_env_var(PROJECT_PATH_ENV, prev_project_path);
    restore_env_var(TEST_PATH_ENV, prev_test_path);
    tmp_setup::clean_up(&temp_dir);
}

#[test]
fn update_properties_from_env() {
    let _guard = test_lock().lock().unwrap();
    let temp_dir = tmp_setup::create(TMP_DIR_TITLE, file_operations::load_steam_cache());
    let prev_project_path = retrieve_env_var(PROJECT_PATH_ENV);
    let prev_test_path = retrieve_env_var(TEST_PATH_ENV);
    let prev_dir = env::current_dir().unwrap();


    env::set_current_dir(&temp_dir).unwrap();
    let mut steam_api_key_val = "INITIAL";
    let mut recipient_email_val = "recipient@example.com";
    let mut smtp_host_val = "smtp.initial.com";
    let mut smtp_port_val = "587";
    let mut smtp_email_val = "user@initial.com";
    let mut smtp_username_val = "initial_user";
    let mut smtp_password_val = "initial_pwd";
    let mut env_data = create_env_str(steam_api_key_val, recipient_email_val, smtp_host_val, smtp_port_val, smtp_email_val, smtp_username_val, smtp_password_val, &temp_dir);
    general::write_file(&temp_dir, ENV_FILENAME, &env_data);

    steam_api_key_val = "UPDATED";
    recipient_email_val = "recipient2@example.com";
    smtp_host_val = "smtp.updated.com";
    smtp_port_val = "587";
    smtp_email_val = "user@updated.com";
    smtp_username_val = "updated_user";
    smtp_password_val = "updated_pwd";
    env_data = create_env_str( steam_api_key_val, recipient_email_val, smtp_host_val, smtp_port_val, smtp_email_val, smtp_username_val, smtp_password_val, &temp_dir);
    // println!("Env data:\n{}", env_data);
    general::write_file(&temp_dir, ENV_FILENAME, &env_data);

    unsafe {
        env::remove_var(STEAM_API_KEY_ENV);
        env::remove_var(RECIPIENT_EMAIL_ENV);
        env::remove_var(SMTP_HOST_ENV);
        env::remove_var(SMTP_PORT_ENV);
        env::remove_var(SMTP_EMAIL_ENV);
        env::remove_var(SMTP_USERNAME_ENV);
        env::remove_var(SMTP_PASSWORD_ENV);
        env::remove_var(PROJECT_PATH_ENV);
        env::remove_var(TEST_PATH_ENV);
    }

    properties::update_properties_from_env();
    let updated_properties = properties::load_properties().expect("Updated properties should load");
    let key_str = properties::env_vars::get_decrypt_key(properties::get_project_path());
    
    let encrypted_steam_key = updated_properties[PROP_STEAM_API_KEY].as_str().unwrap().to_string();
    let decrypted_steam_key = properties::passwords::decrypt(&key_str, encrypted_steam_key);
    assert_eq!(decrypted_steam_key, steam_api_key_val);
    assert_eq!(updated_properties[PROP_RECIPIENT_EMAIL].as_str().unwrap(), recipient_email_val);
    assert_eq!(updated_properties[PROP_SMTP_EMAIL].as_str().unwrap(), smtp_email_val);
    assert_eq!(updated_properties[PROP_SMTP_USERNAME].as_str().unwrap(), smtp_username_val);
    assert_eq!(updated_properties[PROP_SMTP_HOST].as_str().unwrap(), smtp_host_val);
    assert_eq!(updated_properties[PROP_SMTP_PORT].as_i64().unwrap(), smtp_port_val.parse::<i64>().unwrap());    assert_eq!(updated_properties[PROP_SMTP_HOST].as_str().unwrap(), smtp_host_val);
    assert_eq!(updated_properties[PROP_SMTP_EMAIL].as_str().unwrap(), smtp_email_val);
    assert_eq!(updated_properties[PROP_SMTP_USERNAME].as_str().unwrap(), smtp_username_val);
    let encrypted_smtp_pwd = updated_properties[PROP_SMTP_PASSWORD].as_str().unwrap().to_string();
    let decrypted_smtp_pwd = properties::passwords::decrypt(&key_str, encrypted_smtp_pwd);
    assert_eq!(decrypted_smtp_pwd, smtp_password_val);
    assert_eq!(updated_properties[PROP_PROJECT_PATH].as_str().unwrap(), temp_dir.display().to_string());
    assert_eq!(updated_properties[PROP_TEST_PATH].as_str().unwrap(), temp_dir.join(DEFAULT_TEST_DIR).display().to_string());

    env::set_current_dir(prev_dir).unwrap();
    restore_env_var(PROJECT_PATH_ENV, prev_project_path);
    restore_env_var(TEST_PATH_ENV, prev_test_path);
    tmp_setup::clean_up(&temp_dir);
}