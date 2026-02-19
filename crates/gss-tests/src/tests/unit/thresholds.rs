use file_ops::{settings, thresholds};
use file_types::common;
use properties;
use constants::operations::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};
use constants::operations::thresholds::THRESHOLD_FILENAME;
use file_ops::thresholds::update_thresholds;
use structs::internal::data::GameThreshold;
use structs::response::steam::App;
use structs::response::gog::{BaseMoney, FinalMoney, GameInfo as GOGGame, GameInfoBuilder as GOGGameBuilder, Price};
use structs::response::microsoft_store::{PriceInfo, ProductInfo as MSGame, ProductInfoBuilder as MSGameBuilder};
use crate::tests::helper;

fn delete_thresholds() {
    if !properties::is_testing_enabled() { properties::set_test_mode(true); }
    settings::update_alias_reuse_state(1);
    let mut config_path = properties::get_data_path();
    config_path.push_str("/");
    config_path.push_str(THRESHOLD_FILENAME);
    common::delete_file(config_path);
}

fn add_simple_threshold(game_title: &str, game_alias: &str, price: f64) {
    let mut thresholds = thresholds::load_thresholds().unwrap_or_default();
    let mut unique_title = true;
    for threshold in &thresholds{
        if threshold.title == game_title {
            unique_title = false;
            break;
        }
    }
    if unique_title {
        thresholds.push(GameThreshold {
            title: game_title.to_string(),
            alias: game_alias.to_string(),
            steam_id: 123,
            gog_id: 456,
            microsoft_store_id: String::from("abc"),
            currency: String::from("USD"),
            desired_price: price
        });
    }
    update_thresholds(thresholds);
}

fn test_steam_app() -> App{
    App{
        app_id: 220,
        name: "Half-Life 2".to_string(),
        last_modified: 678910,
        price_change_number: 1112131415,
    }
}

fn test_gog_game() -> GOGGame {
    let id_str = String::from("123");
    let title = String::from("Random GOG Game");
    let price = Price {
        final_price: String::new(),
        base_price: String::new(),
        discount: None,
        final_money: FinalMoney {
            amount: String::new(),
            currency: "USD".to_string(),
            discount: String::new(),
        },
        base_money: BaseMoney {
            amount: String::new(),
            currency: String::new(),
        }
    };
    let icon_link = String::new();
    let store_page_link = String::new();
    GOGGameBuilder::new(id_str, title, price, icon_link, store_page_link)
}

fn test_ms_game() -> MSGame {
    let id_str = String::from("abc");
    let title = String::from("Random Microsoft Game");
    let price = PriceInfo {
        msrp: None,
        price: None,
        badge_text: None,
        force_to_display_price: false,
        narrator_text: "".to_string(),
        ownership: 0,
    };
    let icon_link = String::new();
    let store_page_link = String::new();
    MSGameBuilder::new(id_str, title, price, icon_link, store_page_link)
}

#[tokio::test]
async fn add_steam_game() {
    delete_thresholds();
    let client = reqwest::Client::new();
    let app = test_steam_app();
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
    helper::teardown();
}

#[test]
fn add_gog_game() {
    delete_thresholds();
    let game = test_gog_game();
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
    helper::teardown();
}

#[test]
fn add_microsoft_store_game() {
    delete_thresholds();
    let game = test_ms_game();
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
    helper::teardown();
}

#[test]
fn update_alias() {
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    add_simple_threshold(&game_title, &game_alias, price);

    // Check that alias is empty
    match thresholds::load_thresholds(){
        Ok(thresholds) =>
            assert_eq!(game_alias, thresholds[0].alias, "Alias should be \'{}\' not \'{}\'.", "", thresholds[0].alias),
        Err(_) => assert!(false, "Could not load the thresholds when alias is expected to be empty.")
    }

    // Check that new alias is present in threshold
    let new_alias = String::from("new_rg");
    thresholds::update_alias(&game_title, &new_alias);
    match thresholds::load_thresholds(){
        Ok(thresholds) =>
            assert_eq!(new_alias, thresholds[0].alias, "Alias should be \'{}\' not \'{}\'.", new_alias, thresholds[0].alias),
        Err(_) => assert!(false, "Could not load the thresholds when alias is expected to be {}.", new_alias)
    }
    helper::teardown();
}

#[test]
fn update_price() {
    delete_thresholds();
    let first_game = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    add_simple_threshold(&first_game, &game_alias, price);

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
    add_simple_threshold(&second_game, &game_alias, new_price);
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
    helper::teardown();
}

#[test]
fn update_id(){
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    add_simple_threshold(&game_title, &game_alias, price);

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
    helper::teardown();
}

#[test]
fn update_id_str(){
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    add_simple_threshold(&game_title, &game_alias, price);

    // Check that new store ids are successfully updated
    let new_ms_id = "cba";
    thresholds::update_id_str(&game_title, MICROSOFT_STORE_ID, new_ms_id);
    match thresholds::load_thresholds(){
        Ok(thresholds) => {
            assert_eq!(new_ms_id, thresholds[0].microsoft_store_id, "Microsoft Store ID should be \'{}\' not \'{}\'.", new_ms_id, thresholds[0].microsoft_store_id);
        },
        Err(_) => assert!(false, "Could not load thresholds when store IDs (string) where updated.")
    }
    helper::teardown();
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
    add_simple_threshold(&first_game, &game_alias, price);

    // Check that threshold is properly added
    match thresholds::load_thresholds(){
        Ok(thresholds) => {
            assert_eq!(1, thresholds.len(), "Thresholds length before deletion should be 1");
            assert_eq!(first_game, thresholds[0].title, "Game title should {} not {}", first_game, thresholds[0].title);
        },
        Err(e) => assert!(false, "Could not load thresholds before deletion.\n{}",e)
    }

    // Delete test threshold
    thresholds::remove(&first_game);
    match thresholds::load_thresholds(){
        Ok(thresholds) => assert_eq!(0, thresholds.len(), "Thresholds length after deletion should be 0"),
        Err(_) => assert!(false, "Could not load thresholds after deletion.")
    }

    //Delete multiple thresholds via alias
    add_simple_threshold(&second_game, &game_alias_2, price);
    add_simple_threshold(&third_game, &game_alias_2, price);
    match thresholds::load_thresholds(){
        Ok(thresholds) => {
            assert_eq!(2, thresholds.len(), "Thresholds length before deletion should be 1");
            assert_eq!(second_game, thresholds[0].title, "Game title should {} not {}", second_game, thresholds[0].title);
            assert_eq!(third_game, thresholds[1].title, "Game title should {} not {}", third_game, thresholds[1].title);
        },
        Err(e) => assert!(false, "Could not load thresholds before deletion.\n{}",e)
    }
    thresholds::remove(&game_alias_2);
    match thresholds::load_thresholds(){
        Ok(thresholds) => assert_eq!(0, thresholds.len(), "Thresholds length after deletion should be 0"),
        Err(_) => assert!(false, "Could not load thresholds after deletion.")
    }
    helper::teardown();
}