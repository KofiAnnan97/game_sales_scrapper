use crate::app::enums::{Tab, View};
use crate::components::custom_widgets as cw;
use crate::views::settings as sttngs_view;
use crate::views::settings::{alias_settings, store_selection};
use crate::{
    app::{
        message::{MainMessage, Message},
        state::App,
    },
    tabs,
    views::sub_windows::manual_prune,
};
use iced::widget::container;
use iced::{
    Element, Length, Padding,
    widget::{Button, column, row, text},
    window,
};
use iced_aw::{Menu, TabBarPosition, TabLabel, Tabs, menu, menu_bar, menu_items};

pub fn view(app: &App, window_id: window::Id) -> Element<'_, Message> {
    if Some(window_id) == app.manual_prune_window {
        return manual_prune(app);
    }

    let quick_file_menu = Menu::new(menu_items!(
        (text("Selected Stores").size(20)),
        (store_selection(app)),
        (row![
            Button::new(text("Select None"))
                .on_press(MainMessage::SelectNoStores.into())
                .padding(4),
            Button::new(text("Select All"))
                .on_press(MainMessage::SelectAllStores.into())
                .padding(4),
        ]
        .spacing(10)),
        (text("Alias Settings").size(20)),
        (alias_settings(app)),
    ))
    .width(280.0);

    // let customize_menu = Menu::new(menu_items!(
    //     (text("Font size")),
    //     (text("Themes..."))
    // )).width(320.0);

    let file_menu = Menu::new(menu_items!(
        // (cw::submenu_button("Customize"), customize_menu),
        (cw::submenu_button("Quick Settings..."), quick_file_menu),
        (cw::menu_text_button("Settings", MainMessage::OpenSettings.into())),
    ))
    .width(320.0);

    let actions_menu = Menu::new(menu_items!(
        (cw::menu_text_button("Preview Sales", MainMessage::OpenSalesPreview.into())),
        // (text!("Edit Alert Schedule")),
        // (text("Update cache")),
        (cw::menu_text_button("Logs", MainMessage::OpenLogsView.into())),
    ))
    .width(320.0)
    .close_on_item_click(true);

    let menu_bar = menu_bar!(
        (container(text("File")), file_menu),
        (container(text("Actions")), actions_menu)
    )
    .spacing(5.0)
    .padding(Padding::new(4.0))
    .draw_path(menu::DrawPath::Backdrop)
    .close_on_background_click_global(true);

    let base_view = Tabs::new(|tab| Message::Main(MainMessage::TabSelected(tab)))
        .push(
            Tab::Search,
            TabLabel::Text(String::from("Search")),
            tabs::search::search_tab(app),
        )
        .push(
            Tab::Thresholds,
            TabLabel::Text(String::from("Thresholds")),
            app.view_thresholds(),
        )
        .set_active_tab(&app.tab)
        .tab_bar_position(TabBarPosition::Top)
        .width(Length::Fill);

    let top_row = row![menu_bar,].width(Length::Fill).spacing(0);
    let mut visible_tabs_count: u32 = 0;

    let tab_bar = {
        let mut bar = row![];
        for view in app.view_bar.iter() {
            if view == &View::Settings {
                bar = bar.push(cw::closable_window_button(
                    "Settings",
                    MainMessage::OpenSettings.into(),
                    Some(MainMessage::CloseSettings.into()),
                    app.active_view == View::Settings,
                ));
                visible_tabs_count += 1;
            }
            if view == &View::Preview {
                bar = bar.push(cw::closable_window_button(
                    "Preview",
                    MainMessage::OpenSalesPreview.into(),
                    Some(MainMessage::CloseSalesPreview.into()),
                    app.active_view == View::Preview,
                ));
                visible_tabs_count += 1;
            }
            if view == &View::Logs {
                bar = bar.push(cw::closable_window_button(
                    "Logs",
                    MainMessage::OpenLogsView.into(),
                    Some(MainMessage::CloseLogsView.into()),
                    app.active_view == View::Logs,
                ));
                visible_tabs_count += 1;
            }
        }
        bar.padding(1)
    };

    let settings_window: Element<'_, Message> = sttngs_view::view(app);
    let preview_window: Element<'_, Message> = app.preview_view.view().map(Message::Preview);
    let logs_window: Element<'_, Message> = app.logging_view.view().map(Message::Logging);

    let right_pane = match app.active_view {
        View::Base => {
            // if app.show_dialog && app.tab == {
            //     stack![
            //         base_view,
            //         cs::backdrop(MainMessage::HideDialog.into()),
            //         center(message_dialog(
            //             &app.status_message,
            //             &app.message_details,
            //             MainMessage::HideDialog.into()
            //         ))
            //     ]
            //     .into()
            // } else {
            base_view.into()
            // }
        }
        View::Settings => settings_window,
        View::Preview => preview_window,
        View::Logs => logs_window,
    };

    let content = column![
        top_row,
        if visible_tabs_count > 0 {
            row![
                cw::closable_window_button(
                    "General",
                    MainMessage::OpenBaseView.into(),
                    None,
                    app.active_view == View::Base
                ),
                tab_bar
            ]
        } else {
            row![tab_bar]
        },
        container(right_pane).width(Length::Fill).padding(5),
    ]
    .spacing(5);

    content.into()
}
