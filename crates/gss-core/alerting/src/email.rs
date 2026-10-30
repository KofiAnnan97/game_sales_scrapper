use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use std::net::IpAddr;
use url::Url;

use constants::alerting::email::{
    ALLOWED_IMAGE_DOMAINS, ALLOWED_STORE_DOMAINS, EMAIL_STYLESHEET, HTML_BODY_HEADER,
};
use constants::operations::properties::{
    PROP_RECIPIENT_EMAIL, PROP_SMTP_EMAIL, PROP_SMTP_HOST, PROP_SMTP_PASSWORD, PROP_SMTP_PORT,
    PROP_SMTP_USERNAME,
};
use properties;
use types::internal::data::SaleInfo;
use types::internal::store::GameStore;

pub fn params_check() {
    // Get email parameters
    let recipient = properties::get_recipient();
    let smtp_host = properties::get_smtp_host();
    let smtp_port: u16 = properties::get_smtp_port();
    let smtp_email = properties::get_smtp_email();
    let smtp_user = properties::get_smtp_user();
    let smtp_pwd = properties::get_smtp_pwd(false);

    // Create error message
    let mut err_msg = String::new();
    if recipient.is_empty() {
        err_msg.push_str(&format!("  - {}\n", PROP_RECIPIENT_EMAIL));
    }
    if smtp_host.is_empty() {
        err_msg.push_str(&format!("  - {}\n", PROP_SMTP_HOST));
    }
    if smtp_port == 0 {
        err_msg.push_str(&format!("  - {} (cannot be 0)\n", PROP_SMTP_PORT));
    }
    if smtp_email.is_empty() {
        err_msg.push_str(&format!("  - {}\n", PROP_SMTP_EMAIL));
    }
    if smtp_user.is_empty() {
        err_msg.push_str(&format!("  - {}\n", PROP_SMTP_USERNAME));
    }
    if smtp_pwd.is_empty() {
        err_msg.push_str(&format!("  - {}\n", PROP_SMTP_PASSWORD));
    }
    if !err_msg.is_empty() {
        panic!(
            "Cannot send email without the following properties:\n{}",
            err_msg
        );
    }
}

pub fn send_plain_text_msg(recipient: &str, subject: &str, body: &str) {
    let smtp_host = properties::get_smtp_host();
    let smtp_port: u16 = properties::get_smtp_port();
    let smtp_email = properties::get_smtp_email();
    let smtp_user = properties::get_smtp_user();
    let smtp_pwd = properties::get_smtp_pwd(false);

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
    let icon_link = validate_url(&info.icon_link, ALLOWED_IMAGE_DOMAINS)
        .map(escape_html_char)
        .unwrap_or_default();
    let game_title = escape_html_char(&info.title);
    let old_price = escape_html_char(&info.original_price.to_string());
    let new_price = escape_html_char(&info.current_price.to_string());
    let discount = escape_html_char(&info.discount_percentage.to_string());
    let store_page_link = validate_url(&info.store_page_link, ALLOWED_STORE_DOMAINS)
        .map(escape_html_char)
        .unwrap_or_default();

    let game_card = format!(
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
    View on {6} &#8594;
    </a>

    </div>

    </div>
    "#,
        icon_link, game_title, old_price, new_price, discount, store_page_link, store_name
    );
    game_card
}

pub fn create_store_cards(store_name: GameStore, sales: Vec<SaleInfo>) -> String {
    let simple_name = store_name.get_name();

    let mut store_cards = format!(
        r#"
    <!-- {0} -->
    <div class="store">
    <h2 class="storefront">{0}</h2>
    "#,
        store_name
    );
    for game in sales {
        store_cards.push_str(&create_game_card(game, simple_name));
    }
    store_cards.push_str("</div>");
    store_cards
}

pub fn create_html_body(sales_info_html: &str) -> String {
    let html_body = format!(
        r#"
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
    "#,
        EMAIL_STYLESHEET, HTML_BODY_HEADER, sales_info_html
    );
    html_body
}

pub fn send_html_msg(recipient: &str, subject: &str, body: &str) -> Result<String, String> {
    let smtp_host = properties::get_smtp_host();
    let smtp_port: u16 = properties::get_smtp_port();
    let smtp_email = properties::get_smtp_email();
    let smtp_user = properties::get_smtp_user();
    let smtp_pwd = properties::get_smtp_pwd(false);

    let email = Message::builder()
        .from(smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(subject)
        .multipart(MultiPart::alternative().singlepart(SinglePart::html(body.to_string())))
        .unwrap();

    let creds = Credentials::new(smtp_user, smtp_pwd);

    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .unwrap()
        .credentials(creds)
        .port(smtp_port)
        .authentication(vec![Mechanism::Login])
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok("Email send attempt completed successfully".into()),
        Err(e) => Err(format!("Failed to send email - {}", e)),
    }
}

fn escape_html_char(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

pub fn validate_url<'a>(input: &'a str, allowed_domains: &[&str]) -> Option<&'a str> {
    if input.is_empty()
        || input
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return None;
    }

    match Url::parse(input) {
        Ok(url) => {
            // Reject non HTTPs schemes
            if url.scheme() != "https" && url.scheme() != "http" {
                return None;
            }

            // Reject user credentials
            if !url.username().is_empty() || url.password().is_some() {
                return None;
            }

            // Reject all ports in url
            if url.port().is_some() {
                return None;
            }

            if let Some(url_host) = url.host_str()
                && url_host.parse::<IpAddr>().is_err()
                && allowed_domains.contains(&url_host)
            {
                Some(input)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
