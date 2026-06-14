use crate::app::Message;
use crate::ui::styles;
use athan::core::*;

use iced::widget::{Space, column, pick_list, row, text, text_input, toggler};
use iced::{Alignment, Element, Fill};
use std::borrow::Borrow;

pub fn can_be_float(s: &str) -> bool {
    s.is_empty()
        || s.parse::<f64>().is_ok()
        || matches!(s, "-" | "+" | "." | "-." | "+.")
}

pub fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    is_invalid: bool,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let label_color = if is_invalid { styles::ERROR } else { styles::TEXT_MUTED };
    let border_style = if is_invalid {
        styles::text_input_invalid
    } else {
        styles::text_input
    };

    column![
        text(label).size(11).color(label_color),
        text_input(label, value)
            .on_input(on_change)
            .padding(8)
            .style(border_style)
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

pub fn toggle_row<'a>(
    label: &'a str,
    is_active: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).size(14).color(styles::TEXT_MUTED),
        Space::new().width(Fill),
        toggler(is_active).on_toggle(on_toggle),
    ]
    .align_y(Alignment::Center)
    .into()
}

pub fn adjustment_input<'a>(inputs: &'a [String; Prayer::COUNT], prayer: Prayer) -> Element<'a, Message> {
    let val = &inputs[prayer.index()];
    let is_invalid = val.parse::<i32>().map_or(true, |v| !(-120..=120).contains(&v));
    labeled_input(prayer.name(), val, is_invalid, move |value| {
        Message::AdjustmentChanged(prayer, value)
    })
}

pub fn adjustment_grid(inputs: &[String; Prayer::COUNT]) -> Element<'_, Message> {
    let rows: Vec<Element<Message>> = Prayer::all()
        .chunks(2)
        .map(|chunk| {
            let elems: Vec<Element<Message>> = chunk
                .iter()
                .copied()
                .map(|prayer| adjustment_input(inputs, prayer))
                .collect();

            iced::widget::Row::with_children(elems).spacing(12).into()
        })
        .collect();

    iced::widget::Column::with_children(rows).spacing(12).into()
}
