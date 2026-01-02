use std::collections::HashMap;
use dotenv::dotenv as dotenv_linux;
use dotenvy::dotenv as dotenv_windows;

static STEAM_API_KEY_ENV : &str = "STEAM_API_KEY";
pub static RECIPIENT_EMAIL_ENV : &str = "RECIPIENT_EMAIL";
pub static SMTP_HOST_ENV : &str = "SMTP_HOST";
pub static SMTP_PORT_ENV : &str = "SMTP_PORT";
pub static SMTP_EMAIL_ENV : &str = "SMTP_EMAIL";
pub static SMTP_USERNAME_ENV : &str = "SMTP_USERNAME";
static SMTP_PASSWORD_ENV : &str = "SMTP_PWD";
pub static PROJECT_PATH: &str = "PROJECT_PATH";
pub static TEST_PATH: &str = "TEST_PATH";

pub fn get_variables() -> HashMap<String, String> {
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }
    let recipient = std::env::var(RECIPIENT_EMAIL_ENV).expect("RECIPIENT_EMAIL must be set");
    let smtp_host = std::env::var(SMTP_HOST_ENV).expect("SMTP_HOST must be set");
    let smtp_port = std::env::var(SMTP_PORT_ENV).expect("SMTP_PORT must be set");
    let smtp_email = std::env::var(SMTP_EMAIL_ENV).expect("SMTP_EMAIL must be set");
    let smtp_user = std::env::var(SMTP_USERNAME_ENV).expect("SMTP_USERNAME must be set");
    //let smtp_pwd = std::env::var(SMTP_PASSWORD_ENV).expect("SMTP_PWD must be set");
    let cwd = std::env::current_dir().unwrap().display().to_string();
    let project_path = std::env::var(PROJECT_PATH).unwrap_or_else(|_| cwd);
    let vars: HashMap<String, String> = HashMap::from([
        (RECIPIENT_EMAIL_ENV.to_string(), recipient),
        (SMTP_HOST_ENV.to_string(), smtp_host),
        (SMTP_PORT_ENV.to_string(), smtp_port),
        (SMTP_EMAIL_ENV.to_string(), smtp_email),
        (SMTP_USERNAME_ENV.to_string(), smtp_user),
        (PROJECT_PATH.to_string(), project_path),
    ]);
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