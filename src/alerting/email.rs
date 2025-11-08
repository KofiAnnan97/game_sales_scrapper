use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use dotenv::dotenv;

pub fn send(recipient: &str, subject: &str, body: &str) {
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