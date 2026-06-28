use file_ops::settings;
use iced::widget::{Button, Scrollable,scrollable, column, container, rich_text, row, span, table, text, center_x, center_y};
use iced::{font, Element, Font, Length};

use crate::components::{custom_widgets, custom_styles};

use crate::Message;

pub enum ActionDisplayed{
    NoAction,
    CheckPrices,
    UpdateCache,
    TestEmail,
    Logs,
}

pub fn view_actions(app: &crate::App) -> Element<'_, Message> {
    let mut sales_by_store = column![];
    for (store, games) in app.sales_info_by_store.iter() {
        if games.is_empty() { continue; }
        let store_name = settings::get_proper_store_name(store).unwrap_or_else(|| store.clone());
        sales_by_store = sales_by_store.push(
            container(text(store_name).size(40.0)
                .font(Font{
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                }))
                .center_x(Length::Fill)
                .padding(10.0)
        );
        sales_by_store = sales_by_store.push(container(custom_widgets::create_sales_table( &games)).center_x(Length::Fill));
    }

    let sale_info_table_scrollable = Scrollable::new(column![
        text(format!("Status: {}", app.status_message)),
        sales_by_store
    ])//sale_info_table)
    .width(Length::Fill);

    let logs_display = Scrollable::new(text(&app.log))
        .width(Length::Fill)
        .height(Length::Fill);

    let status_display = Scrollable::new(text(&app.status_message));

    let display_results = match app.action_displayed {
        ActionDisplayed::CheckPrices => sale_info_table_scrollable,
        ActionDisplayed::Logs => logs_display,
        ActionDisplayed::TestEmail => status_display,
        ActionDisplayed::UpdateCache => status_display,
        ActionDisplayed::NoAction => Scrollable::new(text(""))
    };

    column![
        row![
            Button::new(text("Check prices"))
                .on_press(Message::CheckPrices)
                .padding(10),
            Button::new(text("Send Test Email"))
                .on_press(Message::SendEmail)
                .padding(10),
            Button::new(text("Update Game Cache"))
                .on_press(Message::UpdateCache)
                .padding(10),
            Button::new(text("Show Logs"))
                .on_press(Message::LogsShown)
                .padding(10)
        ]
        .spacing(10),
        display_results,
    ]
    .spacing(15)
    .padding(10)
    .into()
}