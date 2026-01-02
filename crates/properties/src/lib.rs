use dotenv::dotenv as dotenv_linux;
use dotenvy::dotenv as dotenv_windows;
use std::{env, fs};
use std::fs::{metadata, read_to_string};
use std::path::{Path, PathBuf};
use serde_json::{json, Value, Result};

use file_types::json as json_data;

// Environment variable names
pub static PROJECT_VAR_NAME : &str = "PROJECT_PATH";
pub static TEST_VAR_NAME : &str = "TEST_PATH";

// Filenames
static PROPERTIES_FILENAME : &str = "properties.json";

// Directories
static DATA_DIR : &str = "data";

// Properties variables
static TEST_MODE : &str = "test_mode";

pub fn get_properties_path() -> String{
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }
    let project_path = env::var(PROJECT_VAR_NAME.to_string()).unwrap_or_else(|_| String::from("."));
    let mut path_buf: PathBuf = [&project_path, DATA_DIR].iter().collect();
    let data_path = path_buf.display().to_string();
    if !Path::new(&data_path).is_dir() { let _ = fs::create_dir(&data_path); }
    path_buf = [data_path, PROPERTIES_FILENAME.to_string()].iter().collect();
    let properties_path = path_buf.display().to_string();
    let path_str = json_data::get_path(&properties_path);  //Creates file if it does not exist already
    match metadata(&path_str){
        Ok(md) => {
            if md.len() == 0 {
                let properties = json!({TEST_MODE: 0});
                let properties_str = serde_json::to_string_pretty(&properties);
                json_data::write_to_file(properties_path.to_string(), properties_str.expect("Initial settings could not be created."));
            }
        },
        Err(e) => eprintln!("Error: {}", e)
    }
    path_str
}

pub fn load_properties() -> Result<Value> {
    let data = read_to_string(get_properties_path()).unwrap();
    let body: Value = serde_json::from_str(&data)?;
    Ok(body)
}

pub fn get_test_mode() -> bool {
    let filepath = get_properties_path();
    let data = read_to_string(filepath).unwrap();
    let body : Value = serde_json::from_str(&data).expect("Get test mode - could not convert to JSON");
    let test_mode = serde_json::to_string(&body[TEST_MODE]).unwrap();
    let mut is_enabled = false;
    match serde_json::from_str::<i32>(&test_mode){
        Ok(data) => {
            if data == 1 {
                is_enabled = true;
            } else {
                is_enabled = false;
            }
        },
        Err(e) => eprintln!("Get Test mode Error: {}", e)
    };
    is_enabled
}

pub fn set_test_mode(is_enabled: bool) {
    match load_properties(){
        Ok(data) => {
            let mut properties = data;
            let enabled = if is_enabled { 1 } else { 0 };
            *properties.get_mut(TEST_MODE).unwrap() = json!(enabled);
            let properties_str = serde_json::to_string_pretty(&properties);
            json_data::write_to_file(get_properties_path(), properties_str.expect("Test mode properties could not be created."));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn get_data_path() -> String {
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }
    let path_env = if get_test_mode() { TEST_VAR_NAME.to_string() } else { PROJECT_VAR_NAME.to_string() };
    //println!("Env var: {:?}", &path_env);
    let mut data_path = env::var(path_env).unwrap_or_else(|_| String::from("."));
    let path: PathBuf = [&data_path, DATA_DIR].iter().collect();
    data_path = path.display().to_string();
    //println!("Path: {}", data_path);
    if !Path::new(&data_path).is_dir() {
        let _ = fs::create_dir(&data_path);
    }
    data_path
}