use crate::utils::pricing_utils::check_prices_for_email;
use alerting::email;
use properties;
use reqwest::Client;
use stores::pc::steam;
use stores::search::provider::SearchProvider;

pub async fn send_sales_email(search_provider: SearchProvider) -> Result<String, String> {
    email::params_check().map_err(|err| err.to_string())?;
    let use_html = true;
    match check_prices_for_email(use_html, search_provider).await {
        Ok(sales_str) => {
            let html_body = email::create_html_body(&sales_str).to_string();
            println!("Email Contents:\n{}", html_body);
            if sales_str.is_empty() {
                println!("No game(s) on sale at price thresholds");
            } else {
                println!("Sending email...");
                let to_address = &properties::get_recipient();
                return match email::send_html_msg(
                    to_address,
                    "Check Out Which Games Are On Sale",
                    &html_body,
                ) {
                    Ok(_) => Ok("Email sent (or send attempt completed).".into()),
                    Err(e) => Err(format!("{}", e)),
                };
            }
            Ok(String::from("Email sent (or send attempt completed)."))
        }
        Err(err) => Err(format!("Cound not send an email due to:\n{}", err)),
    }
}

pub async fn update_cache(http_client: Client) -> Result<String, String> {
    match steam::update_cached_games(&http_client).await {
        Ok(results) => Ok(results),
        Err(err) => Err(err.to_string()),
    }
}
