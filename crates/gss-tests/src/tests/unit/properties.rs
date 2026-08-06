use std::env;
use std::fs::{read_to_string};
use std::path::{Path};

use files::general;
use properties;
use properties::passwords::Password;
use serde_json::Value;

use constants::operations::properties::*;
use crate::utils::{file_operations, tmp_setup};


const TMP_DIR_TITLE: &str = "properties";

#[test]
fn create_properties_file() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
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
    assert_eq!(json[PROP_PROJECT_PATH].as_str().unwrap(), _tmp_env.temp_dir.display().to_string(), "Expected {} to be {} in properties file", PROP_PROJECT_PATH, _tmp_env.temp_dir.display().to_string());
    assert_eq!(json[PROP_TEST_PATH].as_str().unwrap(), _tmp_env.temp_dir.join(DEFAULT_TEST_DIR).display().to_string(), "Expected {} to be {} in properties file", PROP_TEST_PATH, _tmp_env.temp_dir.join(DEFAULT_TEST_DIR).display().to_string());

    _tmp_env.tear_down();
}

#[test]
fn load_properties_from_file() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let _ = properties::get_properties_path();
    let loaded_properties = properties::load_properties().expect("Properties should load");

    assert_eq!(loaded_properties[PROP_STEAM_API_KEY].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_STEAM_API_KEY);
    assert_eq!(loaded_properties[PROP_RECIPIENT_EMAIL].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_RECIPIENT_EMAIL);
    assert_eq!(loaded_properties[PROP_SMTP_HOST].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_SMTP_HOST);
    assert_eq!(loaded_properties[PROP_SMTP_PORT].as_i64().unwrap(), 0);
    assert_eq!(loaded_properties[PROP_SMTP_EMAIL].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_SMTP_EMAIL);
    assert_eq!(loaded_properties[PROP_SMTP_USERNAME].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_SMTP_USERNAME);
    assert_eq!(loaded_properties[PROP_SMTP_PASSWORD].as_str().unwrap(), "", "Expected {} to be empty string in loaded properties", PROP_SMTP_PASSWORD);
    assert_eq!(loaded_properties[PROP_PROJECT_PATH].as_str().unwrap(), _tmp_env.temp_dir.display().to_string());
    assert_eq!(loaded_properties[PROP_TEST_PATH].as_str().unwrap(), _tmp_env.temp_dir.join(DEFAULT_TEST_DIR).display().to_string());

    _tmp_env.tear_down();
}

#[test]
fn sub_directories_created() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let _ = properties::get_properties_path();

    let data_path = properties::get_data_path();
    let config_path = properties::get_config_path();

    assert_eq!(data_path, _tmp_env.temp_dir.join(DATA_DIR).display().to_string());
    assert_eq!(config_path, _tmp_env.temp_dir.join(CONFIG_DIR).display().to_string());
    assert!(Path::new(&data_path).is_dir());
    assert!(Path::new(&config_path).is_dir());

    _tmp_env.tear_down();
}

#[test]
fn update_properties_from_env() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let mut steam_api_key_val = "INITIAL";
    let mut recipient_email_val = "recipient@example.com";
    let mut smtp_host_val = "smtp.initial.com";
    let mut smtp_port_val: u16 = 587;
    let mut smtp_email_val = "user@initial.com";
    let mut smtp_username_val = "initial_user";
    let mut smtp_password_val = "initial_pwd";
    let mut env_data = file_operations::create_env_str(steam_api_key_val, recipient_email_val, smtp_host_val, smtp_port_val, smtp_email_val, smtp_username_val, smtp_password_val, &_tmp_env.temp_dir);
    general::write_file(&_tmp_env.temp_dir, ENV_FILENAME, &env_data);

    steam_api_key_val = "UPDATED";
    recipient_email_val = "recipient2@example.com";
    smtp_host_val = "smtp.updated.com";
    smtp_port_val = 587;
    smtp_email_val = "user@updated.com";
    smtp_username_val = "updated_user";
    smtp_password_val = "updated_pwd";
    env_data = file_operations::create_env_str( steam_api_key_val, recipient_email_val, smtp_host_val, smtp_port_val, smtp_email_val, smtp_username_val, smtp_password_val, &_tmp_env.temp_dir);
    // println!("Env data:\n{}", env_data);
    general::write_file(&_tmp_env.temp_dir, ENV_FILENAME, &env_data);

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
    let steam_key = Password::new_encrypted(encrypted_steam_key);
    let decrypted_steam_key = steam_key.get_value(Some(&key_str));
    assert_eq!(decrypted_steam_key, steam_api_key_val);
    assert_eq!(updated_properties[PROP_RECIPIENT_EMAIL].as_str().unwrap(), recipient_email_val);
    assert_eq!(updated_properties[PROP_SMTP_EMAIL].as_str().unwrap(), smtp_email_val);
    assert_eq!(updated_properties[PROP_SMTP_USERNAME].as_str().unwrap(), smtp_username_val);
    assert_eq!(updated_properties[PROP_SMTP_HOST].as_str().unwrap(), smtp_host_val);
    assert_eq!(updated_properties[PROP_SMTP_PORT].as_i64().unwrap(), smtp_port_val as i64);    
    assert_eq!(updated_properties[PROP_SMTP_HOST].as_str().unwrap(), smtp_host_val);
    assert_eq!(updated_properties[PROP_SMTP_EMAIL].as_str().unwrap(), smtp_email_val);
    assert_eq!(updated_properties[PROP_SMTP_USERNAME].as_str().unwrap(), smtp_username_val);
    let encrypted_smtp_pwd = updated_properties[PROP_SMTP_PASSWORD].as_str().unwrap().to_string();
    let smtp_pwd = Password::new_encrypted(encrypted_smtp_pwd);
    let decrypted_smtp_pwd = smtp_pwd.get_value(Some(&key_str));
    assert_eq!(decrypted_smtp_pwd, smtp_password_val);
    assert_eq!(updated_properties[PROP_PROJECT_PATH].as_str().unwrap(), _tmp_env.temp_dir.display().to_string());
    assert_eq!(updated_properties[PROP_TEST_PATH].as_str().unwrap(), _tmp_env.temp_dir.join(DEFAULT_TEST_DIR).display().to_string());

    _tmp_env.tear_down();
}