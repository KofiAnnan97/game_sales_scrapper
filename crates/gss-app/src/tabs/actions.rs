use iced::widget::{Button, Scrollable, column, row, text};
use iced::{Element,Length};

use files::general;

use crate::components::custom_widgets as cw;
use crate::{LOADING_FRAMES_SIZE, MainMessage, Message, STATUS_ERR};

pub enum ActionDisplayed{
    NoAction,
    UpdateCache,
    Logs,
}

pub fn view_actions(app: &crate::App) -> Element<'_, Message> {
    let cache_loading = cw::text_loading_indicator("Retrieve games to cache", app.caching_loading_frame, LOADING_FRAMES_SIZE);

    let complete_logs = general::get_contents(&app.current_log_file) + &app.log_batch;
    let logs_display = Scrollable::new(text(complete_logs))
        .width(Length::Fill)
        .height(Length::Fill);

    let caching_display = Scrollable::new(
        if app.is_caching_in_progress {
            cache_loading
        } else if !app.message_details.is_empty() && app.status_message == STATUS_ERR{
            text("Cache update failed.")
        } else {
            text(&app.status_message)
        }
    );

    let display_results = match app.action_displayed {
        ActionDisplayed::Logs => logs_display,
        ActionDisplayed::UpdateCache => caching_display,
        ActionDisplayed::NoAction => Scrollable::new(text(""))
    };

    column![
        row![
            Button::new(text("Update Game Cache"))
                .on_press(MainMessage::UpdateCache.into())
                .padding(10),
            Button::new(text("Show Logs"))
                .on_press(MainMessage::LogsShown.into())
                .padding(10)
        ]
        .spacing(10),
        display_results,
    ]
    .spacing(15)
    .padding(10)
    .into()
}