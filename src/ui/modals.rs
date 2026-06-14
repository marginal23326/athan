use crate::app::{App, Message, SettingsTab};
use athan::core::{ELV_RANGE, LAT_RANGE, LON_RANGE, TZ_RANGE};
use crate::ui::components::{adjustment_grid, is_valid_float, labeled_input, labeled_picker, toggle_row};
use crate::ui::styles;
use athan::core::*;

use iced::widget::text::Wrapping;
use iced::widget::{Space, button, column, container, row, scrollable, text};
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

fn tab_button<'a>(text_label: &'a str, tab: SettingsTab, current_tab: SettingsTab) -> Element<'a, Message> {
    let is_active = tab == current_tab;
    button(
        text(text_label)
            .size(12)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Fill),
    )
    .width(Fill)
    .padding([6, 0])
    .on_press(Message::SetSettingsTab(tab))
    .style(move |_theme, status| {
        let base_text_color = if is_active { styles::ACCENT } else { styles::TEXT_MUTED };
        let bg = if is_active { Some(styles::SURFACE.into()) } else { None };

        let mut style = iced::widget::button::Style {
            text_color: base_text_color,
            background: bg,
            border: iced::Border {
                radius: 6.0.into(),
                width: if is_active { 1.5 } else { 0.0 },
                color: if is_active {
                    styles::BORDER
                } else {
                    iced::Color::TRANSPARENT
                },
            },
            ..Default::default()
        };

        if status == iced::widget::button::Status::Hovered {
            style.text_color = if is_active {
                styles::ACCENT
            } else {
                styles::TEXT_PRIMARY
            };
            if !is_active {
                style.background = Some(styles::SURFACE_HIGHLIGHT.into());
            }
        }
        style
    })
    .into()
}

pub fn settings_modal(app: &App) -> Element<'_, Message> {
    let header = modal_header("Configuration", Message::ToggleSettings);

    // Segmented control style tab bar
    let tab_bar = container(
        row![
            tab_button("Location", SettingsTab::Location, app.inputs.active_tab),
            tab_button("Method", SettingsTab::Calculation, app.inputs.active_tab),
            tab_button("Adjust", SettingsTab::Adjustments, app.inputs.active_tab),
            tab_button("Audio", SettingsTab::Audio, app.inputs.active_tab),
            tab_button("UI", SettingsTab::Interface, app.inputs.active_tab),
        ]
        .spacing(4),
    )
    .padding(4)
    .style(|_| iced::widget::container::Style {
        background: Some(styles::BG.into()),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: styles::BORDER,
        },
        ..Default::default()
    });

    let active_content: Element<'_, Message> = match app.inputs.active_tab {
        SettingsTab::Location => {
            let is_name_invalid = app.inputs.loc_name_input.trim().is_empty();
            let is_tz_invalid = !is_valid_float(&app.inputs.tz_input, TZ_RANGE);
            let is_lat_invalid = !is_valid_float(&app.inputs.lat_input, LAT_RANGE);
            let is_lon_invalid = !is_valid_float(&app.inputs.lon_input, LON_RANGE);
            let is_elv_invalid = !is_valid_float(&app.inputs.elv_input, ELV_RANGE);

            let location_grid = column![
                row![
                    labeled_input(
                        "City Name",
                        &app.inputs.loc_name_input,
                        is_name_invalid,
                        Message::LocationNameChanged
                    ),
                    labeled_input(
                        "UTC Offset (Hrs)",
                        &app.inputs.tz_input,
                        is_tz_invalid,
                        Message::TimezoneChanged
                    ),
                ]
                .spacing(12),
                row![
                    labeled_input(
                        "Latitude",
                        &app.inputs.lat_input,
                        is_lat_invalid,
                        Message::LatitudeChanged
                    ),
                    labeled_input(
                        "Longitude",
                        &app.inputs.lon_input,
                        is_lon_invalid,
                        Message::LongitudeChanged
                    ),
                ]
                .spacing(12),
                labeled_input(
                    "Elevation (m)",
                    &app.inputs.elv_input,
                    is_elv_invalid,
                    Message::ElevationChanged
                )
            ]
            .spacing(12);

            #[allow(unused_mut)]
            let mut items: Vec<Element<Message>> = vec![
                text("Geographic Settings").size(13).color(styles::TEXT_PRIMARY).into(),
                location_grid.into(),
                toggle_row("Daylight Saving Time (+1 hr)", app.location.dst, |_| Message::ToggleDst),
            ];

            #[cfg(feature = "detect")]
            {
                let mut btn = button(
                    text(if app.is_detecting {
                        "Detecting…"
                    } else {
                        "Auto Detect Location"
                    })
                    .size(12),
                )
                .style(styles::button)
                .padding([6, 14])
                .width(Fill);

                if !app.is_detecting {
                    btn = btn.on_press(Message::DetectLocation);
                }
                items.push(btn.into());
            }

            if let Some(err) = &app.error {
                items.push(text(err).size(13).color(styles::ERROR).into());
            }

            iced::widget::Column::with_children(items).spacing(20).into()
        }

        SettingsTab::Calculation => column![
            text("Calculation Methodology").size(13).color(styles::TEXT_PRIMARY),
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
        .spacing(16)
        .into(),

        SettingsTab::Adjustments => column![
            text("Manual Minute Adjustments").size(13).color(styles::TEXT_PRIMARY),
            text("Offset specific prayer times by a number of minutes. Use negative values to make a prayer earlier.")
                .size(11)
                .color(styles::TEXT_MUTED)
                .wrapping(Wrapping::WordOrGlyph),
            Space::new().height(4),
            adjustment_grid(&app.inputs.adjustment_inputs)
        ]
        .spacing(12)
        .into(),

        SettingsTab::Audio => column![
            text("Audio & Notifications").size(13).color(styles::TEXT_PRIMARY),
            row![
                text("Adhan Volume").size(13).color(styles::TEXT_MUTED),
                Space::new().width(Fill),
                text(format!("{}%", (app.volume * 100.0).round() as i32))
                    .size(13)
                    .color(styles::TEXT_MUTED),
            ],
            iced::widget::slider(0.0..=1.0, app.volume, Message::VolumeChanged).step(0.01_f32)
        ]
        .spacing(16)
        .into(),

        SettingsTab::Interface => {
            #[allow(unused_mut)]
            let mut items: Vec<Element<Message>> = vec![
                text("Application Interface")
                    .size(13)
                    .color(styles::TEXT_PRIMARY)
                    .into(),
                toggle_row("Start on login (minimized)", app.start_on_boot, |_| {
                    Message::ToggleStartOnBoot
                }),
                toggle_row("Show Arabic Names", app.show_arabic, |_| Message::ToggleArabic),
                toggle_row("Use 24-Hour Format", app.use_24h, |_| Message::ToggleTimeFormat),
            ];

            #[cfg(feature = "hijri")]
            items.push(toggle_row("Show Hijri Date", app.show_hijri, |_| Message::ToggleHijri));

            iced::widget::Column::with_children(items).spacing(16).into()
        }
    };

    let inner_content = column![header, tab_bar, active_content].spacing(20);

    container(scrollable(inner_content.padding(20)))
        .width(400)
        .style(styles::surface_card)
        .into()
}
