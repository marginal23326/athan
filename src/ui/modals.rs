use crate::app::{App, Message};
use crate::ui::components::{adjustment_grid, adjustment_summary, labeled_input, labeled_picker, toggle_row};
use crate::ui::styles;
use athan::core::*;

use iced::widget::text::Wrapping;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Fill};

fn modal_header<'a>(title: &'a str, on_close: Message) -> Element<'a, Message> {
    row![
        text(title).size(18).color(styles::TEXT_PRIMARY),
        Space::new().width(Fill),
        button(text("Close").size(13))
            .on_press(on_close)
            .style(styles::button)
            .padding([4, 12]),
    ]
    .align_y(Alignment::Center)
    .into()
}

pub fn settings_modal(app: &App) -> Element<'_, Message> {
    let header = modal_header("Configuration", Message::ToggleSettings);

    let location_grid = column![
        row![
            labeled_input("City Name", &app.inputs.loc_name_input, Message::LocationNameChanged),
            column![
                text("UTC Offset (Hrs)").size(11).color(styles::TEXT_MUTED),
                text_input("UTC Offset (Hrs)", &app.inputs.tz_input)
                    .on_input(Message::TimezoneChanged)
                    .padding(8)
                    .style(styles::text_input)
                    .width(Fill),
            ]
            .spacing(4)
            .width(Fill)
        ]
        .spacing(12),
        row![
            labeled_input("Latitude", &app.inputs.lat_input, Message::LatitudeChanged),
            labeled_input("Longitude", &app.inputs.lon_input, Message::LongitudeChanged),
        ]
        .spacing(12),
        labeled_input("Elevation (m)", &app.inputs.elv_input, Message::ElevationChanged)
    ]
    .spacing(12);

    let calc_group = column![
        text("Methodology").size(13).color(styles::TEXT_PRIMARY),
        labeled_picker(
            "Fajr & Isha Convention",
            CalculationMethod::variants(),
            Some(app.calculation_method),
            Message::MethodChanged
        ),
        labeled_picker(
            "Asr Juristic Rule",
            AsrMethod::variants(),
            Some(app.asr_method),
            Message::AsrMethodChanged
        ),
    ]
    .spacing(12);

    let adjustment_text = adjustment_summary(app.prayer_adjustments);
    let adjustment_group = column![
        text("Minute Adjustments").size(13).color(styles::TEXT_PRIMARY),
        row![
            text(if adjustment_text.is_empty() {
                "All 0".into()
            } else {
                adjustment_text
            })
            .size(12)
            .color(styles::TEXT_MUTED)
            .wrapping(Wrapping::WordOrGlyph),
            Space::new().width(Fill),
            button(text("Edit").size(12))
                .on_press(Message::ToggleAdjustments)
                .style(styles::button)
                .padding([6, 14])
        ]
        .spacing(12)
        .align_y(Alignment::Center)
    ]
    .spacing(8);

    let interface_group = column![
        text("Interface Options").size(13).color(styles::TEXT_PRIMARY),
        toggle_row("Show Arabic Names", app.show_arabic, |_| Message::ToggleArabic),
        toggle_row("Show Hijri Date", app.show_hijri, |_| Message::ToggleHijri),
    ]
    .spacing(12);

    let location_section = column![
        text("Location Data").size(13).color(styles::TEXT_PRIMARY),
        location_grid,
        button(text("Auto Detect Location").size(12))
            .on_press(Message::DetectLocation)
            .style(styles::button)
            .padding([6, 14])
            .width(Fill),
    ]
    .spacing(12);

    let mut inner_content =
        column![header, location_section, calc_group, adjustment_group, interface_group].spacing(20);

    if let Some(err) = &app.error {
        inner_content = inner_content.push(text(err).size(13).color(styles::ERROR));
    }

    container(scrollable(inner_content.padding(20)))
        .width(360)
        .style(styles::surface_card)
        .into()
}

pub fn adjustments_modal(app: &App) -> Element<'_, Message> {
    let header = modal_header("Minute Adjustments", Message::ToggleAdjustments);

    container(scrollable(
        column![header, adjustment_grid(app)].spacing(20).padding(20),
    ))
    .width(360)
    .style(styles::surface_card)
    .into()
}
