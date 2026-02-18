#[cfg(test)]
use stores::pc::microsoft_store;

// Constants
static GAME_TITLE: &str = "SILENT HILL f";
static GAME_ID: &str = "9n5nfrqv2hqq";

#[tokio::test]
async fn search_game() {
    let client = reqwest::Client::new();
    let search_list =  microsoft_store::search_game_by_title(GAME_TITLE, &client)
        .await.unwrap_or_else(|_| Vec::new());
    let mut is_game_present = false;
    for product in search_list {
        if product.title == GAME_TITLE {
            is_game_present = true;
            break;
        }
    }
    assert!(is_game_present, "Could not find game: {}", GAME_TITLE);
}

#[tokio::test]
async fn get_price_info() {
    let client = reqwest::Client::new();
    match microsoft_store::get_price_details(GAME_ID, &client).await {
        Some(info) => {
            assert_eq!(info.title, GAME_TITLE, "{} != {}", info.title, GAME_TITLE);
            assert_ne!("", info.original_price, "Original price field is empty");
            assert_ne!("", info.current_price, "Current price field is empty");
            assert_ne!("", info.discount_percentage, "Discount % field is empty");
            assert_ne!("", info.icon_link, "Icon link field is empty");
            //assert_ne!("", info.store_page_link, "Store page link field is empty");
        }
        None =>  assert!(false, "Game with id {} does not exist", GAME_ID),
    }
}