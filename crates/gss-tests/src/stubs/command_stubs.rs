use std::path::PathBuf;

use constants::operations::thresholds::THRESHOLDS;
use dotenv::dotenv as dotenv_linux;
use dotenvy::dotenv as dotenv_windows;
use file_types::{csv, general};
use serde_json::json;
use structs::internal::data::{GameThreshold, SimpleGameThreshold};

use crate::utils::file_operations::{self, get_threshold_path};


pub fn add_fake_threshold(alias: &str, title: &str, price: f64) {
    add_threshold(alias, title, 1, 2, "c", price);
}

pub fn add_threshold(alias: &str, title: &str, steam_id: u32, gog_id: u32, ms_id: &str, price: f64) {
    let game_thresh = GameThreshold{
        title: String::from(title),
        alias: String::from(alias),
        steam_id,
        gog_id,
        microsoft_store_id: String::from(ms_id),
        currency: String::from("USD"),
        desired_price: price,
    };
    let mut thresholds = file_operations::load_thresholds();
    let mut unique = true;
    for threshold in &thresholds {
        if threshold.title == title {
            unique = false;
            break;
        }
    }
    if unique { thresholds.push(game_thresh); }
    match file_operations::load_threshold_data() {
        Ok(data) => {
            let mut thresholds_data = data;
            *thresholds_data.get_mut(THRESHOLDS.to_string()).unwrap() = json!(thresholds);
            let thresholds_str = serde_json::to_string_pretty(&thresholds_data);
            general::write_to_file(get_threshold_path(), thresholds_str.expect("Cannot update thresholds for testing"));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

pub fn get_sample_csv(filename: &str) -> String {
    let thresholds = vec![
        SimpleGameThreshold{ name: String::from("Hollow Knight"), price: 9.99 },
        SimpleGameThreshold{ name: String::from("Stardew Valley"), price: 7.99 },
    ];
    if cfg!(target_os = "windows") { dotenv_windows().ok(); }
    else if cfg!(target_os = "linux") { dotenv_linux().ok(); }
    //let test_path = std::env::var("TEST_PATH").unwrap_or(String::from("."));
    let test_path = properties::get_data_path();
    let path_buf: PathBuf = [&test_path, "data", filename].iter().collect();
    let csv_path = path_buf.display().to_string();
    csv::generate_csv(&csv_path, thresholds);
    csv_path
}