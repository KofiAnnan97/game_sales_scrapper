static GAME_TITLE: &str = "Half-Life 2";
static GAME_ID: usize = 220;

#[cfg(test)]
mod steam_api_tests {
    use crate::steam;

    // Constants
    use crate::steam_api_test::{GAME_ID, GAME_TITLE};

    #[tokio::test]
    async fn search_game() {
        let search_list = steam::search_by_keyphrase(GAME_TITLE)
            .await.unwrap_or_else(|e| Vec::new());
        let mut is_game_present = false;
        for title in search_list {
            if title == GAME_TITLE {
                is_game_present = true;
                break;
            }
        }
        assert!(is_game_present, "Could not find game: {}", GAME_TITLE);
    }

    #[tokio::test]
    async fn get_price_info() {
        let client = reqwest::Client::new();
        match steam::get_price_details(GAME_ID, &client).await {
            Ok(info) => {
                assert_eq!(info.title, GAME_TITLE, "{} != {}", info.title, GAME_TITLE);
                assert_ne!("", info.original_price, "Original price field is empty");
                assert_ne!("", info.current_price, "Current price field is empty");
                assert_ne!("", info.discount_percentage, "Discount % field is empty");
                assert_ne!("", info.icon_link, "Icon link field is empty");
                assert_ne!("", info.store_page_link, "Store page link field is empty");
            }
            Err(e) =>  assert!(false, "Game with id {} does not exist\nError: {}", GAME_ID, e),
        }
    }
}