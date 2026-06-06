mod ui;

use athan::core::*;

use iced::widget::{
    Space, button, center, column, container, mouse_area, opaque, pick_list, row, rule, scrollable, space, stack, text,
    text::Wrapping, text_input, toggler,
};
use iced::{Alignment, Element, Fill, Font, Subscription, Task};
use std::borrow::Borrow;

#[derive(Debug, Clone)]
enum Message {
    Tick(time::OffsetDateTime),
    MethodChanged(CalculationMethod),
    AsrMethodChanged(AsrMethod),
    LatitudeChanged(String),
    LongitudeChanged(String),
    TimezoneChanged(String),
    ElevationChanged(String),
    ToggleSettings,
    ToggleArabic,
    ToggleHijri,
    ToggleAdjustments,
    LocationNameChanged(String),
    AdjustmentChanged(Prayer, String),
    FocusNext,
    FocusPrevious,
    EscapePressed,
    DetectLocation,
    LocationDetected(Result<LocationData, String>),
}

struct SettingsState {
    pub lat_input: String,
    pub lon_input: String,
    pub tz_input: String,
    pub elv_input: String,
    pub loc_name_input: String,
    pub adjustment_inputs: [String; Prayer::COUNT],
}

struct App {
    location: Location,
    calculation_method: CalculationMethod,
    asr_method: AsrMethod,
    prayer_adjustments: PrayerAdjustments,
    prayer_times: Option<PrayerTimes>,
    hijri_date: HijriDate,
    qiblah: f64,
    now: time::OffsetDateTime,
    settings_open: bool,
    adjustments_open: bool,
    show_arabic: bool,
    show_hijri: bool,
    inputs: SettingsState,
    error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let now = time::OffsetDateTime::now_utc();
        let loc = Location::default();
        let calc = CalculationMethod::UmmAlQura;
        let asr = AsrMethod::Shafi;
        let adjustments = PrayerAdjustments::prayer_start_safety();
        let data = calculate_daily_prayer_data(now, &loc, calc, asr, adjustments);
        let err = Self::calculation_error(data.prayer_times);

        Self {
            location: loc.clone(),
            calculation_method: calc,
            asr_method: asr,
            prayer_adjustments: adjustments,
            prayer_times: data.prayer_times,
            hijri_date: data.hijri_date,
            qiblah: data.qiblah,
            now,
            settings_open: false,
            adjustments_open: false,
            show_arabic: false,
            show_hijri: false,
            inputs: SettingsState {
                lat_input: loc.coordinates.latitude.to_string(),
                lon_input: loc.coordinates.longitude.to_string(),
                tz_input: loc.timezone_offset.to_string(),
                elv_input: loc.elevation.to_string(),
                loc_name_input: loc.name.clone(),
                adjustment_inputs: Prayer::ALL.map(|prayer| adjustments.get(prayer).to_string()),
            },
            error: err,
        }
    }
}

impl App {
    fn recalculate(&mut self) {
        let data = calculate_daily_prayer_data(
            self.now,
            &self.location,
            self.calculation_method,
            self.asr_method,
            self.prayer_adjustments,
        );
        self.prayer_times = data.prayer_times;
        self.hijri_date = data.hijri_date;
        self.qiblah = data.qiblah;
        self.error = Self::calculation_error(data.prayer_times);
    }

    fn calculation_error(prayer_times: Option<PrayerTimes>) -> Option<String> {
        prayer_times
            .is_none()
            .then(|| "Cannot compute times for this date/location.".into())
    }

    fn adjustment_input(&self, prayer: Prayer) -> &str {
        &self.inputs.adjustment_inputs[prayer.index()]
    }

    fn set_adjustment_input(&mut self, prayer: Prayer, value: String) {
        self.inputs.adjustment_inputs[prayer.index()] = value;
    }
}

fn new() -> App {
    App::default()
}

fn update(app: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::Tick(now) => {
            let old = app.location.local_date(app.now);
            app.now = now;
            if app.location.local_date(app.now) != old {
                app.recalculate();
            }
        }
        Message::MethodChanged(m) => {
            app.calculation_method = m;
            app.recalculate();
        }
        Message::AsrMethodChanged(m) => {
            app.asr_method = m;
            app.recalculate();
        }
        Message::LatitudeChanged(s) => {
            app.inputs.lat_input = s.clone();
            if let Ok(v) = s.parse::<f64>() {
                app.location.coordinates.latitude = v;
                app.recalculate();
            }
        }
        Message::LongitudeChanged(s) => {
            app.inputs.lon_input = s.clone();
            if let Ok(v) = s.parse::<f64>() {
                app.location.coordinates.longitude = v;
                app.recalculate();
            }
        }
        Message::TimezoneChanged(s) => {
            app.inputs.tz_input = s.clone();
            if let Ok(v) = s.parse::<f64>() {
                app.location.timezone_offset = v;
                app.recalculate();
            }
        }
        Message::ElevationChanged(s) => {
            app.inputs.elv_input = s.clone();
            if let Ok(v) = s.parse::<f64>() {
                app.location.elevation = v;
                app.recalculate();
            }
        }
        Message::LocationNameChanged(s) => {
            app.inputs.loc_name_input = s.clone();
            app.location.name = s;
        }
        Message::AdjustmentChanged(prayer, s) => {
            app.set_adjustment_input(prayer, s.clone());
            if let Ok(minutes) = s.parse::<i32>()
                && (-120..=120).contains(&minutes)
            {
                app.prayer_adjustments.set(prayer, minutes);
                app.recalculate();
            }
        }
        Message::DetectLocation => {
            let future = async {
                tokio::task::spawn_blocking(detect_location)
                    .await
                    .map_err(|e| format!("Internal error: {e}"))?
            };

            return Task::perform(future, Message::LocationDetected);
        }
        Message::LocationDetected(Ok(data)) => {
            app.inputs.loc_name_input = data.name.clone();
            app.inputs.lat_input = data.lat.to_string();
            app.inputs.lon_input = data.lon.to_string();
            app.inputs.tz_input = format!("{:.1}", data.offset);
            app.inputs.elv_input = data.elevation.to_string();
            app.location.name = data.name;
            app.location.coordinates.latitude = data.lat;
            app.location.coordinates.longitude = data.lon;
            app.location.timezone_offset = data.offset;
            app.location.elevation = data.elevation;
            app.recalculate();
        }
        Message::LocationDetected(Err(e)) => {
            app.error = Some(e);
        }
        Message::ToggleSettings => {
            app.settings_open = !app.settings_open;
            if !app.settings_open {
                app.adjustments_open = false;
            }
        }
        Message::ToggleArabic => app.show_arabic = !app.show_arabic,
        Message::ToggleHijri => app.show_hijri = !app.show_hijri,
        Message::ToggleAdjustments => app.adjustments_open = !app.adjustments_open,
        Message::FocusNext => return iced::widget::operation::focus_next(),
        Message::FocusPrevious => return iced::widget::operation::focus_previous(),
        Message::EscapePressed => {
            if app.adjustments_open {
                app.adjustments_open = false;
            } else if app.settings_open {
                app.settings_open = false;
            }
        }
    }
    Task::none()
}

// UI Component Helpers

fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(11).color(ui::styles::TEXT_MUTED),
        text_input(label, value)
            .on_input(on_change)
            .padding(8)
            .style(ui::styles::text_input)
    ]
    .spacing(4)
    .width(Fill)
    .into()
}

fn labeled_picker<'a, T, L>(
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
        text(label).size(11).color(ui::styles::TEXT_MUTED),
        pick_list(options, selected, on_change)
            .width(Fill)
            .padding(8)
            .style(ui::styles::pick_list)
    ]
    .spacing(4)
    .into()
}

fn toggle_row<'a>(label: &'a str, is_active: bool, on_toggle: impl Fn(bool) -> Message + 'a) -> Element<'a, Message> {
    row![
        text(label).size(14).color(ui::styles::TEXT_MUTED),
        Space::new().width(Fill),
        toggler(is_active).on_toggle(on_toggle),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn adjustment_input<'a>(app: &'a App, prayer: Prayer) -> Element<'a, Message> {
    labeled_input(prayer.name(), app.adjustment_input(prayer), move |value| {
        Message::AdjustmentChanged(prayer, value)
    })
}

fn adjustment_grid(app: &App) -> Element<'_, Message> {
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

fn adjustment_summary(adjustments: PrayerAdjustments) -> String {
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

// View Logic & Assembly

fn view(app: &App) -> Element<'_, Message> {
    let base = main_view(app);

    if app.settings_open {
        let settings_layer = opaque(
            mouse_area(
                center(opaque(settings_modal(app))).style(|_| iced::widget::container::Style {
                    background: Some(ui::styles::MODAL_BACKDROP.into()),
                    ..Default::default()
                }),
            )
            .on_press(Message::ToggleSettings),
        );

        if app.adjustments_open {
            stack![
                base,
                settings_layer,
                opaque(
                    mouse_area(
                        center(opaque(adjustments_modal(app))).style(|_| iced::widget::container::Style {
                            background: Some(ui::styles::MODAL_BACKDROP.into()),
                            ..Default::default()
                        })
                    )
                    .on_press(Message::ToggleAdjustments)
                )
            ]
            .into()
        } else {
            stack![base, settings_layer].into()
        }
    } else {
        base
    }
}

fn main_view(app: &App) -> Element<'_, Message> {
    let offset = time::UtcOffset::from_whole_seconds((app.location.timezone_offset * 3600.0) as i32)
        .unwrap_or(time::UtcOffset::UTC);
    let now_local = app.now.to_offset(offset);

    let next_p = app
        .prayer_times
        .as_ref()
        .map(|times| next_prayer(times, now_local.time()));

    let top_bar = row![
        column![
            text(&app.location.name).size(18).color(ui::styles::TEXT_PRIMARY),
            if app.show_hijri {
                text(if app.show_arabic {
                    app.hijri_date.arabic_display()
                } else {
                    app.hijri_date.display()
                })
                .size(13)
                .color(ui::styles::ACCENT)
            } else {
                text(now_local.format(&*DATE_FMT).unwrap_or_default())
                    .size(13)
                    .color(ui::styles::TEXT_MUTED)
            }
        ]
        .spacing(2),
        Space::new().width(Fill),
        button(text("Menu").size(13))
            .on_press(Message::ToggleSettings)
            .style(ui::styles::button)
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
        .color(ui::styles::TEXT_MUTED),
        Space::new().width(Fill),
        text(now_local.format(&*DATE_FMT).unwrap_or_default())
            .size(12)
            .color(ui::styles::TEXT_MUTED),
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
        background: Some(ui::styles::BG.into()),
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
                    text("NEXT PRAYER").size(11).color(ui::styles::TEXT_MUTED),
                    text(prayer.name().to_uppercase())
                        .size(30)
                        .color(ui::styles::TEXT_PRIMARY),
                    text(format_time(ptime)).size(15).color(ui::styles::TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Fill),
                text(format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60))
                    .size(46)
                    .color(ui::styles::ACCENT)
                    .font(Font::MONOSPACE)
                    .wrapping(Wrapping::WordOrGlyph)
            ]
            .align_y(Alignment::Center)
        } else {
            row![text("No upcoming prayer").color(ui::styles::TEXT_MUTED)]
        }
    } else {
        row![text("Cannot compute").color(ui::styles::ERROR)]
    };

    container(content)
        .padding(24)
        .style(ui::styles::outline_card)
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
                    ui::styles::ACCENT
                } else {
                    ui::styles::TEXT_PRIMARY
                };

                let row_content = container(
                    row![
                        text(prayer.name()).size(15).color(text_col),
                        Space::new().width(Fill),
                        if app.show_arabic && prayer.name() != prayer.arabic_name() {
                            Element::from(text(prayer.arabic_name()).size(14).color(if is_next {
                                ui::styles::ACCENT
                            } else {
                                ui::styles::TEXT_MUTED
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
                            ui::styles::ACCENT_MUTED
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
                            color: ui::styles::BORDER,
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
            .style(ui::styles::surface_card)
            .width(Fill)
            .into()
    } else {
        text("Could not calculate prayer times.")
            .size(14)
            .color(ui::styles::TEXT_MUTED)
            .into()
    }
}

fn modal_header<'a>(title: &'a str, on_close: Message) -> Element<'a, Message> {
    row![
        text(title).size(18).color(ui::styles::TEXT_PRIMARY),
        Space::new().width(Fill),
        button(text("Close").size(13))
            .on_press(on_close)
            .style(ui::styles::button)
            .padding([4, 12]),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn settings_modal(app: &App) -> Element<'_, Message> {
    let header = modal_header("Configuration", Message::ToggleSettings);

    let location_grid = column![
        row![
            labeled_input("City Name", &app.inputs.loc_name_input, Message::LocationNameChanged),
            column![
                text("UTC Offset (Hrs)").size(11).color(ui::styles::TEXT_MUTED),
                text_input("UTC Offset (Hrs)", &app.inputs.tz_input)
                    .on_input(Message::TimezoneChanged)
                    .padding(8)
                    .style(ui::styles::text_input)
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
        text("Methodology").size(13).color(ui::styles::TEXT_PRIMARY),
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
        text("Minute Adjustments").size(13).color(ui::styles::TEXT_PRIMARY),
        row![
            text(if adjustment_text.is_empty() {
                "All 0".into()
            } else {
                adjustment_text
            })
            .size(12)
            .color(ui::styles::TEXT_MUTED)
            .wrapping(Wrapping::WordOrGlyph),
            Space::new().width(Fill),
            button(text("Edit").size(12))
                .on_press(Message::ToggleAdjustments)
                .style(ui::styles::button)
                .padding([6, 14])
        ]
        .spacing(12)
        .align_y(Alignment::Center)
    ]
    .spacing(8);

    let interface_group = column![
        text("Interface Options").size(13).color(ui::styles::TEXT_PRIMARY),
        toggle_row("Show Arabic Names", app.show_arabic, |_| Message::ToggleArabic),
        toggle_row("Show Hijri Date", app.show_hijri, |_| Message::ToggleHijri),
    ]
    .spacing(12);

    let location_section = column![
        text("Location Data").size(13).color(ui::styles::TEXT_PRIMARY),
        location_grid,
        button(text("Auto Detect Location").size(12))
            .on_press(Message::DetectLocation)
            .style(ui::styles::button)
            .padding([6, 14])
            .width(Fill),
    ]
    .spacing(12);

    let mut inner_content =
        column![header, location_section, calc_group, adjustment_group, interface_group].spacing(20);

    if let Some(err) = &app.error {
        inner_content = inner_content.push(text(err).size(13).color(ui::styles::ERROR));
    }

    container(scrollable(inner_content.padding(20)))
        .width(360)
        .style(ui::styles::surface_card)
        .into()
}

fn adjustments_modal(app: &App) -> Element<'_, Message> {
    let header = modal_header("Minute Adjustments", Message::ToggleAdjustments);

    container(scrollable(
        column![header, adjustment_grid(app)].spacing(20).padding(20),
    ))
    .width(360)
    .style(ui::styles::surface_card)
    .into()
}

// Application Setup

fn subscription(_app: &App) -> Subscription<Message> {
    let tick =
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick(time::OffsetDateTime::now_utc()));

    let keyboard_listen = iced::event::listen_with(|event, status, _id| {
        if status == iced::event::Status::Ignored
            && let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event
        {
            match key {
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab) => Some(if modifiers.shift() {
                    Message::FocusPrevious
                } else {
                    Message::FocusNext
                }),
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::EscapePressed),
                _ => None,
            }
        } else {
            None
        }
    });

    Subscription::batch(vec![tick, keyboard_listen])
}

fn theme(_app: &App) -> iced::Theme {
    iced::Theme::custom(
        String::from("Athan"),
        iced::theme::Palette {
            background: ui::styles::BG,
            text: ui::styles::TEXT_PRIMARY,
            primary: ui::styles::ACCENT,
            success: ui::styles::ACCENT,
            danger: ui::styles::ERROR,
            warning: ui::styles::ACCENT,
        },
    )
}

fn main() {
    if std::env::args().len() > 1 {
        if let Err(e) = athan::cli::run() {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    } else {
        launch_gui().expect("GUI error");
    }
}

fn launch_gui() -> iced::Result {
    iced::application(new, update, view)
        .subscription(subscription)
        .theme(theme)
        .window_size(iced::Size::new(450.0, 640.0))
        .centered()
        .run()
}
