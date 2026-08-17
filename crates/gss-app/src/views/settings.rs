use constants::icons::{DOWN_ARROW, LEFT_ARROW};
use iced::widget::{Button, Checkbox, Column, Scrollable, TextInput, button, center, column, container, row, stack, text};
use iced::{Element, Length};
use types::internal::store::GameStore;

use crate::{Message, MainMessage};
use crate::components::custom_styles::{highlight_on_click_style, backdrop};
use crate::components::custom_widgets::message_dialog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    General,
    Email,
    Customize,
    Stores(GameStore),
}

pub fn view(app: &crate::App) -> Element<'_, Message> {
    let store_arrow_orientation = if app.store_settings_expanded {
        DOWN_ARROW
    } else {
        LEFT_ARROW
    };
    let mut stores_subsection = column![
        button(text(format!("{}  Store(s)", store_arrow_orientation)))
        .on_press(MainMessage::StoreSettingsExpanded(!app.store_settings_expanded).into())
        .width(Length::Fill)
        .style(button::text),
    ];
    if app.store_settings_expanded {
        stores_subsection = stores_subsection.push(
            row![
                button(text(format!("{}Steam", " ".repeat(8))))
                .on_press(MainMessage::PageSelected(Page::Stores(GameStore::STEAM)).into())
                .width(Length::Fill)
                .style(|theme, status| highlight_on_click_style(theme, status, app.settings_page == Page::Stores(GameStore::STEAM)))
            ]    
        )
    }

    let sidebar = column![
        button(text("General"))
            .on_press(MainMessage::PageSelected(Page::General).into())
            .width(Length::Fill)
            .style(|theme, status| highlight_on_click_style(theme, status, app.settings_page == Page::General)),
        button(text("Email"))
            .on_press(MainMessage::PageSelected(Page::Email).into())
            .width(Length::Fill)
            .style(|theme, status| highlight_on_click_style(theme, status, app.settings_page == Page::Email)),
        button(text("Customize"))
            .on_press(MainMessage::PageSelected(Page::Customize).into())
            .width(Length::Fill)
            .style(|theme, status| highlight_on_click_style(theme, status, app.settings_page == Page::Customize)),
        stores_subsection,
    ]
    .spacing(8)
    .padding(20)
    .width(Length::Fixed(220.0));

    let content: Element<'_, Message> = match app.settings_page {
        Page::General => general_settings(app),
        Page::Email => email_settings(app),
        Page::Customize => customize_settings(app),
        Page::Stores(store_page) => {
            match store_page {
                GameStore::STEAM => steam_settings(app),
                GameStore::GOOD_OLD_GAMES => todo!(),
                GameStore::MICROSOFT_STORE_PC => todo!(),
            } 
        }
    };

    let body = row![
        container(sidebar)
            .style(container::bordered_box),
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(32),
    ]
    .height(Length::Fill);

    let content = column![
        body,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    if app.show_dialog{
        stack![
            content,
            backdrop(MainMessage::HideDialog.into()),
            center(message_dialog(
                "INFO",
                "Settings were saved successfully.",
                MainMessage::CloseSettings.into())
            )
        ]
        .into()
    } else {
        content.into()
    }
}


pub fn store_selection(app: &crate::App) -> Column<'_, Message> {
    app.available_stores.iter().fold(column![], |column, game_store| {
        let label = game_store.get_name();
        column.push(
            Checkbox::new(app.selected_stores.contains(game_store))
                .label(label)
                .on_toggle(move |enabled| MainMessage::ToggleStore(game_store.clone(), enabled).into())
                .width(Length::Fill),
        )
    }).into()
}

pub fn alias_settings(app: &crate::App) -> Column<'_, Message>{
    column![
        Checkbox::new(app.alias_enabled)
            .label("Enable aliases")
            .on_toggle(|toggle| MainMessage::ToggleAliasEnabled(toggle).into())
            .width(Length::Fill),
        Checkbox::new(app.alias_reuse_enabled)
            .label("Enable alias reuse")
            .on_toggle(|toggle| MainMessage::ToggleAliasReuse(toggle).into())
            .width(Length::Fill),
    ].into()
}

fn general_settings(app: &crate::App) -> Element<'_, Message>{
    let content = column![
        row![
            if app.test_mode {
                column![
                text("Test path"),
                TextInput::new("Enter test path", &app.test_path)
                    .on_input(|input| MainMessage::TestPathChanged(input).into())
                    .padding(5)
                    .width(Length::Fill),
                ].spacing(4)
            } else {
                column![
                text("Project path"),
                TextInput::new("Enter project path", &app.project_path)
                    .on_input(|input| MainMessage::ProjectPathChanged(input).into())
                    .padding(5)
                    .width(Length::Fill),
                ]
                .spacing(4)
            }, 
            Checkbox::new(app.test_mode)
                .label("Enable test mode")
                .on_toggle(|toggle| MainMessage::ToggleTestMode(toggle).into())
                .width(Length::Fixed(150.))
                .spacing(10), 
        ]
        .spacing(10),
        text("Selected Stores"),
        store_selection(app),
        row![
            Button::new(text("Select None"))
                .on_press(MainMessage::SelectNoStores.into())
                .padding(4),
            Button::new(text("Select All"))
                .on_press(MainMessage::SelectAllStores.into())
                .padding(4),
        ]
        .spacing(10),
        text("Alias Settings"),
        alias_settings(app),
    ]
    .spacing(12)
    .padding(10);

    let scrollable_content = Scrollable::new(content).height(Length::Fill);
        
    column![
        text("General Settings").size(18),
        scrollable_content
            .height(Length::Fill),
        Button::new(text("Save Settings"))
            .on_press(MainMessage::SaveSettings.into())
            .padding(10),
    ].into()
}

fn email_settings(app: &crate::App) -> Element<'_, Message> {

    let content = column![
        column![
            text("Recipient email"),
            TextInput::new("Enter recipient email", &app.recipient_email)
                .on_input(|input| MainMessage::RecipientEmailChanged(input).into())
                .padding(5)
                .width(Length::Fill),
        ]
        .spacing(4),
        row![
            column![
                text("SMTP host"),
                TextInput::new("Enter SMTP host", &app.smtp_host)
                    .on_input(|input| MainMessage::SmtpHostChanged(input).into())
                    .padding(5)
                    .width(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill),
            column![
                text("SMTP port"),
                TextInput::new("Enter SMTP port", &app.smtp_port)
                    .on_input(|input| MainMessage::SmtpPortChanged(input).into())
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
                    .on_input(|input| MainMessage::SmtpEmailChanged(input).into())
                    .padding(5)
                    .width(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill),
            column![
                text("SMTP user"),
                TextInput::new("Enter SMTP user", &app.smtp_user)
                    .on_input(|input| MainMessage::SmtpUserChanged(input).into())
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
                .on_input(|input| MainMessage::SmtpPasswordChanged(input).into())
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
            Checkbox::new(app.reveal_sensitive_data)
                .label("Reveal sensitive data")
                .on_toggle(|toggle| MainMessage::ToggleSensitiveData(toggle).into())
                .spacing(10)
        ].padding(10),
        
    ]
    .spacing(12)
    .padding(10);

    let scrollable_content = Scrollable::new(content).height(Length::Fill);
        
    column![
        text("Email Settings").size(18),
        scrollable_content
            .height(Length::Fill),
        Button::new(text("Save Settings"))
            .on_press(MainMessage::SaveSettings.into())
            .padding(10),
    ].into()
}

fn customize_settings(_app: &crate::App) -> Element<'_, Message> {
    text("Customize Settings").size(18)
        .into()
}

fn steam_settings(app: &crate::App) -> Element<'_, Message> {
    let mut cache_button =  Button::new(text("Update Game Cache"))
        .padding(8);
    if !app.is_caching_in_progress {
        cache_button = cache_button
           .on_press(MainMessage::UpdateCache.into())
    } 

    let content = column![
        row![
            Checkbox::new(app.reveal_sensitive_data)
                .label("Reveal sensitive data")
                .on_toggle(|toggle| MainMessage::ToggleSensitiveData(toggle).into())
                .spacing(10)
        ].padding(10),
        column![
            text("Steam API key"),
            if app.reveal_sensitive_data {
                TextInput::new("Enter Steam API key", &app.steam_api_key)
                    .on_input(|input| MainMessage::SteamApiKeyChanged(input).into())
                    .padding(5)
                    .width(Length::Fill)
            } else {
                TextInput::new("Enter Steam API key", &app.steam_api_key)
                    .padding(5)
                    .width(Length::Fill)
            }
        ]
        .spacing(4),
        row![
            cache_button,
            text("[Placeholder for cache update status]")
        ]
        .spacing(10)
    ]
    .spacing(12);

    let scrollable_content = Scrollable::new(content).height(Length::Fill);

    column![
        text("Steam Settings").size(18),
        scrollable_content
            .height(Length::Fill),
        Button::new(text("Save Settings"))
            .on_press(MainMessage::SaveSettings.into())
            .padding(10),
    ].into()
}