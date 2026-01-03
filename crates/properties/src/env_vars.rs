use std::collections::HashMap;
use std::fs::{self, metadata, File};
use std::path::PathBuf;
use dotenv::dotenv as dotenv_linux;
use dotenvy::dotenv as dotenv_windows;
use rand::distr::{Alphanumeric, SampleString};

use file_types::common;
use crate::passwords;

// Directories
static DATA_DIR : &str = "data";

// Filename
pub static ENV_FILENAME : &str = ".env";
static DECRYPT_FILENAME : &str = "decrypt.key";

// Environment Variables
pub static STEAM_API_KEY_ENV : &str = "STEAM_API_KEY";
pub static RECIPIENT_EMAIL_ENV : &str = "RECIPIENT_EMAIL";
pub static SMTP_HOST_ENV : &str = "SMTP_HOST";
pub static SMTP_PORT_ENV : &str = "SMTP_PORT";
pub static SMTP_EMAIL_ENV : &str = "SMTP_EMAIL";
pub static SMTP_USERNAME_ENV : &str = "SMTP_USERNAME";
pub static SMTP_PASSWORD_ENV : &str = "SMTP_PWD";
pub static PROJECT_PATH: &str = "PROJECT_PATH";
pub static TEST_PATH: &str = "TEST_PATH";

pub fn get_decrypt_key() -> String{
    let path_buf: PathBuf = [get_project_path().as_str(), DATA_DIR, DECRYPT_FILENAME].iter().collect();
    let path_str = path_buf.display().to_string();
    let mut key_str = String::new();
    if !path_buf.is_file() { 
        File::create_new(&path_str).expect("Failed to create decrypt key file");
        match metadata(&path_buf){
            Ok(md) => {
                if md.len() == 0 {
                    key_str = Alphanumeric.sample_string(&mut rand::rng(), 32);
                    common::write_to_file(path_str, key_str.to_owned());
                }
            },
            Err(e) => eprintln!("Could not create decrypt key file: {}\n{}", DECRYPT_FILENAME, e)
        }
    } else { key_str = fs::read_to_string(path_str).unwrap_or_default(); }
    key_str
}

pub fn get_variables() -> HashMap<String, String> {
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }

    let mut vars: HashMap<String, String> = HashMap::new();
    let mut env_path = std::env::current_dir().unwrap();
    env_path.push(ENV_FILENAME);
    if env_path.is_file() {
        let steam_key_plain = std::env::var(STEAM_API_KEY_ENV).expect("STEAM_API_KEY must be set");
        let steam_key_encrypted = passwords::encrypt(get_decrypt_key().as_str(), steam_key_plain);
        let recipient = std::env::var(RECIPIENT_EMAIL_ENV).expect("RECIPIENT_EMAIL must be set");
        let smtp_host = std::env::var(SMTP_HOST_ENV).expect("SMTP_HOST must be set");
        let smtp_port = std::env::var(SMTP_PORT_ENV).expect("SMTP_PORT must be set");
        let smtp_email = std::env::var(SMTP_EMAIL_ENV).expect("SMTP_EMAIL must be set");
        let smtp_user = std::env::var(SMTP_USERNAME_ENV).expect("SMTP_USERNAME must be set");
        let smtp_pwd_plain = std::env::var(SMTP_PASSWORD_ENV).expect("SMTP_PWD must be set");
        let smtp_pwd_encrypted = passwords::encrypt(get_decrypt_key().as_str(), smtp_pwd_plain);
        let cwd = std::env::current_dir().unwrap().display().to_string();
        let project_path = std::env::var(PROJECT_PATH).unwrap_or_else(|_| cwd);
        vars = HashMap::from([
            (STEAM_API_KEY_ENV.to_string(), steam_key_encrypted),
            (RECIPIENT_EMAIL_ENV.to_string(), recipient),
            (SMTP_HOST_ENV.to_string(), smtp_host),
            (SMTP_PORT_ENV.to_string(), smtp_port),
            (SMTP_EMAIL_ENV.to_string(), smtp_email),
            (SMTP_USERNAME_ENV.to_string(), smtp_user),
            (SMTP_PASSWORD_ENV.to_string(), smtp_pwd_encrypted),
            (PROJECT_PATH.to_string(), project_path),
        ]);
    }
    vars
}

pub fn get_project_path() -> String {
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }
    let cwd = std::env::current_dir().unwrap().display().to_string();
    let project_path = std::env::var(PROJECT_PATH).unwrap_or_else(|_| cwd);
    project_path
}

pub fn get_test_path() -> String {
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }
    let cwd = std::env::current_dir().unwrap().display().to_string();
    let test_path = std::env::var(TEST_PATH).unwrap_or_else(|_| cwd);
    test_path
}