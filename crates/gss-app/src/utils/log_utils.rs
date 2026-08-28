use std::ffi::OsStr;
use std::fs;
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Utc};

use constants::operations::logging::APP_SUBDIR;
use files::general;
use serde::Deserialize;
use serde_json::from_str;

use crate::views::logs::Screen;

#[derive(Debug, Deserialize, Clone)]
pub struct LogData{
    pub timestamp: String,
    pub level: String,
    pub screen: String,
    pub message: String
}

#[derive(Debug, Clone)]
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

pub struct Logger {
    file_path: String,
    batch: Vec<String>
}

impl Logger {
    pub fn new(file_path: &str) -> Logger {
        Self {
            file_path: file_path.into(),
            batch: Vec::new(),
        }
    }
   
    fn can_update(&self) -> bool {
        !self.file_path.is_empty() && !self.batch.is_empty()
    }

    fn message_builder(&self, msg: &str, screen: Screen, log_level: LogLevel) -> String {
        let dt: DateTime<Utc> = SystemTime::now().into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let msg = format!("{{\"timestamp\": \"{}\", \"level\": \"{}\", \"screen\": \"{}\", \"message\": \"{}\"}}\n", dt, log_level, screen, msg);
        msg
    }

    pub fn get_filename(&self) -> String {
        get_filename(&self.file_path)
    }

    pub fn get_full_logs(&self) -> String {
        let mut log_block = String::new();
        for log in &self.batch {
            log_block.push_str(&format!("{}\n", log));
        }
        general::get_contents(&self.file_path) + &log_block
    }

    pub fn log(&mut self, screen: Screen, level: LogLevel, msg: &str) {
        match level {
            LogLevel::DEBUG => self.debug(screen, msg),
            LogLevel::WARN => self.warn(screen, msg),
            LogLevel::INFO => self.info(screen, msg),
            LogLevel::ERROR => self.error(screen, msg),
        }
    }

    pub fn debug(&mut self, screen: Screen, msg: &str) {
        self.batch.push(self.message_builder(msg, screen, LogLevel::DEBUG));
    }

    pub fn warn(&mut self, screen: Screen, msg: &str) {
        self.batch.push(self.message_builder(msg, screen, LogLevel::WARN));
    }

    pub fn info(&mut self, screen: Screen, msg: &str) {
        self.batch.push(self.message_builder(msg, screen, LogLevel::INFO));
    }

    pub fn error(&mut self, screen: Screen, msg: &str) {
        self.batch.push(self.message_builder(msg, screen, LogLevel::ERROR));
    }

    pub fn _fatal(&mut self, panic_info: PanicHookInfo) {
        self.batch.push(fatal_message_builder(&panic_info));
    }

    pub fn flush(&mut self) {
        if self.can_update() {
            update_file(&self.file_path, &self.batch);
            self.batch.clear();
        }
    }
}

pub fn get_filename(file_path: &str) -> String {
    let path = Path::new(file_path);
    path.file_name().unwrap_or(OsStr::new("")).to_string_lossy().into()
}

pub fn get_log_data(file_path: &str) -> String { 
    let content = general::get_contents(file_path);
    content
} 

pub fn update_file(file_path: &str, batch: &Vec<String>) {
    // TODO: Organize by time stamp
    let mut log_block = String::new();
    for log in batch {
        log_block.push_str(log);
    }
    general::append_to_file(file_path, log_block.as_str());
}

pub fn get_log_path() -> String {
    let path_buf: PathBuf = [properties::get_log_path(), APP_SUBDIR.to_string()].iter().collect();
    let app_logs_path = path_buf.display().to_string();
    let _ = general::create_dir(&app_logs_path);
    app_logs_path
}

pub fn new_log() -> String {
    let app_logs_path = get_log_path();
    let dt: DateTime<Utc> = SystemTime::now().into();
    let timestamp = dt.format("%Y_%m_%d_%H_%M_%T").to_string();
    let filename = format!("{}_app.log", timestamp).replace(":", "_");
    general::write_file(Path::new(&app_logs_path), &filename, "");
    let filepath = Path::new(&app_logs_path).join(&filename);
    filepath.display().to_string()
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

    let msg = format!("{{\"timestamp\": \"{}\", \"level\": \"{}\", \"screen\": \"{}\", \"message\": \"Application panicked at {} with \'{}\'\"}}\n", dt, FATAL, Screen::None, location, msg);
    msg
}

pub fn get_app_logs() -> Vec<String> {
    let mut log_names: Vec<String> = Vec::new();
    match fs::read_dir(get_log_path()) {
        Ok(dir) => {
            for d in dir {
                if let Some(file_type) = d.ok() {
                    log_names.push(file_type.file_name().to_str().unwrap().to_string());
                }
            }
            log_names.sort();
            log_names = log_names.into_iter().rev().collect();
        }
        Err(_) => {}
    }
    log_names
}

pub fn parse_logs(log_str: String) -> Vec<LogData> {
    let mut log_data: Vec<LogData> = Vec::new();
    for line in log_str.lines() {
        match from_str::<LogData>(line.trim()) {
            Ok(data) => log_data.push(data),
            Err(_) => {}
        }
    }
    log_data
}

pub fn delete_log(file_name: &str) -> bool {
    let path_buf: PathBuf = [&get_log_path(), file_name].iter().collect();
    let file_path = path_buf.display().to_string();
    general::delete_file(file_path);
    false
}