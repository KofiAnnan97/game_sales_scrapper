#[cfg(test)]
use std::collections::HashMap;
use std::{env, fs};
use std::fs::read_to_string;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use regex::Regex;
use dotenv::dotenv as dotenv_linux;
use dotenvy::dotenv as dotenv_windows;
use serde_json::{json, Value};

use structs::internal::data::{GameThreshold, SimpleGameThreshold};
use file_types::{common, csv};
use file_ops::settings;
use constants::operations::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};
use constants::operations::thresholds::THRESHOLDS;
use crate::tests::helper::{self, get_threshold_path};

// Sample Game Data IDs
static E33_GAME_TITLE: &str = "Clair Obscur: Expedition 33";
static E33_STEAM_ID: u32 = 1903340;
static E33_GOG_ID: u32 = 2125022825;
static E33_MS_ID: &str = "9ppt8k6gqhrz";

// Regex patterns
static SELECT_STORES_PRTN: &str = r"\[(X|\s)\]\s+(.*)";
static GAME_THRESH_PTRN: &str = r"-\s+(.*)\s+=>\s+(\d+.\d+|\d+)";
static PRICE_CHECK_PTRN: &str = r"-\s(?<title>.*)\s:\s\d+.\d+\s->\s\d+.\d+\s\(";

fn add_fake_threshold(alias: &str, title: &str, price: f64) {
    add_threshold(alias, title, 1, 2, "c", price);
}

fn add_threshold(alias: &str, title: &str, steam_id: u32, gog_id: u32, ms_id: &str, price: f64) {
    let game_thresh = GameThreshold{
        title: String::from(title),
        alias: String::from(alias),
        steam_id,
        gog_id,
        microsoft_store_id: String::from(ms_id),
        currency: String::from("USD"),
        desired_price: price,
    };
    let mut thresholds = helper::load_thresholds();
    let mut unique = true;
    for threshold in &thresholds {
        if threshold.title == title {
            unique = false;
            break;
        }
    }
    if unique { thresholds.push(game_thresh); }
    match helper::load_threshold_data() {
        Ok(data) => {
            let mut thresholds_data = data;
            *thresholds_data.get_mut(THRESHOLDS.to_string()).unwrap() = json!(thresholds);
            let thresholds_str = serde_json::to_string_pretty(&thresholds_data);
            common::write_to_file(get_threshold_path(), thresholds_str.expect("Cannot update thresholds for testing"));
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}

fn get_sample_csv(filename: &str) -> String {
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

#[test]
fn config_cmd() {
    helper::clear_settings(); 
    let _ = Command::new("cargo")
        .args(["run","--release","--","config","settings","-s","-g","-e","0"])
        .output()
        .expect("failed to execute process");
    let stores = helper::load_stores();
    let mut steam_present = false;
    let mut gog_present = false;
    let mut ms_present = false;
    //println!("{:?}", stores);
    for store_name in stores {
        if store_name == STEAM_STORE_ID { steam_present = true; }
        else if store_name == GOG_STORE_ID { gog_present = true; }
        else if store_name == MICROSOFT_STORE_ID { ms_present = true; }
    }
    assert_eq!(true, steam_present, "Steam should be a selected store");
    assert_eq!(true, gog_present, "Gog should be a selected store");
    assert_ne!(true, ms_present, "MSC should not be a selected store");
    let are_aliases_enabled = helper::load_alias_state();
    assert_eq!(false, are_aliases_enabled, "Aliases should not be enabled in settings" );
    helper::teardown();
}

#[tokio::test]
#[ignore = "Not fully implemented"]
async fn add_cmd() {
    helper::clear_settings();
    helper::clear_thresholds();

    // Check that add fails without config setup
    let price_str = "19.99";
    // let add_wo_config = Command::new("cargo")
    //     .args(["run","--","add","-t",E33_GAME_TITLE,"-p",price_str,"-a","0"])
    //     .output()
    //     .expect("failed to execute proces");
    //
    // let config_err_msg = "Please configure which stores to query";
    // let result_err = str::from_utf8(&add_wo_config.stderr).unwrap_or_default();
    // assert!(result_err.contains(config_err_msg), "Code did not throw error {} for not having settings configured.", config_err_msg);

    // Update settings
    let _ = Command::new("cargo")
        .args(["run","--release","--","config","settings","-a","-e","1"])
        .output()
        .expect("failed to execute proces");

    // Add value
    // let mut add_process = Command::new("cargo")
    //     .args(["run","--","add","-t",E33_GAME_TITLE,"-p",price_str])
    //     .stdin(Stdio::piped())
    //     .stdout(Stdio::piped())
    //     .spawn()
    //     .expect("failed to execute process");
    //
    // let mut stdin = add_process.stdin.take().expect("failed to open stdin");
    // let mut stdout = add_process.stdout.take().expect("failed to open stdout");
    // stdin.write_all(b"0\n").unwrap();
    //
    // let mut output = Vec::new();
    // stdout.read_to_end(&mut output).expect("Failed to read from stdout");
    // println!("{:?}", output);

    //let exit_status = add_process.wait().expect("Child process wasn't running");
}

#[tokio::test]
#[ignore = "Not fully implemented"]
async fn bulk_insert_cmd() {
    helper::clear_settings();
    helper::clear_thresholds();
    let filename = "bulk-insert-test.csv";
    let csv_path = get_sample_csv(filename);

    // Update settings
    let _ = Command::new("cargo")
        .args(["run","--","config","settings","-a","-e","0"])
        .output()
        .expect("failed to execute proces");

    // let bi_process = if cfg!(target_os = "windows") {
    //     Command::new("cmd")
    //         .args(["/C","cargo","run","--","bulk-insert","-f",&csv_path])
    //         .stdin(Stdio::piped())
    //         .stdout(Stdio::piped())
    //         .spawn()
    //         .expect("failed to execute process")
    // } else {
    //     Command::new("cargo")
    //         .args(["run","--","bulk-insert","-f",&csv_path])
    //         .stdin(Stdio::piped())
    //         .stdout(Stdio::piped())
    //         .spawn()
    //         .expect("failed to execute process")
    // };
}

#[test]
fn update_price_cmd() {
    helper::clear_thresholds();
    helper::clear_settings();
    
    let title = "A single game";
    let alias = "ASG";
    let price = 69.99;
    add_fake_threshold(alias, title, price);
    
    // update threshold using game title
    let mut new_price = "19.99";
    let _ = Command::new("cargo")
        .args(["run","--release","--","update","-t",title,"-p",new_price])
        .output()
        .expect("failed to execute process");
    let mut thresholds = helper::load_thresholds();
    assert_eq!(1, thresholds.len(), "There should only be 1 threshold");
    assert_eq!(title, thresholds[0].title, "The game title should be {title} not {}", thresholds[0].title);
    assert_eq!(new_price.parse::<f64>().unwrap(), thresholds[0].desired_price, "The desired price should be {} not {}", new_price, thresholds[0].desired_price);
    
    // update price using alias
    new_price = "34.99";
    let _ = Command::new("cargo")
        .args(["run","--release","--","update","-t",alias,"-p",new_price])
        .output()
        .expect("failed to execute process");
    thresholds = helper::load_thresholds();
    assert_eq!(1, thresholds.len(), "There should only be 1 threshold");
    assert_eq!(alias, thresholds[0].alias, "The game alias should be {alias} not {}", thresholds[0].alias);
    assert_eq!(new_price.parse::<f64>().unwrap(), thresholds[0].desired_price, "The desired price should be {} not {}", new_price, thresholds[0].desired_price);
    helper::teardown();
}

#[test]
fn remove_cmd() {
    helper::clear_thresholds();
    helper::clear_settings();

    let title = "Soon to be removed";
    let alias = "SR";
    let price = 69.99;
    add_fake_threshold(alias, title, price);

    // Remove threshold by title
    let _ = Command::new("cargo")
        .args(["run","--release","--","remove","-t", title])
        .output()
        .expect("failed to execute process");
    let mut thresholds = helper::load_thresholds();
    assert_eq!(0, thresholds.len(), "There should not be any thresholds present");

    // Remove threshold by alias
    add_fake_threshold(alias, title, price);
    let _ = Command::new("cargo")
        .args(["run","--release","--","remove","-t", alias])
        .output()
        .expect("failed to execute process");
    thresholds = helper::load_thresholds();
    assert_eq!(0, thresholds.len(), "There should not be any thresholds present");
    helper::teardown();
}

#[test]
fn list_selected_stores_cmd() {
    helper::clear_settings();

    let _ = Command::new("cargo")
        .args(["run","--release","--","config","settings","-m"])
        .output()
        .expect("failed to execute process");

    let ss_out = Command::new("cargo")
        .args(["run","--release","--","--list-selected-stores", "--test_flag"])
        .output()
        .expect("failed to execute process");
    println!("{:?}", ss_out);
    let output = str::from_utf8(&ss_out.stdout).unwrap_or_default();
    let re = Regex::new(SELECT_STORES_PRTN).unwrap();
    let mut results = vec![];
    for(_, [choice, store_name]) in re.captures_iter(output).map(|c| c.extract() ){
        results.push((choice, store_name));
    }
    let expected = vec![
        (" ", settings::get_proper_store_name(STEAM_STORE_ID).unwrap()),
        (" ", settings::get_proper_store_name(GOG_STORE_ID).unwrap()),
        ("X", settings::get_proper_store_name(MICROSOFT_STORE_ID).unwrap()),
    ];
    for result in results {
        let idx = expected.iter().position(|threshold| result.1 == threshold.1);
        if !idx.is_none() {
            let i = idx.unwrap();
            assert_eq!(expected[i].0, result.0, "The box for {} should be [{}] not [{}]", result.1, expected[i].0, result.0);
        } else{
            assert!(false, "Something when wrong with option -> [{}] {}", result.0, result.1);
        }
    }
    helper::teardown();
}

#[test]
fn list_thresholds_cmd() {
    helper::clear_thresholds();
    helper::clear_settings();

    let title = "Listed game #1";
    let alias = "LG1";
    let price = 69.99;
    add_fake_threshold(alias, title, price);

    let lt_out = Command::new("cargo")
        .args(["run","--release","--","--list-thresholds"])
        .output()
        .expect("failed to execute process");
    println!("{:?}", lt_out);
    let output = str::from_utf8(&lt_out.stdout).unwrap_or_default();
    let re = Regex::new(GAME_THRESH_PTRN).unwrap();
    let mut results = vec![];
    for(_, [game_title, price]) in re.captures_iter(output).map(|c| c.extract() ){
        results.push((game_title, price));
    }
    let expected = vec![
        (title, price),
    ];
    assert_eq!(expected[0].0, results[0].0, "The game title should be \'{}\' not \'{}\'", expected[0].0, results[0].0);
    assert_eq!(expected[0].1, results[0].1.parse::<f64>().unwrap(), "The game price should be \'{}\' not \'{}\'", expected[0].1, results[0].1);
    helper::teardown();
}

#[tokio::test]
async fn check_prices() {
    helper::clear_thresholds();
    helper::clear_settings();

    add_threshold("E33", E33_GAME_TITLE, E33_STEAM_ID, E33_GOG_ID, E33_MS_ID, 9999.99);
    let cp_out = Command::new("cargo")
        .args(["run","--release","--","--check-prices"])
        .output()
        .expect("failed to execute process");
    println!("{:?}", cp_out);
    let output = str::from_utf8(&cp_out.stdout).unwrap_or_default();
    let lines = output.split("\n").collect::<Vec<&str>>();
    let mut curr_store = "";
    let mut games_by_store: HashMap<&str, Vec<&str>> = HashMap::new();
    let steam_proper = settings::get_proper_store_name(STEAM_STORE_ID).unwrap();
    games_by_store.insert(&steam_proper, Vec::new());
    let gog_proper = settings::get_proper_store_name(GOG_STORE_ID).unwrap();
    games_by_store.insert(&gog_proper, Vec::new());
    let ms_proper = settings::get_proper_store_name(MICROSOFT_STORE_ID).unwrap();
    games_by_store.insert(&ms_proper, Vec::new());

    let re = Regex::new(PRICE_CHECK_PTRN).unwrap();

    for i in 3..lines.len() {
        if lines[i].contains(&steam_proper) { curr_store = &steam_proper; }
        else if lines[i].contains(&gog_proper) { curr_store = &gog_proper; }
        else if lines[i].contains(&ms_proper) { curr_store = &ms_proper; }
        else if lines[i].is_empty() { continue; }
        else{
            for(_, [game_title]) in re.captures_iter(lines[i]).map(|c| c.extract() ){
                if let Some(games) =games_by_store.get_mut(curr_store){
                    games.push(&game_title);
                }
            }
        }
    }
    //print!("{:?}", games_by_store);

    let expected = HashMap::from([
        (steam_proper.as_str(), vec![E33_GAME_TITLE]),
        (gog_proper.as_str(), vec![E33_GAME_TITLE]),
        (ms_proper.as_str(), vec![E33_GAME_TITLE]),
    ]);

    let mut expected_title = expected.get(steam_proper.as_str()).unwrap()[0];
    let mut actual_title = games_by_store.get(steam_proper.as_str()).unwrap()[0];
    assert_eq!(expected_title, actual_title, "{} -> Game title should be {} not {}", steam_proper, expected_title, actual_title);
    expected_title = expected.get(gog_proper.as_str()).unwrap()[0];
    actual_title = games_by_store.get(gog_proper.as_str()).unwrap()[0];
    assert_eq!(expected_title, actual_title, "{} -> Game title should be {} not {}", gog_proper, expected_title, actual_title);
    expected_title = expected.get(ms_proper.as_str()).unwrap()[0];
    actual_title = games_by_store.get(ms_proper.as_str()).unwrap()[0];
    assert_eq!(expected_title, actual_title, "{} -> Game title should be {} not {}", ms_proper, expected_title, actual_title);
    helper::teardown();
}