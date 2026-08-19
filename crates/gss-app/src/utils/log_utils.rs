use std::path::{Path, PathBuf};
use std::time::{SystemTime};
use chrono::offset::Utc;
use chrono::DateTime;
use std::panic::PanicHookInfo;

use files::general;
use constants::operations::logging::*;
use properties::get_log_path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LogLevel {
    DEBUG,
    WARN,
    INFO,
    ERROR
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::DEBUG => write!(f, "DEBUG"),
            LogLevel::WARN => write!(f, "WARN"),
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::ERROR => write!(f, "ERROR"),
        }
    }
}

pub static FATAL : &str = "FATAL";

pub fn new_log() -> String {
    let path_buf: PathBuf = [get_log_path(), APP_SUBDIR.to_string()].iter().collect();
    let app_log_path = path_buf.display().to_string();
    let _ = general::create_dir(&app_log_path);
    let dt: DateTime<Utc> = SystemTime::now().into();
    let timestamp = dt.format("%Y_%m_%d_%H_%M_%T").to_string();
    let filename = format!("{}_app.log", timestamp).replace(":", "_");
    general::write_file(&path_buf, &filename, "");
    let filepath = Path::new(&app_log_path).join(&filename);
    filepath.display().to_string()
}


pub fn message_builder(msg: &str, log_level: LogLevel) -> String {
    let dt: DateTime<Utc> = SystemTime::now().into();
    dt.format("%Y-%m-%d %H:%M:%S").to_string();
    let msg = format!("{{ timestamp: \"{}\", level: \"{}\", message: \"{}\"}}\n", dt, log_level, msg);
    msg
}

pub fn fatal_message_builder(panic_info: &PanicHookInfo<'_>) -> String {
    let payload = panic_info.payload();
    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        String::from("Unknown panic")
    };

    let dt: DateTime<Utc> = SystemTime::now().into();
    dt.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let location = panic_info.location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| String::from("Unknown location"));

    let msg = format!("{{timestamp: \"{}\", level: \"{}\", message: \"Application panicked with \'{}\'\", location:\"{}\"}}\n", dt, FATAL, msg, location);
    msg
}