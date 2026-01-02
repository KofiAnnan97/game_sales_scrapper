use std::fs::{self, metadata, read_to_string};
use std::path::{Path, PathBuf};
use serde_json::{json, Value, Result};

use file_types::common;
pub mod env_vars;
use env_vars::{RECIPIENT_EMAIL_ENV, SMTP_HOST_ENV, SMTP_PORT_ENV, SMTP_EMAIL_ENV, SMTP_USERNAME_ENV};

// Environment variable names
pub static PROJECT_PATH_ENV : &str = "PROJECT_PATH";
pub static TEST_PATH_ENV : &str = "TEST_PATH";

// Directories
static DATA_DIR : &str = "data";

// Filenames
static PROPERTIES_FILENAME : &str = "properties.json";

// Properties variable names
static PROP_STEAM_API_KEY : &str = "steam_api_key";
static PROP_RECIPIENT_EMAIL : &str = "recipient_email";
static PROP_SMTP_HOST : &str = "smtp_host";
static PROP_SMTP_PORT : &str = "smtp_port";
static PROP_SMTP_EMAIL : &str = "smtp_email";
static PROP_SMTP_USERNAME : &str = "smtp_user";
static PROP_SMTP_PASSWORD : &str = "smtp_pwd";
static PROP_PROJECT_PATH : &str = "project_path";
static PROP_TEST_MODE : &str = "test_mode";

pub fn get_properties_path() -> String{
    let project_path = env_vars::get_project_path();
    let mut path_buf: PathBuf = [&project_path, DATA_DIR].iter().collect();
    let data_path = path_buf.display().to_string();
    if !Path::new(&data_path).is_dir() { let _ = fs::create_dir(&data_path); }
    path_buf = [data_path, PROPERTIES_FILENAME.to_string()].iter().collect();
    let properties_path = path_buf.display().to_string();
    let path_str = common::get_path(&properties_path);  //Creates file if it does not exist already
    match metadata(&path_str){
        Ok(md) => {
            if md.len() == 0 {
                let vars = env_vars::get_variables();
                let port: u16 = vars.get(SMTP_PORT_ENV).unwrap().parse().unwrap();
                let properties = json!({
                    PROP_RECIPIENT_EMAIL: vars.get(RECIPIENT_EMAIL_ENV).unwrap(),
                    PROP_SMTP_HOST: vars.get(SMTP_HOST_ENV).unwrap(),
                    PROP_SMTP_PORT: port,
                    PROP_SMTP_EMAIL: vars.get(SMTP_EMAIL_ENV).unwrap(),
                    PROP_SMTP_USERNAME: vars.get(SMTP_USERNAME_ENV).unwrap(),
                    PROP_PROJECT_PATH: vars.get(PROJECT_PATH_ENV).unwrap(),
                    PROP_TEST_MODE: 0
                });
                let properties_str = serde_json::to_string_pretty(&properties);
                common::write_to_file(properties_path.to_string(), properties_str.expect("Initial properties could not be created."));
            }
        },
        Err(e) => eprintln!("Error: {}", e)
    }
    path_str
}

pub fn update_properties() {
    let prev_recipient = get_recipient();
    let prev_host = get_smtp_host();
    let prev_port = get_smtp_port();
    let prev_email = get_smtp_email();
    let prev_user = get_smtp_user();
    let prev_project_path = get_project_path();

    let vars = env_vars::get_variables();
    let curr_recipient = vars.get(RECIPIENT_EMAIL_ENV).unwrap().to_string();
    let curr_host = vars.get(SMTP_HOST_ENV).unwrap().to_string();
    let curr_port = vars.get(SMTP_PORT_ENV).unwrap().to_string().parse::<u16>().unwrap();
    let curr_email = vars.get(SMTP_EMAIL_ENV).unwrap().to_string();
    let curr_user = vars.get(SMTP_USERNAME_ENV).unwrap().to_string();
    let curr_project_path = vars.get(PROJECT_PATH_ENV).unwrap().to_string();

    let mut can_update = false;
    if !curr_recipient.is_empty() && prev_recipient != curr_recipient { can_update = true; }
    if !can_update && !curr_host.is_empty() && prev_host != curr_host { can_update = true; }
    if !can_update && prev_port != curr_port { can_update = true; }
    if !can_update && !curr_email.is_empty() && prev_email != curr_email { can_update = true; }
    if !can_update && !curr_user.is_empty() && prev_user != curr_user { can_update = true; }
    if !can_update && !curr_project_path.is_empty() && prev_project_path != curr_project_path { can_update = true; }
    println!("Test mode: {}", get_test_mode());
    if can_update {
        let properties = json!({
            PROP_RECIPIENT_EMAIL: curr_recipient,
            PROP_SMTP_HOST: curr_host,
            PROP_SMTP_PORT: curr_port,
            PROP_SMTP_EMAIL: curr_email,
            PROP_SMTP_USERNAME: curr_user,
            PROP_PROJECT_PATH: curr_project_path.to_string(),
            PROP_TEST_MODE: get_test_mode(),
        });
        let properties_str = serde_json::to_string_pretty(&properties);
        common::write_to_file(get_properties_path(), properties_str.expect("Properties could not be updated."));
    }
}

pub fn load_properties() -> Result<Value> {
    let data = read_to_string(get_properties_path()).unwrap();
    let body: Value = serde_json::from_str(&data)?;
    Ok(body)
}

fn get_string_var(var_name: &str) -> String {
    match load_properties() {
        Ok(properties) => {
            let var = properties.get(var_name).unwrap();
            var.as_str().unwrap().to_string()
        }
        Err(_) => panic!("Failed to load properties file.")
    }
}

fn get_integer_var(var_name: &str) -> i64 {
    match load_properties() {
        Ok(properties) => {
            let var = properties.get(var_name).unwrap();
            match var.as_i64(){
                Some(i) => i,
                None => panic!("Failed to parse \"{}\". Please check that this is an integer.", var_name)
            }
        }
        Err(_) => panic!("Failed to load properties file.")
    }
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

pub fn get_project_path() -> String {
    get_string_var(PROP_PROJECT_PATH)
}

pub fn get_test_mode() -> i16 {
    get_integer_var(PROP_TEST_MODE) as i16
}

pub fn is_testing_enabled() -> bool {
    let state = get_integer_var(PROP_TEST_MODE);
    if state == 1 { true } else { false }
}

pub fn set_test_mode(is_enabled: bool) {
    match load_properties(){
        Ok(data) => {
            let mut properties = data;
            let enabled = if is_enabled { 1 } else { 0 };
            *properties.get_mut(PROP_TEST_MODE).unwrap() = json!(enabled);
            let properties_str = serde_json::to_string_pretty(&properties);
            common::write_to_file(get_properties_path(), properties_str.expect("Test mode properties could not be created."));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn get_data_path() -> String {
    let mut data_path = if is_testing_enabled() { env_vars::get_test_path() } else { get_project_path() };
    let path: PathBuf = [&data_path, DATA_DIR].iter().collect();
    data_path = path.display().to_string();
    //println!("Path: {}", data_path);
    if !Path::new(&data_path).is_dir() { let _ = fs::create_dir(&data_path); }
    data_path
}