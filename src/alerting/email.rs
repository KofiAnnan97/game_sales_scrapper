use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use dotenv::dotenv;

use crate::structs::data::SaleInfo;

pub fn send_plain_text_msg(recipient: &str, subject: &str, body: &str) {
    dotenv().ok();
    let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let smtp_port : u16 = std::env::var("SMTP_PORT").expect("SMTP_PORT must be set")
                                                    .parse().expect("Not a valid u16");
    let smtp_email = &std::env::var("SMTP_EMAIL").expect("SMTP_EMAIL must be set");
    let smtp_user = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_pwd = std::env::var("SMTP_PWD").expect("SMTP_PWD must be set");

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

pub fn get_stylesheet() -> String {
    return String::from(r#"
        <style>
            img {
                max-width: auto;
                max-height: 100px;
                display: block;
                margin: auto;
            }
            table {
                border-collapse: collapse;
                width: 100%;
            }
            th, td {
                text-align: right;
                padding: 8px;
                border-bottom: 1px solid #01250a;
            }
            tr {
                color:rgb(255, 255, 255);
                background-color: #abceac;
            }
            tr:hover {background-color: #499862;}
            body{
                background-color: rgb(225, 215, 215);
                padding: 25px;
            }
            .title {
                color: #5C6AC4;
            }
            .storefront{
                color: #053e00;
            }
        </style>
    "#);
}

pub fn create_storefront_table_html(store_name: &str, sales: Vec<SaleInfo>) -> String{
    let mut rows = String::new(); 
    for s_info in sales{
        rows += &format!("<tr>
                <td>
                    <a href=\"{store_page}\">
                        <img src=\"{icon}\" alt=\"{title}\">
                    </a>
                </td>
                <td style=\"text-align: left;\">
                    <a href=\"{store_page}\">{title}</a>
                </td>
                <td><del>${old_price}</del> ${new_price}</td>
                <td style=\"text-align: center;\">({price_off}% off)</td>
            </tr>", 
        icon=s_info.icon_link, title=s_info.title,
        old_price=s_info.original_price, new_price=s_info.current_price, 
        price_off=s_info.discount_percentage, store_page=s_info.store_page_link);
    }
    let data = format!(r#"
        <h2 class="storefront">{}</h2>
        <table>
            {}
        </table>
    "#, store_name, rows);
    return data;
}

pub fn create_html_body(sales_info_html: &str) -> String{
    let stylesheet = get_stylesheet();
    return format!(r#"
        <!DOCTYPE html>
        <html>
        <head>
            {}
            <title>Game Sales Scrapper</title>
        </head>
        <body>
            <p>
                One or more games is less than or equal to the respective price threshold you set. 
                A game may appear multiple times if the product is on sale on multiple storefronts.
            </p>
            {}
        </body>
        </html>
    "#, stylesheet, sales_info_html);
}

pub fn send_html_msg(recipient: &str, subject: &str, body: &str) {
    dotenv().ok();
    let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let smtp_port : u16 = std::env::var("SMTP_PORT").expect("SMTP_PORT must be set")
                                                    .parse().expect("Not a valid u16");
    let smtp_email = &std::env::var("SMTP_EMAIL").expect("SMTP_EMAIL must be set");
    let smtp_user = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_pwd = std::env::var("SMTP_PWD").expect("SMTP_PWD must be set");

    let html_content = format!(r#"{}"#, create_html_body(body));
    let email = Message::builder()
        .from(smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative().singlepart(SinglePart::html(html_content.to_string())),
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