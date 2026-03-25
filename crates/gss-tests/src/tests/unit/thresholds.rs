use file_ops::{settings, thresholds};
use file_types::general;
use properties;
use constants::operations::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};
use constants::operations::thresholds::{THRESHOLD_FILENAME};
use crate::stubs::threshold_stubs;
use crate::utils::file_operations;

fn delete_thresholds() {
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    settings::update_alias_reuse_state(1);
    let mut config_path = properties::get_data_path();
    config_path.push_str("/");
    config_path.push_str(THRESHOLD_FILENAME);
    general::delete_file(config_path);
}

#[tokio::test]
async fn add_steam_game() {
    delete_thresholds();
    let client = reqwest::Client::new();
    let app = threshold_stubs::test_steam_app();
    let game_title = &app.name.clone();
    let game_id = app.app_id;
    thresholds::add_steam_game(game_title.clone(), app, 10.00, &client).await;

    match thresholds::load_thresholds() {
        Ok(thresholds) => {
            assert_eq!(game_title.clone(), thresholds[0].title, "Expected {} not {}", game_title.clone(), thresholds[0].title);
            assert_eq!(game_id, thresholds[0].steam_id, "Expected {} not {}", game_id, thresholds[0].steam_id);
        },
        Err(_) => assert!(false, "Could not find game: {} ({})", game_title.clone(), game_id),
    }
    file_operations::teardown();
}

#[test]
fn add_gog_game() {
    delete_thresholds();
    let game = threshold_stubs::test_gog_game();
    let game_title = &game.title.clone();
    let game_id = game.id.parse::<u32>().unwrap();
    thresholds::add_gog_game(game_title.clone(), &game, 10.00);

    match thresholds::load_thresholds() {
        Ok(thresholds) => {
            assert_eq!(game_title.clone(), thresholds[0].title, "Expected {} not {}", game_title.clone(), thresholds[0].title);
            assert_eq!(game_id, thresholds[0].gog_id, "Expected {} not {}", game_id, thresholds[0].gog_id);
        },
        Err(_) => assert!(false, "Could not find game: {} ({})", game_title.clone(), game_id),
    }
    file_operations::teardown();
}

#[test]
fn add_microsoft_store_game() {
    delete_thresholds();
    let game = threshold_stubs::test_ms_game();
    let game_title = &game.title.clone();
    let game_id = &game.product_id.clone();
    thresholds::add_microsoft_store_game(game_title.clone(), &game, 10.00);

    match thresholds::load_thresholds() {
        Ok(thresholds) => {
            assert_eq!(game_title.clone(), thresholds[0].title, "Expected {} not {}", game_title.clone(), thresholds[0].title);
            assert_eq!(*game_id, thresholds[0].microsoft_store_id, "Expected {} not {}", game_id, thresholds[0].microsoft_store_id);
        },
        Err(_) => assert!(false, "Could not find game: {} ({})", game_title.clone(), game_id),
    }
    file_operations::teardown();
}

#[test]
fn update_alias() {
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    threshold_stubs::add_simple_threshold(&game_title, &game_alias, price);

    // Check that alias is empty
    match thresholds::load_thresholds(){
        Ok(thresholds) =>
            assert_eq!(game_alias, thresholds[0].alias, "Alias should be \'{}\' not \'{}\'.", "", thresholds[0].alias),
        Err(_) => assert!(false, "Could not load the thresholds when alias is expected to be empty.")
    }

    // Check that alias map key is created for game
    match thresholds::load_alias_map(){
        Ok(map) => {
            assert_eq!(true, map.contains_key(&game_alias), "Alias map does not contain key \'{}\'", &game_alias);
            assert_eq!(game_title.clone(), map.get(&game_alias).unwrap()[0], "Title should be \'{}\' not \'{}\'.", game_title.clone(), map.get(&game_alias).unwrap()[0])
        },
        Err(_) => assert!(false, "Could not load the alias map"),
    }

    // Check that new alias is present in threshold
    let new_alias = String::from("new_rg");
    thresholds::update_threshold_alias(game_title.clone(), &new_alias);
    match thresholds::load_thresholds(){
        Ok(thresholds) =>
            assert_eq!(new_alias, thresholds[0].alias, "Alias should be \'{}\' not \'{}\'.", new_alias, thresholds[0].alias),
        Err(_) => assert!(false, "Could not load the thresholds when alias is expected to be {}.", new_alias)
    }

    // Check that alias map is updated 
    match thresholds::load_alias_map() {
        Ok(map) => {
            assert_eq!(false, map.contains_key(&game_alias),  "Alias map should not contain \'{}\'", &game_alias);
            assert_eq!(true, map.contains_key(&new_alias), "Alias map does not contain key \'{}\'", &game_alias);
            assert_eq!(game_title.clone(), map.get(&new_alias).unwrap()[0], "Title should be \'{}\' not \'{}\'.", game_title.clone(), map.get(&new_alias).unwrap()[0]);
        },
        Err(_) => assert!(false, "Could not load the alias map"),
    }

    file_operations::teardown();
}

#[test]
fn update_price() {
    delete_thresholds();
    let first_game = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    threshold_stubs::add_simple_threshold(&first_game, &game_alias, price);

    // Check that new price is present in threshold
    let new_price = 20.00;
    thresholds::update_price(&first_game, new_price);
    match thresholds::load_thresholds(){
        Ok(thresholds) =>
            assert_eq!(new_price, thresholds[0].desired_price, "Price should be \'{}\' not \'{}\'.", new_price, thresholds[0].desired_price),
        Err(_) => assert!(false, "Could not load thresholds when desired price was updated..")
    }

    // Check if the price can be updated for two thresholds with the same alias
    let second_game = String::from("Random Game 2");
    threshold_stubs::add_simple_threshold(&second_game, &game_alias, new_price);
    let last_price = 40.00;
    thresholds::update_price(&game_alias, last_price);
    match thresholds::load_thresholds(){
        Ok(thresholds) =>{
            assert_eq!(2, thresholds.len(), "The number of thresholds should be 2 not {}", thresholds.len());
            assert_eq!(last_price, thresholds[0].desired_price, "Price should be \'{}\' not \'{}\' for {}.", last_price, thresholds[0].desired_price, thresholds[0].title);
            assert_eq!(last_price, thresholds[1].desired_price, "Price should be \'{}\' not \'{}\' for {}.", last_price, thresholds[1].desired_price, thresholds[1].title);
        }
        Err(_) => assert!(false, "Could not load thresholds when desired price was updated..")
    }
    file_operations::teardown();
}

#[test]
fn update_id(){
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    threshold_stubs::add_simple_threshold(&game_title, &game_alias, price);

    // Check that new store ids are successfully updated
    let new_steam_id = 333;
    let new_gog_id = 456;
    thresholds::update_id(&game_title, STEAM_STORE_ID, new_steam_id);
    thresholds::update_id(&game_title, GOG_STORE_ID, new_gog_id);
    match thresholds::load_thresholds(){
        Ok(thresholds) => {
            assert_eq!(new_steam_id, thresholds[0].steam_id, "Steam ID should be \'{}\' not \'{}\'.", new_steam_id, thresholds[0].steam_id);
            assert_eq!(new_gog_id, thresholds[0].gog_id, "GOG ID should be \'{}\' not \'{}\'.", new_gog_id, thresholds[0].gog_id);
        },
        Err(_) => assert!(false, "Could not load thresholds when store IDs (integer) where updated.")
    }
    file_operations::teardown();
}

#[test]
fn update_id_str(){
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    threshold_stubs::add_simple_threshold(&game_title, &game_alias, price);

    // Check that new store ids are successfully updated
    let new_ms_id = "cba";
    thresholds::update_id_str(&game_title, MICROSOFT_STORE_ID, new_ms_id);
    match thresholds::load_thresholds(){
        Ok(thresholds) => {
            assert_eq!(new_ms_id, thresholds[0].microsoft_store_id, "Microsoft Store ID should be \'{}\' not \'{}\'.", new_ms_id, thresholds[0].microsoft_store_id);
        },
        Err(_) => assert!(false, "Could not load thresholds when store IDs (string) where updated.")
    }
    file_operations::teardown();
}

#[test]
fn remove_game(){
    delete_thresholds();
    let first_game = String::from("Random Game");
    let second_game = String::from("Random Game 2");
    let third_game = String::from("Random Game 3");
    let game_alias = String::from("rg");
    let game_alias_2 = String::from("rg2");
    let price = 10.0;
    threshold_stubs::add_simple_threshold(&first_game, &game_alias, price);

    // Check that threshold is properly added
    match thresholds::load_thresholds(){
        Ok(thresholds) => {
            assert_eq!(1, thresholds.len(), "Thresholds length before deletion should be 1");
            assert_eq!(first_game, thresholds[0].title, "Game title should {} not {}", first_game, thresholds[0].title);
        },
        Err(e) => assert!(false, "Could not load thresholds before deletion.\n{}",e)
    }
    match thresholds::load_alias_map() {
        Ok(aliases) => assert_eq!(1, aliases.len(), "There should be {} alias(es) not {}", 1, aliases.len()),
        Err(_) => assert!(false, "Could not load alias map after deletion.")
    }

    // Delete test threshold
    thresholds::remove(&first_game);
    match thresholds::load_thresholds(){
        Ok(thresholds) => assert_eq!(0, thresholds.len(), "Thresholds length after deletion should be 0"),
        Err(_) => assert!(false, "Could not load thresholds after deletion.")
    }
    match thresholds::load_alias_map() {
        Ok(aliases) => assert_eq!(0, aliases.len(), "There should be no aliases present in the alias map"),
        Err(_) => assert!(false, "Could not load alias map after deletion.")
    }

    //Delete multiple thresholds via alias
    threshold_stubs::add_simple_threshold(&second_game, &game_alias_2, price);
    threshold_stubs::add_simple_threshold(&third_game, &game_alias_2, price);
    match thresholds::load_thresholds(){
        Ok(thresholds) => {
            assert_eq!(2, thresholds.len(), "Thresholds length before deletion should be 1");
            assert_eq!(second_game, thresholds[0].title, "Game title should {} not {}", second_game, thresholds[0].title);
            assert_eq!(third_game, thresholds[1].title, "Game title should {} not {}", third_game, thresholds[1].title);
        },
        Err(e) => assert!(false, "Could not load thresholds before deletion.\n{}",e)
    }
    match thresholds::load_alias_map() {
        Ok(aliases) => assert_eq!(1, aliases.len(), "There should be {} alias(es) not {}", 1, aliases.len()),
        Err(_) => assert!(false, "Could not load alias map after deletion.")
    }

    thresholds::remove(&game_alias_2);
    match thresholds::load_thresholds(){
        Ok(thresholds) => assert_eq!(0, thresholds.len(), "Thresholds length after deletion should be 0"),
        Err(_) => assert!(false, "Could not load thresholds after deletion.")
    }
    match thresholds::load_alias_map() {
        Ok(aliases) => assert_eq!(0, aliases.len(), "There should be no aliases present in the alias map"),
        Err(_) => assert!(false, "Could not load alias map after deletion.")
    }
    file_operations::teardown();
}