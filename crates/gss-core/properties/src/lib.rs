use std::fs::{metadata, read_to_string};
use std::path::{Path, PathBuf};
use serde_json::{json, Value, Result};

use files::general;
pub mod env_vars;
pub mod passwords;
use constants::operations::properties::{DATA_DIR, CONFIG_DIR, DEFAULT_TEST_DIR, PROPERTIES_FILENAME, ENV_FILENAME,
                                        PROJECT_PATH_ENV, TEST_PATH_ENV, STEAM_API_KEY_ENV, RECIPIENT_EMAIL_ENV, 
                                        SMTP_HOST_ENV, SMTP_PORT_ENV, SMTP_EMAIL_ENV, SMTP_USERNAME_ENV, 
                                        SMTP_PASSWORD_ENV, PROP_STEAM_API_KEY, PROP_RECIPIENT_EMAIL, PROP_SMTP_HOST, 
                                        PROP_SMTP_PORT, PROP_SMTP_EMAIL, PROP_SMTP_USERNAME, PROP_SMTP_PASSWORD, 
                                        PROP_PROJECT_PATH, PROP_TEST_PATH, PROP_TEST_MODE, PROP_SLIDING_STEAM_APPID};
use constants::operations::logging::LOG_DIR;
use crate::env_vars::{get_decrypt_key, EnvVar};
use crate::passwords::Password;

// Retrieve paths

pub fn get_properties_path() -> String{
    let mut project_path = env_vars::get_project_path(); 
    if project_path.is_empty() { project_path = std::env::current_dir().unwrap().display().to_string(); }
    let mut path_buf: PathBuf = [&project_path, CONFIG_DIR].iter().collect();
    let data_path = path_buf.display().to_string();
    general::create_dir(&data_path);
    path_buf = [data_path, PROPERTIES_FILENAME.to_string()].iter().collect();
    let properties_path = path_buf.display().to_string();
    let path_str = general::get_path(&properties_path);
    match metadata(&path_str){
        Ok(md) => {
            if md.len() == 0 {
                let vars = env_vars::get_variables();
                let mut generic_test_path = String::new();
                if vars.is_empty() { 
                    let path_buf: PathBuf = [&project_path, DEFAULT_TEST_DIR].iter().collect();
                    generic_test_path = path_buf.display().to_string();
                    general::create_dir(&generic_test_path);
                }
                let properties = json!({
                    PROP_STEAM_API_KEY : if let Some(EnvVar::Secret(steam_key)) = vars.get(STEAM_API_KEY_ENV) { steam_key.get_value(None) } else { String::new() },
                    PROP_RECIPIENT_EMAIL: if let Some(EnvVar::Text(recipient)) = vars.get(RECIPIENT_EMAIL_ENV) { &recipient } else { "" },
                    PROP_SMTP_HOST: if let Some(EnvVar::Text(host)) = vars.get(SMTP_HOST_ENV) { &host } else { "" },
                    PROP_SMTP_PORT: if let Some(EnvVar::Number(port)) = vars.get(SMTP_PORT_ENV) { *port } else { 0u16 },
                    PROP_SMTP_EMAIL: if let Some(EnvVar::Text(email)) = vars.get(SMTP_EMAIL_ENV) { &email } else { "" },
                    PROP_SMTP_USERNAME: if let Some(EnvVar::Text(user)) = vars.get(SMTP_USERNAME_ENV) { &user } else { "" },
                    PROP_SMTP_PASSWORD: if let Some(EnvVar::Secret(smtp_pwd)) = vars.get(SMTP_PASSWORD_ENV) { smtp_pwd.get_value(None) } else { String::new() },
                    PROP_PROJECT_PATH: &project_path,
                    PROP_TEST_PATH: if let Some(EnvVar::Text(test_path)) = vars.get(TEST_PATH_ENV) { &test_path} else { &generic_test_path },
                    PROP_SLIDING_STEAM_APPID: 0,
                    PROP_TEST_MODE: 0
                });
                let properties_str = serde_json::to_string_pretty(&properties);
                general::write_to_file(properties_path.to_string(), properties_str.expect("Initial properties could not be created."));
            }
        },
        Err(e) => eprintln!("Error: {}", e)
    }
    path_str
}

pub fn get_data_path() -> String {
    let mut data_path = if is_testing_enabled() { get_test_path() } else { get_project_path() };
    let path: PathBuf = [&data_path, DATA_DIR].iter().collect();
    data_path = path.display().to_string();
    //println!("Path: {}", data_path);
    general::create_dir(&data_path);
    data_path
}

pub fn get_config_path() -> String {
    let mut config_path = if is_testing_enabled() { get_test_path() } else { get_project_path() };
    let path: PathBuf = [&config_path, CONFIG_DIR].iter().collect();
    config_path = path.display().to_string();
    general::create_dir(&config_path);
    config_path
}

pub fn get_log_path() -> String {
    let path: PathBuf = [&get_project_path(), LOG_DIR].iter().collect();
    let log_path = path.display().to_string();
    general::create_dir(&log_path);
    log_path
}

// Properties Functions

pub fn update_properties_from_env() {
    let prev_steam_key = get_steam_api_key(false);
    let prev_recipient = get_recipient();
    let prev_host = get_smtp_host();
    let prev_port = get_smtp_port();
    let prev_email = get_smtp_email();
    let prev_user = get_smtp_user();
    let prev_pwd = get_smtp_pwd(false);
    let prev_project_path = get_project_path();
    let prev_test_path = get_test_path();

    let vars = env_vars::get_variables();
    if !vars.is_empty() {
        let curr_steam_key = vars.get(STEAM_API_KEY_ENV).and_then(EnvVar::as_password);
        let curr_recipient = vars.get(RECIPIENT_EMAIL_ENV).and_then(EnvVar::as_str);
        let curr_host = vars.get(SMTP_HOST_ENV).and_then(EnvVar::as_str);
        let curr_port = vars.get(SMTP_PORT_ENV).and_then(EnvVar::as_u16);
        let curr_email = vars.get(SMTP_EMAIL_ENV).and_then(EnvVar::as_str);
        let curr_user = vars.get(SMTP_USERNAME_ENV).and_then(EnvVar::as_str);
        let curr_pwd = vars.get(SMTP_PASSWORD_ENV).and_then(EnvVar::as_password);
        let curr_project_path = vars.get(PROJECT_PATH_ENV).and_then(EnvVar::as_str);
        let curr_test_path = vars.get(TEST_PATH_ENV).and_then(EnvVar::as_str);

        let key_str = env_vars::get_decrypt_key(env_vars::get_project_path());
        let steam_api_key_updated = curr_steam_key.is_some() && prev_steam_key != curr_steam_key.unwrap().get_value(Some(&key_str));
        let smtp_pwd_updated = curr_pwd.is_some() && prev_pwd != curr_pwd.unwrap().get_value(Some(&key_str));

        let properties_changed = steam_api_key_updated || smtp_pwd_updated
            || (curr_recipient.is_some() && prev_recipient != curr_recipient.unwrap())
            || (curr_host.is_some() && prev_host != curr_host.unwrap())
            || (curr_port.is_some() && prev_port != curr_port.unwrap())
            || (curr_email.is_some() && prev_email != curr_email.unwrap())
            || (curr_user.is_some() && prev_user != curr_user.unwrap())
            || (curr_project_path.is_some() && Path::new(curr_project_path.unwrap()).is_dir()
                && prev_project_path != curr_project_path.unwrap())
            || (curr_test_path.is_some() && Path::new(curr_test_path.unwrap()).is_dir()
                && prev_test_path != curr_test_path.unwrap());

        if properties_changed {
            let properties = json!({
                PROP_STEAM_API_KEY : if steam_api_key_updated { curr_steam_key.unwrap().get_value(None) } else { get_string_var(PROP_STEAM_API_KEY) },
                PROP_RECIPIENT_EMAIL: if curr_recipient.is_some() && &prev_recipient != curr_recipient.unwrap() { curr_recipient.unwrap() } else { &prev_recipient },
                PROP_SMTP_HOST: if curr_host.is_some() && &prev_host == curr_host.unwrap() { curr_host.unwrap() } else { &prev_host },
                PROP_SMTP_PORT: if curr_port.is_some() && prev_port != curr_port.unwrap() {curr_port.unwrap() } else { prev_port },
                PROP_SMTP_EMAIL: if curr_email.is_some() && &prev_email != curr_email.unwrap() { curr_email.unwrap() } else { &prev_email },
                PROP_SMTP_USERNAME: if curr_user.is_some()  && &prev_user != curr_user.unwrap() { curr_user.unwrap() } else { &prev_user },
                PROP_SMTP_PASSWORD: if smtp_pwd_updated { curr_pwd.unwrap().get_value(None) } else { get_string_var(PROP_SMTP_PASSWORD) },
                PROP_PROJECT_PATH: if curr_project_path.is_some() && Path::new(curr_project_path.unwrap()).is_dir(){ curr_project_path.unwrap() } else { &prev_project_path },
                PROP_TEST_PATH: if curr_test_path.is_some() && Path::new(curr_test_path.unwrap()).is_dir() { curr_test_path.unwrap() } else { &prev_test_path },
                PROP_SLIDING_STEAM_APPID: get_sliding_steam_appid(),
                PROP_TEST_MODE: get_test_mode(),
            });
            let properties_str = serde_json::to_string_pretty(&properties);
            general::write_to_file(get_properties_path(), properties_str.expect("Properties could not be updated."));
        }
    } 
    else { eprintln!("Cannot find environment variables. Missing file: {ENV_FILENAME}."); }
}

pub fn load_properties() -> Result<Value> {
    let data = read_to_string(get_properties_path()).unwrap();
    let body: Value = serde_json::from_str(&data)?;
    Ok(body)
}

// Getters

fn get_secret_var(var_name: &str) -> Option<Password> {
    match load_properties() {
        Ok(properties) => match properties.get(var_name) {
            Some(result) => match serde_json::to_string(result) {
                Ok(val) => {
                    let trimmed_val = &val[1..val.len()-1];
                    Some(Password::new_encrypted(trimmed_val.to_string()))
                },
                Err(e) => {
                    eprintln!("Could not extract string from property, \"{}\", {}", var_name, e);
                    None
                },
            },
            None => {
                eprintln!("Warning: Property \"{}\" is empty/does not exist.", var_name);
                None
            }        
        }
        Err(_) => panic!("Failed to load secret from properties file")
    }
}

fn get_string_var(var_name: &str) -> String {
    match load_properties() {
        Ok(properties) => {
            let var = match properties.get(var_name) {
                Some(result) => result.as_str().unwrap().to_string(),
                None => {
                    eprintln!("Warning: Property \"{}\" is empty/does not exist.", var_name);
                    String::new()
                }
            };
            var
        }
        Err(_) => panic!("Failed to load string from properties file.")
    }
}

fn get_integer_var(var_name: &str) -> i64 {
    let default_int : i64 = 0;
    match load_properties() {
        Ok(properties) => {
            //let var = properties.get(var_name);
            match properties.get(var_name){
                Some(var) => var.as_i64().unwrap_or(default_int),
                None => {
                    eprintln!("Failed to parse \"{}\" (defaulting to {}). Please check that this is an integer.",
                              var_name, default_int);
                    default_int
                }
            }
        }
        Err(_) => panic!("Failed to load integer from properties file.")
    }
}

pub fn get_steam_api_key(hidden: bool) -> String {
    let steam_api_key = get_secret_var(PROP_STEAM_API_KEY);
    
    if steam_api_key.is_some() { 
        let mut key = steam_api_key.unwrap();
        if hidden {
            key.set_state(passwords::PasswordState::Hidden);
        } else {
            key.set_state(passwords::PasswordState::PlainText);
        }
        
        let key_str = get_decrypt_key(get_project_path());
        key.get_value(Some(&key_str))
    }
    else { String::new() }
}

pub fn get_recipient() -> String {
    get_string_var(PROP_RECIPIENT_EMAIL)
}

pub fn get_smtp_email() -> String {
    get_string_var(PROP_SMTP_EMAIL)
}

pub fn get_smtp_host() -> String {
    get_string_var(PROP_SMTP_HOST)
}

pub fn get_smtp_port() -> u16 {
    get_integer_var(PROP_SMTP_PORT) as u16
}

pub fn get_smtp_user() -> String {
    get_string_var(PROP_SMTP_USERNAME)
}

pub fn get_smtp_pwd(hidden: bool) -> String {
    let smtp_pwd = get_secret_var(PROP_SMTP_PASSWORD);
    if smtp_pwd.is_some() { 
        let mut key = smtp_pwd.unwrap();
        if hidden {
            key.set_state(passwords::PasswordState::Hidden);
        } else {
            key.set_state(passwords::PasswordState::PlainText);
        }
        let key_str = get_decrypt_key(get_project_path());
        key.get_value(Some(&key_str)) 
    } 
    else { String::new() }
}

pub fn get_project_path() -> String {
    let mut project_path = get_string_var(PROP_PROJECT_PATH);
    if project_path.is_empty() { env_vars::get_project_path(); }
    if !Path::new(&project_path).is_dir() {
        if !project_path.is_empty() {
            eprintln!("Directory does not exist: '{}'. Project path set to current working directory.", &project_path);
        }
        project_path = std::env::current_dir().unwrap().display().to_string();
        set_project_path(&project_path);
    }
    project_path
}

pub fn get_test_path() -> String {
    let mut test_path = get_string_var(PROP_TEST_PATH);
    if test_path.is_empty() { test_path = env_vars::get_test_path(); }
    if !Path::new(&test_path).is_dir() { 
        if !test_path.is_empty() {
            eprintln!("Directory does not exist: '{}'. Default test path set.", &test_path);
        }
        let path_buf: PathBuf = [&get_project_path(), DEFAULT_TEST_DIR].iter().collect();
        test_path = path_buf.display().to_string();
        general::create_dir(&test_path);
        set_test_path(&test_path);
    }
    test_path
}

pub fn get_sliding_steam_appid() -> u32{
    let steam_appid = get_integer_var(PROP_SLIDING_STEAM_APPID);
    if steam_appid > 0 { steam_appid as u32 } else { 0 }
}

pub fn get_test_mode() -> i16 {
    get_integer_var(PROP_TEST_MODE) as i16
}

pub fn is_testing_enabled() -> bool {
    let state = get_integer_var(PROP_TEST_MODE);
    if state == 1 { true } else { false }
}

// Setters

pub fn set_steam_api_key(api_key: String) {
    match load_properties() {
        Ok(data) => {
            let mut properties = data;
            let steam_key = Password::new(&get_decrypt_key(get_project_path()), api_key);
            *properties.get_mut(PROP_STEAM_API_KEY).unwrap() = json!(&steam_key.get_value(None));
            let properties_str = serde_json::to_string_pretty(&properties);
            general::write_to_file(get_properties_path(), properties_str.expect("Sliding steam appid property could not be created/updated."));
        }
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn set_recipient(email_str: &str) {
    match load_properties() {
        Ok(data) => {
            let mut properties = data;
            *properties.get_mut(PROP_RECIPIENT_EMAIL).unwrap() = json!(email_str);
            let properties_str = serde_json::to_string_pretty(&properties);
            general::write_to_file(get_properties_path(), properties_str.expect("recipient email property could not be created/updated."));
        }
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn set_stmp_vars(host: String, port: u16, email: String, user: String, pass: String) {
    match load_properties() {
        Ok(data) => {
            let mut properties = data;
            if !host.is_empty() { *properties.get_mut(PROP_SMTP_HOST).unwrap() = json!(host); }
            if port != 0 { *properties.get_mut(PROP_SMTP_PORT).unwrap() = json!(port); }
            if !email.is_empty() { *properties.get_mut(PROP_SMTP_EMAIL).unwrap() = json!(email); }
            if !user.is_empty() { *properties.get_mut(PROP_SMTP_USERNAME).unwrap() = json!(user); }
            if !pass.is_empty() { 
                let smtp_pwd = Password::new(&get_decrypt_key(get_project_path()), pass);
                *properties.get_mut(PROP_SMTP_PASSWORD).unwrap() = json!(&smtp_pwd.get_value(None)); 
            }
            let properties_str = serde_json::to_string_pretty(&properties);
            general::write_to_file(get_properties_path(), properties_str.expect("Refresh steam appid property could not be created."));
        }
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn set_project_path(path: &str) {
    if !Path::new(&path).is_dir() {
        panic!("Project path was not set because '{}' is not a directory.", path);
    }
    match load_properties() {
        Ok(data) => {
            let mut properties = data;
            *properties.get_mut(PROP_PROJECT_PATH).unwrap() = json!(path);
            let properties_str = serde_json::to_string_pretty(&properties);
            general::write_to_file(get_properties_path(), properties_str.expect("Project path property could not be created/updated."));
        }
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn set_test_path(path: &str) {
    if !Path::new(&path).is_dir() {
        eprintln!("Test path was not set because '{}' is not a directory.", path);
        return;
    }
    match load_properties() {
        Ok(data) => {
            let mut properties = data;
            *properties.get_mut(PROP_TEST_PATH).unwrap() = json!(path);
            let properties_str = serde_json::to_string_pretty(&properties);
            general::write_to_file(get_properties_path(), properties_str.expect("Test path property could not be created/updated."));
        }
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn set_sliding_steam_appid(steam_appid: u32) {
    match load_properties() {
        Ok(data) => {
            let mut properties = data;
            *properties.get_mut(PROP_SLIDING_STEAM_APPID).unwrap() = json!(steam_appid);
            let properties_str = serde_json::to_string_pretty(&properties);
            general::write_to_file(get_properties_path(), properties_str.expect("Sliding steam appid property could not be created/updated."));
        }
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn set_test_mode(is_enabled: bool) {
    match load_properties(){
        Ok(data) => {
            let mut properties = data;
            let enabled = if is_enabled { 1 } else { 0 };
            *properties.get_mut(PROP_TEST_MODE).unwrap() = json!(enabled);
            let properties_str = serde_json::to_string_pretty(&properties);
            general::write_to_file(get_properties_path(), properties_str.expect("Test mode property could not be created."));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}