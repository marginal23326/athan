use crate::app::{App, Message};
use crate::ui::styles;
use athan::core::*;

use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, row, rule, scrollable, space, text, Space};
use iced::{Alignment, Element, Fill, Font};

pub fn main_view(app: &App) -> Element<'_, Message> {
    let offset = time::UtcOffset::from_whole_seconds((app.location.timezone_offset * 3600.0) as i32)
        .unwrap_or(time::UtcOffset::UTC);
    let now_local = app.now.to_offset(offset);

    let next_p = app
        .prayer_times
        .as_ref()
        .map(|times| next_prayer(times, now_local.time()));

    let top_bar = row![
        column![
            text(&app.location.name).size(18).color(styles::TEXT_PRIMARY),
            if app.show_hijri {
                text(if app.show_arabic {
                    app.hijri_date.arabic_display()
                } else {
                    app.hijri_date.display()
                })
                .size(13)
                .color(styles::ACCENT)
            } else {
                text(now_local.format(&*DATE_FMT).unwrap_or_default())
                    .size(13)
                    .color(styles::TEXT_MUTED)
            }
        ]
        .spacing(2),
        Space::new().width(Fill),
        button(text("Menu").size(13))
            .on_press(Message::ToggleSettings)
            .style(styles::button)
            .padding([6, 12]),
    ]
    .align_y(Alignment::Center);

    let footer = row![
        text(format!(
            "Qiblah: {:.1}° {}",
            app.qiblah,
            qiblah_compass_direction(app.qiblah)
        ))
        .size(12)
        .color(styles::TEXT_MUTED),
        Space::new().width(Fill),
        text(now_local.format(&*DATE_FMT).unwrap_or_default())
            .size(12)
            .color(styles::TEXT_MUTED),
    ]
    .align_y(Alignment::Center);

    container(scrollable(
        column![
            top_bar,
            hero_section(app, now_local, next_p),
            prayer_list(app, next_p),
            footer
        ]
        .spacing(24)
        .padding(24),
    ))
    .style(|_| iced::widget::container::Style {
        background: Some(styles::BG.into()),
        ..Default::default()
    })
    .width(Fill)
    .height(Fill)
    .into()
}

fn hero_section(
    app: &App,
    now_local: time::OffsetDateTime,
    next_p: Option<(Prayer, time::Time)>,
) -> Element<'_, Message> {
    let content = if app.prayer_times.is_some() {
        if let Some((prayer, ptime)) = next_p {
            let secs = time_until(ptime, now_local.time()).whole_seconds().max(0);
            row![
                column![
                    text("NEXT PRAYER").size(11).color(styles::TEXT_MUTED),
                    text(prayer.name().to_uppercase())
                        .size(30)
                        .color(styles::TEXT_PRIMARY),
                    text(format_time(ptime)).size(15).color(styles::TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Fill),
                text(format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60))
                    .size(46)
                    .color(styles::ACCENT)
                    .font(Font::MONOSPACE)
                    .wrapping(Wrapping::WordOrGlyph)
            ]
            .align_y(Alignment::Center)
        } else {
            row![text("No upcoming prayer").color(styles::TEXT_MUTED)]
        }
    } else {
        row![text("Cannot compute").color(styles::ERROR)]
    };

    container(content)
        .padding(24)
        .style(styles::outline_card)
        .width(Fill)
        .into()
}

fn prayer_list(app: &App, next_p: Option<(Prayer, time::Time)>) -> Element<'_, Message> {
    if let Some(times) = &app.prayer_times {
        let next_prayer_enum = next_p.map(|(p, _)| p);

        let prayers = times.as_array();

        let rows: Vec<Element<Message>> = prayers
            .iter()
            .enumerate()
            .map(|(i, &(prayer, time))| {
                let is_next = next_prayer_enum == Some(prayer);
                let text_col = if is_next {
                    styles::ACCENT
                } else {
                    styles::TEXT_PRIMARY
                };

                let row_content = container(
                    row![
                        text(prayer.name()).size(15).color(text_col),
                        Space::new().width(Fill),
                        if app.show_arabic && prayer.name() != prayer.arabic_name() {
                            Element::from(text(prayer.arabic_name()).size(14).color(if is_next {
                                styles::ACCENT
                            } else {
                                styles::TEXT_MUTED
                            }))
                        } else {
                            Element::from(space())
                        },
                        Space::new().width(12),
                        text(format_time(time))
                            .size(15)
                            .color(text_col)
                            .font(Font::MONOSPACE),
                    ]
                    .align_y(Alignment::Center)
                    .padding([16, 20]),
                )
                .style(move |_| iced::widget::container::Style {
                    background: Some(
                        if is_next {
                            styles::ACCENT_MUTED
                        } else {
                            iced::Color::TRANSPARENT
                        }
                        .into(),
                    ),
                    border: if is_next {
                        iced::Border {
                            color: iced::Color::TRANSPARENT,
                            width: 0.0,
                            radius: 16.0.into(),
                        }
                    } else {
                        iced::Border::default()
                    },
                    ..Default::default()
                });

                if i < prayers.len() - 1 {
                    column![
                        row_content,
                        rule::horizontal(1).style(|_| rule::Style {
                            color: styles::BORDER,
                            radius: 0.0.into(),
                            fill_mode: rule::FillMode::Full,
                            snap: true
                        })
                    ]
                    .into()
                } else {
                    row_content.into()
                }
            })
            .collect();

        container(iced::widget::Column::with_children(rows))
            .style(styles::surface_card)
            .width(Fill)
            .into()
    } else {
        text("Could not calculate prayer times.")
            .size(14)
            .color(styles::TEXT_MUTED)
            .into()
    }
}
