use std::ffi::OsStr;
use std::fs;
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};

use constants::operations::logging::APP_SUBDIR;
use files::general;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::views::logs::Screen;

const BATCH_LIMIT : usize = 10;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        let log_data = LogData{
            timestamp: dt.to_string(),
            level: log_level.to_string(),
            screen: screen.to_string(),
            message: msg.into()
        };
        match serde_json::to_string(&log_data) {
            Ok(log_msg) => log_msg + "\n",
            Err(_) => String::new()
        }
    }

    fn update_file(&self) {
        // TODO: Organize by time stamp
        let mut log_block = String::new();
        for log in self.batch.iter() {
            log_block.push_str(log);
        }
        general::append_to_file(&self.file_path, log_block.as_str());
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
        self.batch_limit_exceeded_flush();
    }

    pub fn warn(&mut self, screen: Screen, msg: &str) {
        self.batch.push(self.message_builder(msg, screen, LogLevel::WARN));
        self.batch_limit_exceeded_flush();
    }

    pub fn info(&mut self, screen: Screen, msg: &str) {
        self.batch.push(self.message_builder(msg, screen, LogLevel::INFO));
        self.batch_limit_exceeded_flush();
    }

    pub fn error(&mut self, screen: Screen, msg: &str) {
        self.batch.push(self.message_builder(msg, screen, LogLevel::ERROR));
        self.batch_limit_exceeded_flush();
    }

    pub fn _fatal(&mut self, panic_info: PanicHookInfo) {
        self.batch.push(fatal_message_builder(&panic_info));
        self.batch_limit_exceeded_flush();
    }

    fn batch_limit_exceeded_flush(&mut self) {
        if self.batch.len() > BATCH_LIMIT {
            self.flush();
        }
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

    let log_data = LogData{
        timestamp: dt.to_string(),
        level: FATAL.into(),
        screen: Screen::None.to_string(),
        message: format!("Application panicked at {} with \'{}\'", location, msg)
    };
    match serde_json::to_string(&log_data){
        Ok(log_msg) => log_msg + "\n",
        Err(_) =>String::new(),
    }
}

pub fn get_app_logs() -> Vec<String> {
    let mut log_names: Vec<String> = Vec::new();
    match fs::read_dir(get_log_path()) {
        Ok(dir) => {
            for d in dir {
                if let Some(entry) = d.ok() && entry.file_type().unwrap().is_file() {
                    let file_name = entry.file_name();
                    if let Some(file_name) = file_name.to_str() && is_valid_log_filename(file_name){
                        log_names.push(file_name.to_string());
                    }
                }
            }
            log_names.sort();
            log_names = log_names.into_iter().rev().collect();
        }
        Err(_) => {}
    }
    log_names
}

fn is_valid_log_filename(file_name: &str) -> bool {
    match file_name.strip_suffix(LOG_SUFFIX) {
        Some(timestamp) => NaiveDateTime::parse_from_str(timestamp, DT_STR_FORMAT).is_ok(),
        None => false,
    }
}

pub fn parse_logs(log_str: &str) -> Vec<LogData> {
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
    if !is_valid_log_filename(file_name) || Path::new(file_name).components().count() != 1 {
        return false;
    }

    let log_dir = PathBuf::from(get_log_path());
    let log_file = log_dir.join(file_name);

    if !log_dir.exists() || !log_file.exists() {
        return false;
    }

    general::delete_file(log_file.display().to_string())
}