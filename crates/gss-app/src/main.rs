use iced::daemon;
use iced_aw::ICED_AW_FONT_BYTES;

// Common internal libraries
use files::general;

// App specific
use gss_app::app::constants::FALLBACK_LOG;
use gss_app::app::state::App;
use gss_app::utils::log_utils;

fn main() -> iced::Result {
    let log_file = log_utils::new_log().unwrap_or_else(|e| {
        eprintln!("Could not create the application log file: {}", e);
        match general::write_to_file(FALLBACK_LOG.to_string(), String::new()) {
            Ok(_) => FALLBACK_LOG.to_string(),
            Err(fe) => {
                eprintln!(
                    "Could not create fallback log file '{}': {}",
                    FALLBACK_LOG, fe
                );
                FALLBACK_LOG.to_string()
            }
        }
    });
    let log_file_clone = log_file.clone();
    std::panic::set_hook(Box::new(move |panic_info| {
        let panic_msg = log_utils::fatal_message_builder(panic_info);
        general::append_to_file(&log_file_clone, &panic_msg)
            .expect("Could not append panic message to log file.");
    }));

    daemon(move || App::new(log_file.clone()), App::update, App::view)
        .title(|app: &App, window_id| {
            if Some(window_id) == app.manual_prune_window {
                "Prune Logs".to_string()
            } else {
                "Game Sales Scrapper".to_string()
            }
        })
        .subscription(App::subscription)
        .theme(|app: &App, _status| app.theme())
        .font(ICED_AW_FONT_BYTES)
        .run()
}
