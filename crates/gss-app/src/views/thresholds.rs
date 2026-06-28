use iced::widget::{column, row, container, text, TextInput, Button, Scrollable, button};
use iced::{Element, Length};

use crate::{Message, SortColumn, SortOrder};

pub fn thresholds_tab(app: &crate::App) -> Element<'_, Message> {
    let ignore_case_query = app.search_query.to_lowercase();
    let mut thresholds_to_show: Vec<usize> = app.thresholds.iter().enumerate()
        .filter(|(_, threshold)| {
            if ignore_case_query.is_empty() { true } 
            else {
                threshold.title.to_lowercase().contains(&ignore_case_query) || threshold.alias.to_lowercase().contains(&ignore_case_query)
            }
        })
        .map(|(index, _)| index)
        .collect();

    if let Some(column) = app.threshold_sort_column {
        thresholds_to_show.sort_by(|thresh_a, thresh_b| {
            let thresh_cmp = match column {
                SortColumn::Title => app.thresholds[*thresh_a].title.to_lowercase().cmp(&app.thresholds[*thresh_b].title.to_lowercase()),
                SortColumn::Alias => app.thresholds[*thresh_a].alias.to_lowercase().cmp(&app.thresholds[*thresh_b].alias.to_lowercase()),
                SortColumn::SteamId => app.thresholds[*thresh_a].steam_id.cmp(&app.thresholds[*thresh_b].steam_id),
                SortColumn::GogId => app.thresholds[*thresh_a].gog_id.cmp(&app.thresholds[*thresh_b].gog_id),
                SortColumn::MicrosoftId => app.thresholds[*thresh_a].microsoft_store_id.to_lowercase().cmp(&app.thresholds[*thresh_b].microsoft_store_id.to_lowercase()),
                SortColumn::DesiredPrice => app.thresholds[*thresh_a].desired_price.partial_cmp(&app.thresholds[*thresh_b].desired_price).unwrap_or(std::cmp::Ordering::Equal),
            };

            if app.threshold_sort_order == SortOrder::Ascending {
                thresh_cmp
            } else {
                thresh_cmp.reverse()
            }
        });
    }

    let header_row = row![
        Button::new(text(format!("Title {}", header_sort_indicator(app, SortColumn::Title))))
            .on_press(Message::SortThresholds(SortColumn::Title))
            .style(button::text)
            .width(Length::FillPortion(2))
            .padding(8),
        Button::new(text(format!("Alias {}", header_sort_indicator(app, SortColumn::Alias))))
            .on_press(Message::SortThresholds(SortColumn::Alias))
            .style(button::text)
            .width(Length::FillPortion(2))
            .padding(8),
        container(
            Button::new(text(format!("Steam {}", header_sort_indicator(app, SortColumn::SteamId))))
                .on_press(Message::SortThresholds(SortColumn::SteamId))
                .style(button::text)
                .width(Length::Fill)
                .padding(8),
        )
        .width(Length::Fixed(100.0)),
        container(
            Button::new(text(format!("GOG {}", header_sort_indicator(app, SortColumn::GogId))))
                .on_press(Message::SortThresholds(SortColumn::GogId))
                .style(button::text)
                .width(Length::Fill)
                .padding(8),
        )
        .width(Length::Fixed(100.0)),
        container(
            Button::new(text(format!("Microsoft(PC) {}", header_sort_indicator(app, SortColumn::MicrosoftId))))
                .on_press(Message::SortThresholds(SortColumn::MicrosoftId))
                .style(button::text)
                .width(Length::Fill)
                .padding(8),
        )
        .width(Length::Fixed(140.0)),
        Button::new(text(format!("Price {}", header_sort_indicator(app, SortColumn::DesiredPrice))))
            .on_press(Message::SortThresholds(SortColumn::DesiredPrice))
            .style(button::text)
                .width(Length::Fill)
            .padding(8),
        container(text("Actions")).width(Length::FillPortion(1)).padding(8),
    ]
    .spacing(4);

    let header_row_seperator = container(iced::widget::rule::horizontal(2)).padding(4);
    let mut threshold_rows = column![header_row, header_row_seperator];
    let thresholds_empty: bool = thresholds_to_show.is_empty();
    for &index in thresholds_to_show.iter() {
        let threshold = &app.thresholds[index];
        let alias_value = app.threshold_alias_edits.get(index).map(String::as_str).unwrap_or(threshold.alias.as_str());
        let price_value = app.threshold_price_edits.get(index).map(String::as_str).unwrap_or("ERR");

        threshold_rows = threshold_rows.push(
            container(
                row![
                    container(text(threshold.title.clone())).width(Length::FillPortion(2)).padding(8),
                    TextInput::new("alias", alias_value)
                        .on_input(move |value| Message::ThresholdAliasChanged(index, value))
                        .width(Length::FillPortion(2))
                        .padding(5),
                    container(text(if threshold.steam_id != 0 { "✔" } else { "" })).center_x(Length::Fixed(100.0)).padding(8),
                    container(text(if threshold.gog_id != 0 { "✔" } else { "" })).center_x(Length::Fixed(100.0)).padding(8),
                    container(text(if threshold.microsoft_store_id.is_empty() { "" } else { "✔" })).center_x(Length::Fixed(140.0)).padding(8),
                    TextInput::new("price", &price_value)
                        .on_input(move |value| Message::ThresholdPriceChanged(index, value))
                        .width(Length::FillPortion(1))
                        .padding(5),
                    row![
                        Button::new(text("💾")).on_press(Message::UpdateThresholdRow(index)).padding(6),
                        Button::new(text("🗑️")).on_press(Message::RemoveThresholdRow(index)).padding(6),
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(4),
            )            .padding(2),
        );
    }

    let threshold_controls = column![
        TextInput::new("Search thresholds", &app.search_query)
            .on_input(Message::SearchQueryChanged)
            .padding(5)
            .width(Length::Fill),
        row![
            Button::new(text("Reset search")).on_press(Message::SearchQueryChanged(String::new())).padding(6),
            Button::new(text("Refresh list")).on_press(Message::Refresh).padding(6),
        ]
        .spacing(8),
    ]
    .spacing(10)
    .padding(10);

    let thresholds_list = if thresholds_empty {
        Scrollable::new(column![text("No thresholds found.")]).height(Length::Fill)
    } else {
        Scrollable::new(threshold_rows).height(Length::Fill)
    };

    column![threshold_controls, thresholds_list].spacing(12).into()
}

fn header_sort_indicator(app: &crate::App, column: SortColumn) -> &'static str {
    if app.threshold_sort_column == Some(column) {
        match app.threshold_sort_order {
            SortOrder::Ascending => "▲",
            SortOrder::Descending => "▼",
            SortOrder::Original => "",
        }
    } else {
        ""
    }
}