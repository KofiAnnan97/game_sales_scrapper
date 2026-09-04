use std::ffi::OsStr;
use std::fs;
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};

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

impl LogLevel {
    pub fn get_options() -> Vec<String> {
         vec![
            LogLevel::DEBUG.to_string(),
            LogLevel::WARN.to_string(),
            LogLevel::INFO.to_string(),
            LogLevel::ERROR.to_string(),
            FATAL.into()
        ]
    }

    pub fn get_level_idx(level: String) -> usize {
        match level.as_str() {
            "DEBUG" => 0,
            "WARN" => 1,
            "INFO" => 2,
            "ERROR" => 3,
            "FATAL" => 4,
            _ => 0
        }
    }
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
pub static DT_FORMAT : &str = "%Y-%m-%d %H:%M:%S";
pub static DT_STR_FORMAT : &str = "%Y_%m_%d_%H_%M_%S";
pub static LOG_SUFFIX : &str = "_app.log";

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
        dt.format(DT_FORMAT).to_string();
        let msg = format!("{{\"timestamp\": \"{}\", \"level\": \"{}\", \"screen\": \"{}\", \"message\": \"{}\"}}\n", dt, log_level, screen, msg);
        msg
    }

    fn update_file(&self) {
        let mut log_block = String::new();
        for log in self.batch.iter() {
            log_block.push_str(log);
        }
        general::append_to_file(&self.file_path, log_block.as_str());
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
            self.update_file();
            self.batch.clear();
        }
    }

    #[cfg(test)]
    pub fn get_batch_ref(&self) -> &Vec<String> {
        &self.batch
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

pub fn get_log_path() -> String {
    let path_buf: PathBuf = [properties::get_log_path(), APP_SUBDIR.to_string()].iter().collect();
    let app_logs_path = path_buf.display().to_string();
    let _ = general::create_dir(&app_logs_path);
    app_logs_path
}

pub fn new_log() -> String {
    let app_logs_path = get_log_path();
    let dt: DateTime<Utc> = SystemTime::now().into();
    let timestamp = dt.format(DT_STR_FORMAT).to_string();
    let filename = format!("{}{}", timestamp, LOG_SUFFIX).replace(":", "_");

    let last_log = get_last_log();
    let last_log_dt_str = last_log.strip_suffix(LOG_SUFFIX).unwrap_or_default();
    let last_dt = NaiveDateTime::parse_from_str(&last_log_dt_str, DT_STR_FORMAT);

    match last_dt {
        Ok(val) => {
            if dt - val.and_utc() < TimeDelta::minutes(30){
                let path_buf: PathBuf = [app_logs_path, last_log].iter().collect();
                path_buf.display().to_string()
            } else {
                general::write_file(Path::new(&app_logs_path), &filename, "");
                let filepath = Path::new(&app_logs_path).join(&filename);
                filepath.display().to_string()
            }
        },
        Err(_) => {
            general::write_file(Path::new(&app_logs_path), &filename, "");
            let filepath = Path::new(&app_logs_path).join(&filename);
            filepath.display().to_string()
        }
    }
}

fn get_last_log() -> String {
    let logs = get_app_logs();
    // The list is reversed so the first option is the last log file
    match logs.first() {
        Some(log) => log.to_string(),
        None => String::new()
    }
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
                if let Some(entry) = d.ok() && entry.file_type().unwrap().is_file() {
                    log_names.push(entry.file_name().to_str().unwrap().to_string());
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

pub fn get_timestamp(file_name: &str) -> String {
    match String::from(file_name).strip_suffix(LOG_SUFFIX) {
        Some(dt_str) => {
            let dt_long = NaiveDateTime::parse_from_str(&dt_str, DT_STR_FORMAT);
            dt_long.unwrap_or_default().to_string()
        },
        None => String::new(),
    }
}

pub fn delete_log(file_name: &str) -> bool {
    let path_buf: PathBuf = [&get_log_path(), file_name].iter().collect();
    let file_path = path_buf.display().to_string();
    general::delete_file(file_path);
    false
}