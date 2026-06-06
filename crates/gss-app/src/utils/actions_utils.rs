use alerting::email;
use properties;
use stores::pc::{steam};

use crate::utils::pricing_utils::{check_prices};

pub async fn send_sales_email() -> Result<String, String> {
    let smtp_props = std::panic::catch_unwind(|| {email::params_check();});
    if let Err(err) = smtp_props {
        return Err(format!("{:?}", err));
    }
    let use_html = true;
    match check_prices(use_html).await{
        Ok(sales_str) => {
            let html_body = format!(r#"{}"#, email::create_html_body(&sales_str));
            println!("Email Contents:\n{}", html_body);
            if sales_str.is_empty(){ println!("No game(s) on sale at price thresholds"); }
            else {
                println!("Sending email...");
                let to_address = &properties::get_recipient();
                email::send_html_msg(to_address, "Check Out Which Games Are On Sale", &html_body);
            }
            Ok(String::from("Email sent (or send attempt completed)."))
        },
        Err(err) => return Err(format!("Cound not send an email due to:\n{}", err))
    }
}

pub async fn update_cache() -> Result<String, String> {
    steam::update_cached_games().await;
    Ok(String::from("Steam cache updated."))
}