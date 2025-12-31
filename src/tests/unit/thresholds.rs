#[cfg(test)]
use serde_json::json;
use file_ops::thresholds;
use file_types::json as json_data;
use file_ops::settings::{GOG_STORE_ID, MICROSOFT_STORE_ID, STEAM_STORE_ID};
use structs::data::GameThreshold;
use structs::steam::App;
use structs::gog::{GameInfoBuilder as GOGGameBuilder, GameInfo as GOGGame, Price, FinalMoney, BaseMoney};
use structs::microsoft_store::{ProductInfoBuilder as MSGameBuilder, ProductInfo as MSGame, PriceInfo};

// Constants
static THRESHOLD_FILENAME: &str = "thresholds.json";

fn delete_thresholds() {
    let mut config_path = json_data::get_data_path();
    config_path.push_str("/");
    config_path.push_str(THRESHOLD_FILENAME);
    json_data::delete_file(config_path);
}

fn add_simple_threshold(game_title: &str, game_alias: &str, price: f64) {
    let data = json!([GameThreshold {
        title: game_title.to_string(),
        alias: game_alias.to_string(),
        steam_id: 123,
        gog_id: 456,
        microsoft_store_id: String::from("abc"),
        currency: String::from("USD"),
        desired_price: price
    }]);
    let filepath = thresholds::get_path();
    match serde_json::to_string(&data){
        Ok(data) => json_data::write_to_file(filepath, data),
        Err(_) => ()
    }
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

    match thresholds::load_data() {
        Ok(thresholds) => {
            assert_eq!(game_title.clone(), thresholds[0].title, "Expected {} not {}", game_title.clone(), thresholds[0].title);
            assert_eq!(game_id, thresholds[0].steam_id, "Expected {} not {}", game_id, thresholds[0].steam_id);
        },
        Err(_) => assert!(false, "Could not find game: {} ({})", game_title.clone(), game_id),
    }
}

#[test]
fn add_gog_game() {
    delete_thresholds();
    let game = test_gog_game();
    let game_title = &game.title.clone();
    let game_id = game.id.parse::<usize>().unwrap();
    thresholds::add_gog_game(game_title.clone(), &game, 10.00);

    match thresholds::load_data() {
        Ok(thresholds) => {
            assert_eq!(game_title.clone(), thresholds[0].title, "Expected {} not {}", game_title.clone(), thresholds[0].title);
            assert_eq!(game_id, thresholds[0].gog_id, "Expected {} not {}", game_id, thresholds[0].gog_id);
        },
        Err(_) => assert!(false, "Could not find game: {} ({})", game_title.clone(), game_id),
    }
}

#[test]
fn add_microsoft_store_game() {
    delete_thresholds();
    let game = test_ms_game();
    let game_title = &game.title.clone();
    let game_id = &game.product_id.clone();
    thresholds::add_microsoft_store_game(game_title.clone(), &game, 10.00);

    match thresholds::load_data() {
        Ok(thresholds) => {
            assert_eq!(game_title.clone(), thresholds[0].title, "Expected {} not {}", game_title.clone(), thresholds[0].title);
            assert_eq!(*game_id, thresholds[0].microsoft_store_id, "Expected {} not {}", game_id, thresholds[0].microsoft_store_id);
        },
        Err(_) => assert!(false, "Could not find game: {} ({})", game_title.clone(), game_id),
    }
}

/*#[test]
fn set_game_alias_sequence() {}*/

#[test]
fn update_alias() {
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    add_simple_threshold(&game_title, &game_alias, price);

    // Check that alias is empty
    match thresholds::load_data(){
        Ok(thresholds) =>
            assert_eq!(game_alias, thresholds[0].alias, "Alias should be \'{}\' not \'{}\'.", "", thresholds[0].alias),
        Err(_) => assert!(false, "Could not load the thresholds when alias is expected to be empty.")
    }

    // Check that new alias is present in threshold
    let new_alias = String::from("new_rg");
    thresholds::update_alias(&game_title, &new_alias);
    match thresholds::load_data(){
        Ok(thresholds) =>
            assert_eq!(new_alias, thresholds[0].alias, "Alias should be \'{}\' not \'{}\'.", new_alias, thresholds[0].alias),
        Err(_) => assert!(false, "Could not load the thresholds when alias is expected to be {}.", new_alias)
    }
}

#[test]
fn update_price() {
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    add_simple_threshold(&game_title, &game_alias, price);

    // Check that new price is present in threshold
    let new_price = 20.00;
    thresholds::update_price(&game_title, new_price);
    match thresholds::load_data(){
        Ok(thresholds) =>
            assert_eq!(new_price, thresholds[0].desired_price, "Price should be \'{}\' not \'{}\'.", new_price, thresholds[0].desired_price),
        Err(_) => assert!(false, "Could not load thresholds when desired price was updated..")
    }
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
    match thresholds::load_data(){
        Ok(thresholds) => {
            assert_eq!(new_steam_id, thresholds[0].steam_id, "Steam ID should be \'{}\' not \'{}\'.", new_steam_id, thresholds[0].steam_id);
            assert_eq!(new_gog_id, thresholds[0].gog_id, "GOG ID should be \'{}\' not \'{}\'.", new_gog_id, thresholds[0].gog_id);
        },
        Err(_) => assert!(false, "Could not load thresholds when store IDs (integer) where updated.")
    }
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
    match thresholds::load_data(){
        Ok(thresholds) => {
            assert_eq!(new_ms_id, thresholds[0].microsoft_store_id, "Microsoft Store ID should be \'{}\' not \'{}\'.", new_ms_id, thresholds[0].microsoft_store_id);
        },
        Err(_) => assert!(false, "Could not load thresholds when store IDs (string) where updated.")
    }
}

#[test]
fn remove_game(){
    delete_thresholds();
    let game_title = String::from("Random Game");
    let game_alias = String::from("rg");
    let price = 10.0;
    add_simple_threshold(&game_title, &game_alias, price);

    // Check that threshold is properly added
    match thresholds::load_data(){
        Ok(thresholds) => {
            assert_eq!(1, thresholds.len(), "Thresholds length before deletion should be 1");
            assert_eq!(game_title, thresholds[0].title, "Game title should {} not {}", game_title, thresholds[0].title);
        },
        Err(e) => assert!(false, "Could not load thresholds before deletion.\n{}",e)
    }

    // Delete test threshold
    thresholds::remove(&game_title);
    match thresholds::load_data(){
        Ok(thresholds) => assert_eq!(0, thresholds.len(), "Thresholds length after deletion should be 0"),
        Err(_) => assert!(false, "Could not load thresholds after deletion.")
    }
}