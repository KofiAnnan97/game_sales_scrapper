use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use regex::Regex;

use constants::operations::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};
use structs::internal::enums::GameStore;
// use constants::stores::microsoft_store::BASE_URL as MS_BASE_URL;
// use stores::pc::microsoft_store::{self, MockMicrosoftStoreApi};
// use stores::pc::gog::MockGogApi;
// use stores::pc::steam::MockSteamApi;
use crate::stubs::command_stubs;
use crate::utils::file_operations;

// Sample Game Data IDs
static E33_GAME_TITLE: &str = "Clair Obscur: Expedition 33";
static E33_STEAM_ID: u32 = 1903340;
static E33_GOG_ID: u32 = 2125022825;
static E33_MS_ID: &str = "9ppt8k6gqhrz";

// Regex patterns
static SELECT_STORES_PRTN: &str = r"\[(X|\s)\]\s+(.*)";
static GAME_THRESH_PTRN: &str = r"-\s+(.*)\s\[.*\]\s+=>\s+(\d+.\d+|\d+)";
static PRICE_CHECK_PTRN: &str = r"-\s(?<title>.*)\s:\s\d+.\d+\s->\s\d+.\d+\s\(";

fn setup(){
    file_operations::clear_thresholds();
    file_operations::clear_settings();
}

#[test]
fn config_cmd() {
    setup();
    let _ = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","config","settings","-s","-g","-e","0"])
        .output()
        .expect("failed to execute process");
    let stores = file_operations::load_stores();
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
    let are_aliases_enabled = file_operations::load_alias_state();
    assert_eq!(false, are_aliases_enabled, "Aliases should not be enabled in settings" );
    file_operations::teardown();
}

#[tokio::test]
#[ignore = "Not yet implemented"]
async fn add_cmd() {
    setup();
    // Check that add fails without config setup
    // let price_str = "19.99";
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
        .args(["run","--release","-p","gss-cli","--","config","settings","-a","-e","1"])
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
#[ignore = "Not yet implemented"]
async fn bulk_insert_cmd() {
    setup();
    let filename = "bulk-insert-test.csv";
    let _csv_path = command_stubs::get_sample_csv(filename);

    // Update settings
    let _ = Command::new("cargo")
        .args(["run","release","-p","gss-cli","--","config","settings","-a","-e","0"])
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
    setup();
    let title = "A single game";
    let alias = "ASG";
    let price = 69.99;
    command_stubs::add_fake_threshold(alias, title, price);
    
    // update threshold using game title
    let mut new_price = "19.99";
    let _ = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","update","price","-t",title,"-p",new_price])
        .output()
        .expect("failed to execute process");
    let mut thresholds = file_operations::load_thresholds();
    assert_eq!(1, thresholds.len(), "There should only be 1 threshold");
    assert_eq!(title, thresholds[0].title, "The game title should be {title} not {}", thresholds[0].title);
    assert_eq!(new_price.parse::<f64>().unwrap(), thresholds[0].desired_price, "The desired price should be {} not {}", new_price, thresholds[0].desired_price);

    // update price using fuzzy matching
    new_price = "24.99";
    let mut fuzzy_output = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","update","price","-t",&title[0..title.len()-2],"-p",new_price])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");
    if fuzzy_output.stdout.is_some() {
        let stdin = fuzzy_output.stdin.as_mut().expect("Failed to open stdin.");
        stdin.write_all(b"0\n").expect("failed to send input");
    }
    let choice_output = fuzzy_output.wait_with_output().expect("Failed to wait for process to complete.");
    println!("STDOUT: {}", String::from_utf8_lossy(&choice_output.stdout));
    eprintln!("STDERR: {}", String::from_utf8_lossy(&choice_output.stderr));
    thresholds = file_operations::load_thresholds();
    assert_eq!(title, thresholds[0].title, "The game title should be {title} not {}", thresholds[0].title);
    assert_eq!(new_price.parse::<f64>().unwrap(), thresholds[0].desired_price, "The desired price should be {} not {}", new_price, thresholds[0].desired_price);

    // update price using alias
    new_price = "34.99";
    let _ = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","update","price","-t",alias,"-p",new_price])
        .output()
        .expect("failed to execute process");
    thresholds = file_operations::load_thresholds();
    assert_eq!(alias, thresholds[0].alias, "The game alias should be {alias} not {}", thresholds[0].alias);
    assert_eq!(new_price.parse::<f64>().unwrap(), thresholds[0].desired_price, "The desired price should be {} not {}", new_price, thresholds[0].desired_price);
    file_operations::teardown();
}

#[test]
fn update_alias_cmd() {
    setup();
    let title = "A single game";
    let alias = "ASG";
    let price = 69.99;
    let mut new_alias = "New ASG";
    command_stubs::add_fake_threshold(alias, title, price);
    
    // update threshold alias using game title
    let _ = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","update","alias","-t",title,"-a",new_alias])
        .output()
        .expect("failed to execute process");
    let mut thresholds = file_operations::load_thresholds();
    assert_eq!(1, thresholds.len(), "There should only be 1 threshold");
    assert_eq!(title, thresholds[0].title, "The game title should be {title} not {}", thresholds[0].title);
    assert_eq!(new_alias, thresholds[0].alias, "The game alias should be {} not {}", new_alias, thresholds[0].alias);

    // update threshold alias using fuzzy matching
    new_alias = "New New Alias";
    let mut fuzzy_output = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","update","alias","-t",&title[0..title.len()-2],"-a",new_alias])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");
    if fuzzy_output.stdout.is_some() {
        let stdin = fuzzy_output.stdin.as_mut().expect("Failed to open stdin.");
        stdin.write_all(b"0\n").expect("failed to send input");
    }
    let choice_output = fuzzy_output.wait_with_output().expect("Failed to wait for process to complete.");
    println!("STDOUT: {}", String::from_utf8_lossy(&choice_output.stdout));
    eprintln!("STDERR: {}", String::from_utf8_lossy(&choice_output.stderr));
    thresholds = file_operations::load_thresholds();
    assert_eq!(title, thresholds[0].title, "The game title should be {title} not {}", thresholds[0].title);
    assert_eq!(new_alias, thresholds[0].alias, "The game alias should be {} not {}", new_alias, thresholds[0].alias);
}

#[test]
fn remove_cmd() {
    setup();
    let title = "Soon to be removed";
    let alias = "SR";
    let price = 69.99;
    command_stubs::add_fake_threshold(alias, title, price);

    // Remove threshold by title
    let _ = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","remove","-t", title])
        .output()
        .expect("failed to execute process");
    let mut thresholds = file_operations::load_thresholds();
    assert_eq!(0, thresholds.len(), "There should not be any thresholds present");

    // Remove threshold using fuzzy matching
    command_stubs::add_fake_threshold(alias, title, price);
    let mut fuzzy_output = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","remove","-t", &title[0..title.len()-2]])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");
    if fuzzy_output.stdout.is_some() {
        let stdin = fuzzy_output.stdin.as_mut().expect("Failed to open stdin.");
        stdin.write_all(b"0\n").expect("failed to send input");
    }
    let choice_output = fuzzy_output.wait_with_output().expect("Failed to wait for process to complete.");
    println!("STDOUT: {}", String::from_utf8_lossy(&choice_output.stdout));
    eprintln!("STDERR: {}", String::from_utf8_lossy(&choice_output.stderr));
    thresholds = file_operations::load_thresholds();
    assert_eq!(0, thresholds.len(), "There should not be any thresholds present");

    // Remove threshold by alias
    command_stubs::add_fake_threshold(alias, title, price);
    let _ = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","remove","-t", alias])
        .output()
        .expect("failed to execute process");
    thresholds = file_operations::load_thresholds();
    assert_eq!(0, thresholds.len(), "There should not be any thresholds present");
    file_operations::teardown();
}

#[test]
fn list_selected_stores_cmd() {
    setup();
    let _ = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","config","settings","-m"])
        .output()
        .expect("failed to execute process");

    let ss_out = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","--list-selected-stores", "--test_flag"])
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
        (" ", GameStore::STEAM.get_name()),
        (" ", GameStore::GOOD_OLD_GAMES.get_name()),
        ("X", GameStore::MICROSOFT_STORE_PC.get_name()),
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
    file_operations::teardown();
}

#[test]
fn list_thresholds_cmd() {
    setup();
    let title = "Listed game #1";
    let alias = "LG1";
    let price = 69.99;
    command_stubs::add_fake_threshold(alias, title, price);

    let lt_out = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","--list-thresholds"])
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
    file_operations::teardown();
}

#[tokio::test]
async fn check_prices() {
    setup();
    command_stubs::add_threshold("E33", E33_GAME_TITLE, E33_STEAM_ID, E33_GOG_ID, E33_MS_ID, 9999.99);
    // WORK IN PROGRESS: Need to figure out how to mock the API calls that happen when check-prices is ran.
    // let mut steam_mock = MockSteamApi::new();
    // steam_mock.expect_get_price_details()
    //     .withf(|steam_id| steam_id == &E33_STEAM_ID)
    //     .return_once(|_|  Ok(command_stubs::get_steam_price_check(E33_GAME_TITLE, 9999.99, 9.99)));

    // let mut gog_mock = MockGogApi::new();
    // gog_mock.expect_get_price_details_v2()
    //     .withf(|title| title == E33_GAME_TITLE)
    //     .return_once(|_|  Some(command_stubs::get_gog_price_check(E33_GAME_TITLE, 9999.99, 9.99)));

    // let mut ms_mock = MockMicrosoftStoreApi::new();
    // ms_mock.expect_get_price_details()
    //     .withf(|xbox_id| xbox_id == E33_MS_ID)
    //     .return_once(|_| Some(command_stubs::get_ms_price_check(E33_GAME_TITLE, 9999.99, 9.99)));
    let cp_out = Command::new("cargo")
        .args(["run","--release","-p","gss-cli","--","--check-prices"])
        .output()
        .expect("failed to execute process");
    println!("{:?}", cp_out);
    let output = str::from_utf8(&cp_out.stdout).unwrap_or_default();
    let lines = output.split("\n").collect::<Vec<&str>>();
    let mut curr_store = "";
    let mut games_by_store: HashMap<&str, Vec<&str>> = HashMap::new();
    let steam_name = GameStore::STEAM.get_name();
    games_by_store.insert(steam_name, Vec::new());
    let gog_name = GameStore::GOOD_OLD_GAMES.get_name();
    games_by_store.insert(gog_name, Vec::new());
    let ms_name = GameStore::MICROSOFT_STORE_PC.get_name();
    games_by_store.insert(ms_name, Vec::new());

    let re = Regex::new(PRICE_CHECK_PTRN).unwrap();

    for i in 3..lines.len() {
        if lines[i].contains(steam_name) { curr_store = steam_name; }
        else if lines[i].contains(gog_name) { curr_store = gog_name; }
        else if lines[i].contains(ms_name) { curr_store = ms_name; }
        else if lines[i].is_empty() { continue; }
        else{
            for(_, [game_title]) in re.captures_iter(lines[i]).map(|c| c.extract() ){
                if let Some(games) = games_by_store.get_mut(curr_store){
                    games.push(&game_title);
                }
            }
        }
    }
    // print!("Games by store:{:?}", games_by_store);

    let expected = HashMap::from([
        (steam_name, vec![E33_GAME_TITLE]),
        (gog_name, vec![E33_GAME_TITLE]),
        (ms_name, vec![E33_GAME_TITLE]),
    ]);

    let mut expected_title = expected.get(steam_name).unwrap()[0];
    let mut actual_title = games_by_store.get(steam_name).unwrap()[0];
    assert_eq!(expected_title, actual_title, "{} -> Game title should be {} not {}", steam_name, expected_title, actual_title);
    expected_title = expected.get(gog_name).unwrap()[0];
    actual_title = games_by_store.get(gog_name).unwrap()[0];
    assert_eq!(expected_title, actual_title, "{} -> Game title should be {} not {}", gog_name, expected_title, actual_title);
    expected_title = expected.get(ms_name).unwrap()[0];
    actual_title = games_by_store.get(ms_name).unwrap()[0];
    assert_eq!(expected_title, actual_title, "{} -> Game title should be {} not {}", ms_name, expected_title, actual_title);
    file_operations::teardown();
}