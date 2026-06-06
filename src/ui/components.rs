use crate::app::{App, Message};
use crate::ui::styles;
use athan::core::*;

use iced::widget::{column, pick_list, row, text, text_input, toggler, Space};
use iced::{Alignment, Element, Fill};
use std::borrow::Borrow;

pub fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(11).color(styles::TEXT_MUTED),
        text_input(label, value)
            .on_input(on_change)
            .padding(8)
            .style(styles::text_input)
    ]
    .spacing(4)
    .width(Fill)
    .into()
}

pub fn labeled_picker<'a, T, L>(
    label: &'a str,
    options: L,
    selected: Option<T>,
    on_change: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'static,
    L: Borrow<[T]> + 'a,
{
    column![
        text(label).size(11).color(styles::TEXT_MUTED),
        pick_list(options, selected, on_change)
            .width(Fill)
            .padding(8)
            .style(styles::pick_list)
    ]
    .spacing(4)
    .into()
}

pub fn toggle_row<'a>(label: &'a str, is_active: bool, on_toggle: impl Fn(bool) -> Message + 'a) -> Element<'a, Message> {
    row![
        text(label).size(14).color(styles::TEXT_MUTED),
        Space::new().width(Fill),
        toggler(is_active).on_toggle(on_toggle),
    ]
    .align_y(Alignment::Center)
    .into()
}

pub fn adjustment_input<'a>(app: &'a App, prayer: Prayer) -> Element<'a, Message> {
    labeled_input(prayer.name(), app.adjustment_input(prayer), move |value| {
        Message::AdjustmentChanged(prayer, value)
    })
}

pub fn adjustment_grid(app: &App) -> Element<'_, Message> {
    let rows: Vec<Element<Message>> = Prayer::all()
        .chunks(2)
        .map(|chunk| {
            let inputs: Vec<Element<Message>> = chunk
                .iter()
                .copied()
                .map(|prayer| adjustment_input(app, prayer))
                .collect();

            iced::widget::Row::with_children(inputs).spacing(12).into()
        })
        .collect();

    iced::widget::Column::with_children(rows).spacing(12).into()
}

pub fn adjustment_summary(adjustments: PrayerAdjustments) -> String {
    Prayer::all()
        .iter()
        .copied()
        .filter_map(|prayer| {
            let minutes = adjustments.get(prayer);
            (minutes != 0).then(|| format!("{} {minutes:+}", prayer.name()))
        })
        .collect::<Vec<_>>()
        .join("  ")
}
