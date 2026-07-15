use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};

use structs::internal::data::SaleInfo;
use properties;
use constants::operations::settings::{STEAM_STORE_NAME, GOG_STORE_NAME, MICROSOFT_STORE_NAME}; 
use constants::operations::properties::{PROP_RECIPIENT_EMAIL, PROP_SMTP_HOST, PROP_SMTP_PORT, 
                                        PROP_SMTP_EMAIL, PROP_SMTP_USERNAME, PROP_SMTP_PASSWORD};
use constants::alerting::email::{EMAIL_STYLESHEET, HTML_BODY_HEADER};

pub fn params_check(){
    // Get email parameters
    let recipient= properties::get_recipient();
    let smtp_host = properties::get_smtp_host();
    let smtp_port : u16 = properties::get_smtp_port(); 
    let smtp_email = properties::get_smtp_email();
    let smtp_user = properties::get_smtp_user();
    let smtp_pwd = properties::get_smtp_pwd();

    // Create error message
    let mut err_msg = String::new();
    if recipient.is_empty() { err_msg.push_str(&format!("  - {}\n", PROP_RECIPIENT_EMAIL)); }
    if smtp_host.is_empty() { err_msg.push_str(&format!("  - {}\n", PROP_SMTP_HOST)); }
    if smtp_port == 0 { err_msg.push_str(&format!("  - {} (cannot be 0)\n", PROP_SMTP_PORT)); }
    if smtp_email.is_empty() { err_msg.push_str(&format!("  - {}\n", PROP_SMTP_EMAIL)); }
    if smtp_user.is_empty() { err_msg.push_str(&format!("  - {}\n", PROP_SMTP_USERNAME)); }
    if smtp_pwd.is_empty() { err_msg.push_str(&format!("  - {}\n", PROP_SMTP_PASSWORD)); }
    if !err_msg.is_empty() {
        panic!("Cannot send email without the following properties:\n{}", err_msg);
    }
}

pub fn send_plain_text_msg(recipient: &str, subject: &str, body: &str) {
    let smtp_host = properties::get_smtp_host();
    let smtp_port : u16 = properties::get_smtp_port(); 
    let smtp_email = properties::get_smtp_email();
    let smtp_user = properties::get_smtp_user();
    let smtp_pwd = properties::get_smtp_pwd();

    let email = Message::builder()
        .from(smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(subject)
        .body(body.to_string())
        .unwrap();

    let creds = Credentials::new(smtp_user, smtp_pwd);

    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .unwrap()  
        .credentials(creds)
        .port(smtp_port)  
        .authentication(vec![Mechanism::Login])
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully"),
        Err(e) => eprintln!("Failed to send email: {e}"),
    }
}

pub fn create_game_card(info: SaleInfo, store_name: &str) -> String { 
    let icon_link = info.icon_link;
    let game_title = info.title;
    let old_price = info.original_price;
    let new_price = info.current_price;
    let discount = info.discount_percentage;
    let store_page_link = info.store_page_link;

    format!(
    r#"
    <div class="game-card">

    <a href="{5}">
    <img src="{0}"
    alt="{1}">
    </a>

    <div class="game-info">

    <a class="game-title"
    href="{5}">
    {1}
    </a>

    <div class="price-row">
    <span class="old-price">${2}</span>
    <span class="new-price">${3}</span>
    <span class="discount">{4}% OFF</span>
    </div>

    <a class="store-link"
    href="{5}">
    View on {6} →
    </a>

    </div>

    </div>
    "#,
    icon_link, game_title, old_price, new_price, discount, store_page_link, store_name)
}

pub fn create_store_cards(store_name: &str, sales: Vec<SaleInfo>) -> String {
    let simple_name = match store_name {
        STEAM_STORE_NAME => "Steam",
        GOG_STORE_NAME  => "GOG",
        MICROSOFT_STORE_NAME => "Microsoft Store",
        _ => ""
    };
    
    let mut store_cards = format!(r#"
    <!-- {0} -->
    <div class="store">
    <h2 class="storefront">{0}</h2>
    "#, store_name);
    for game in sales {
        store_cards.push_str(&create_game_card(game, simple_name));
    }
    store_cards.push_str("</div>");
    store_cards
}

pub fn create_html_body(sales_info_html: &str) -> String {
    format!(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Game Sale Alerts</title>
    {}
    </head>
    
    <body>
    <div class="container">
    
    {}

    {}

    </div>
    </body>
    </html>
    "#, EMAIL_STYLESHEET, HTML_BODY_HEADER, sales_info_html)
}

pub fn send_html_msg(recipient: &str, subject: &str, body: &str) {
    let smtp_host = properties::get_smtp_host();
    let smtp_port : u16 = properties::get_smtp_port();
    let smtp_email = properties::get_smtp_email();
    let smtp_user = properties::get_smtp_user();
    let smtp_pwd = properties::get_smtp_pwd();
    
    let email = Message::builder()
        .from(smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative().singlepart(SinglePart::html(body.to_string())),
        )
        .unwrap();

    let creds = Credentials::new(smtp_user, smtp_pwd);

    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .unwrap()  
        .credentials(creds)
        .port(smtp_port)  
        .authentication(vec![Mechanism::Login])
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully"),
        Err(e) => eprintln!("Failed to send email: {e}"),
    }
}