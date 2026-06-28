use std::fmt::format;
use std::time::SystemTime;
use chrono::offset::Utc;
use chrono::DateTime;

pub enum LogLevel {
    DEBUG,
    WARN,
    INFO,
    ERROR
}

pub fn message_builder(msg: &str, log_level: LogLevel) -> String {
    
    let now = SystemTime::now();
    let timestamp: DateTime<Utc> = now.into();
    timestamp.format("%Y-%m-%d %T").to_string();

    let log_level_str = match log_level {
        LogLevel::DEBUG => "DEBUG",
        LogLevel::WARN => "WARN",
        LogLevel::INFO => "INFO",
        LogLevel::ERROR => "ERR",
    };

    format!("[{}] {} - {}\n", timestamp, log_level_str, msg)
}