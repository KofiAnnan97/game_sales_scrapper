use std::collections::HashMap;
use std::fs::{self, metadata, File};
use std::path::{PathBuf};
use dotenv as env_linux;
use dotenvy as env_windows;
use rand::distr::{Alphanumeric, SampleString};

use file_types::general;
use crate::passwords;
use constants::operations::properties::{DEFAULT_TEST_DIR, CONFIG_DIR, DECRYPT_FILENAME, 
                                        ENV_FILENAME, STEAM_API_KEY_ENV, RECIPIENT_EMAIL_ENV, 
                                        SMTP_HOST_ENV, SMTP_PORT_ENV,SMTP_EMAIL_ENV, 
                                        SMTP_USERNAME_ENV, SMTP_PASSWORD_ENV, PROJECT_PATH_ENV, 
                                        TEST_PATH_ENV};

pub fn get_decrypt_key(project_path: String) -> String{
    let mut path_buf: PathBuf = [&project_path, CONFIG_DIR].iter().collect();
    let mut path_str: String = path_buf.display().to_string();
    general::create_dir(&path_str);
    path_buf = [&path_str, DECRYPT_FILENAME].iter().collect();
    path_str = path_buf.display().to_string();
    let mut key_str = String::new();
    if !path_buf.is_file() { 
        File::create_new(&path_str).expect("Failed to create decrypt key file");
        match metadata(&path_buf){
            Ok(md) => {
                if md.len() == 0 {
                    key_str = Alphanumeric.sample_string(&mut rand::rng(), 32);
                    general::write_to_file(path_str, key_str.to_owned());
                }
            },
            Err(e) => eprintln!("Could not create decrypt key file: {}\n{}", DECRYPT_FILENAME, e)
        }
    } else { key_str = fs::read_to_string(path_str).unwrap_or_default(); }
    key_str
}

pub fn load_dotenv(file: Option<&str>) {
    match file{
        Some(filename) => {
            if cfg!(target_os = "windows") { env_windows::from_filename_override(filename).ok(); }
            else if cfg!(target_os = "linux") { env_linux::from_filename(filename).ok(); }
        },
        None => {
            if cfg!(target_os = "windows") { env_windows::dotenv().ok(); }
            else if cfg!(target_os = "linux") { env_linux::dotenv().ok(); }
        }
    };
}


pub fn get_variables() -> HashMap<String, String> {
    load_dotenv(None);
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut env_path = std::env::current_dir().unwrap();
    env_path.push(ENV_FILENAME);
    if env_path.is_file() {
        let steam_key_plain = std::env::var(STEAM_API_KEY_ENV).expect("STEAM_API_KEY must be set");
        let steam_key_encrypted = passwords::encrypt(get_decrypt_key(get_project_path()).as_str(), steam_key_plain);
        let recipient = std::env::var(RECIPIENT_EMAIL_ENV).expect("RECIPIENT_EMAIL must be set");
        let smtp_host = std::env::var(SMTP_HOST_ENV).expect("SMTP_HOST must be set");
        let smtp_port = std::env::var(SMTP_PORT_ENV).expect("SMTP_PORT must be set");
        let smtp_email = std::env::var(SMTP_EMAIL_ENV).expect("SMTP_EMAIL must be set");
        let smtp_user = std::env::var(SMTP_USERNAME_ENV).expect("SMTP_USERNAME must be set");
        let smtp_pwd_plain = std::env::var(SMTP_PASSWORD_ENV).expect("SMTP_PWD must be set");
        let smtp_pwd_encrypted = passwords::encrypt(get_decrypt_key(get_project_path()).as_str(), smtp_pwd_plain);
        let cwd = std::env::current_dir().unwrap().display().to_string();
        let project_path = std::env::var(PROJECT_PATH_ENV).unwrap_or_else(|_| cwd.clone());
        let path_buf: PathBuf = [&project_path, DEFAULT_TEST_DIR].iter().collect();
        let generic_test_path = path_buf.display().to_string();
        let test_path = std::env::var(TEST_PATH_ENV).unwrap_or(generic_test_path);
        
        vars = HashMap::from([
            (STEAM_API_KEY_ENV.to_string(), steam_key_encrypted),
            (RECIPIENT_EMAIL_ENV.to_string(), recipient),
            (SMTP_HOST_ENV.to_string(), smtp_host),
            (SMTP_PORT_ENV.to_string(), smtp_port),
            (SMTP_EMAIL_ENV.to_string(), smtp_email),
            (SMTP_USERNAME_ENV.to_string(), smtp_user),
            (SMTP_PASSWORD_ENV.to_string(), smtp_pwd_encrypted),
            (PROJECT_PATH_ENV.to_string(), project_path),
            (TEST_PATH_ENV.to_string(), test_path),
        ]);  
    }
    vars
}

pub(crate) fn get_project_path() -> String {
    load_dotenv(None);
    let project_path = std::env::var(PROJECT_PATH_ENV).unwrap_or_else(|_| String::new());
    project_path
}

pub(crate) fn get_test_path() -> String {
    load_dotenv(None);
    let test_path = std::env::var(TEST_PATH_ENV).unwrap_or_else(|_| String::new());
    test_path
}