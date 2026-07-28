use iced::widget::{column, row, container, text, TextInput, Button, Checkbox, Scrollable};
use iced::{Element, Length};

use crate::Message;

pub fn settings_window(app: &crate::App) -> Element<'_, Message> {
    let settings_content = column![
        if app.test_mode {
            column![
            text("Test path"),
            TextInput::new("Enter test path", &app.test_path)
                .on_input(Message::TestPathChanged)
                .padding(5)
                .width(Length::Fill),
            ].spacing(4)
        } else {
            column![
            text("Project path"),
            TextInput::new("Enter project path", &app.project_path)
                .on_input(Message::ProjectPathChanged)
                .padding(5)
                .width(Length::Fill),
            ]
            .spacing(4)
        },
        column![
            text("Steam API key"),
            if app.reveal_sensitive_data {
                TextInput::new("Enter Steam API key", &app.steam_api_key)
                    .on_input(Message::SteamApiKeyChanged)
                    .padding(5)
                    .width(Length::Fill)
            } else {
                TextInput::new("Enter Steam API key", &app.steam_api_key)
                    .padding(5)
                    .width(Length::Fill)
            }
        ]
        .spacing(4),
        column![
            text("Recipient email"),
            TextInput::new("Enter recipient email", &app.recipient_email)
                .on_input(Message::RecipientEmailChanged)
                .padding(5)
                .width(Length::Fill),
        ]
        .spacing(4),
        row![
            column![
                text("SMTP host"),
                TextInput::new("Enter SMTP host", &app.smtp_host)
                    .on_input(Message::SmtpHostChanged)
                    .padding(5)
                    .width(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill),
            column![
                text("SMTP port"),
                TextInput::new("Enter SMTP port", &app.smtp_port)
                    .on_input(Message::SmtpPortChanged)
                    .padding(5)
                    .width(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill),
        ]
        .spacing(8),
        row![
            column![
                text("SMTP email"),
                TextInput::new("Enter SMTP email", &app.smtp_email)
                    .on_input(Message::SmtpEmailChanged)
                    .padding(5)
                    .width(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill),
            column![
                text("SMTP user"),
                TextInput::new("Enter SMTP user", &app.smtp_user)
                    .on_input(Message::SmtpUserChanged)
                    .padding(5)
                    .width(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill),
        ]
        .spacing(8),
        column![
            text("SMTP password"),
            if app.reveal_sensitive_data {
                TextInput::new("Enter SMTP password", &app.smtp_password)
                .on_input(Message::SmtpPasswordChanged)
                .padding(5)
                .width(Length::Fill)
            } else {
                TextInput::new("Enter SMTP password", &app.smtp_password)
                .padding(5)
                .width(Length::Fill)
            },
        ]
        .spacing(4),
        
        row![
            Checkbox::new(app.test_mode)
                .label("Enable test mode")
                .on_toggle(Message::ToggleTestMode)
                .width(Length::Fixed(200.0))
                .spacing(10), 
            Checkbox::new(app.reveal_sensitive_data)
                .label("Reveal sensitive data")
                .on_toggle(Message::ToggleSensitiveData)
                .spacing(10)
        ].padding(10),
        Button::new(text("Save Settings"))
            .on_press(Message::SaveSettings)
            .padding(10),
        // container(text!("{:?}", app.status_message)),
    ]
    .spacing(12)
    .padding(10);

    let scrollable_settings = Scrollable::new(settings_content).height(Length::Fill);

    container(scrollable_settings)
        .padding(10)
        .height(Length::Fill)
        .into()
}