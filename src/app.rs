use athan::core::*;
use iced::Task;

#[derive(Debug, Clone)]
pub enum Message {
    Tick(time::OffsetDateTime),
    MethodChanged(CalculationMethod),
    AsrMethodChanged(AsrMethod),
    LatitudeChanged(String),
    LongitudeChanged(String),
    TimezoneChanged(String),
    ElevationChanged(String),
    ToggleDst,
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
    LocationDetected(Result<LocationData, DetectError>),
}

pub struct SettingsState {
    pub lat_input: String,
    pub lon_input: String,
    pub tz_input: String,
    pub elv_input: String,
    pub loc_name_input: String,
    pub adjustment_inputs: [String; Prayer::COUNT],
}

pub struct App {
    pub location: Location,
    pub calculation_method: CalculationMethod,
    pub asr_method: AsrMethod,
    pub prayer_adjustments: PrayerAdjustments,
    pub prayer_times: Option<PrayerTimes>,
    pub hijri_date: Option<HijriDate>,
    pub qiblah: f64,
    pub now: time::OffsetDateTime,
    pub settings_open: bool,
    pub adjustments_open: bool,
    pub show_arabic: bool,
    pub show_hijri: bool,
    pub is_detecting: bool,
    pub inputs: SettingsState,
    pub error: Option<String>,
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
            is_detecting: false,
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
    pub fn recalculate(&mut self) {
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

    pub fn set_adjustment_input(&mut self, prayer: Prayer, value: String) {
        self.inputs.adjustment_inputs[prayer.index()] = value;
    }

    fn reset_inputs(&mut self) {
        self.inputs.lat_input = self.location.coordinates.latitude.to_string();
        self.inputs.lon_input = self.location.coordinates.longitude.to_string();
        self.inputs.tz_input = self.location.timezone_offset.to_string();
        self.inputs.elv_input = self.location.elevation.to_string();
        self.inputs.loc_name_input = self.location.name.clone();
        self.inputs.adjustment_inputs = Prayer::ALL.map(|prayer| self.prayer_adjustments.get(prayer).to_string());
    }
}

pub fn new() -> App {
    App::default()
}

pub fn update(app: &mut App, msg: Message) -> Task<Message> {
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
            app.is_detecting = true;

            return Task::perform(
                async {
                    tokio::task::spawn_blocking(detect_location)
                        .await
                        .map_err(|e| DetectError::Internal(format!("Task failed: {e}")))
                        .and_then(|res| res)
                },
                Message::LocationDetected,
            );
        }
        Message::LocationDetected(Ok(data)) => {
            app.is_detecting = false;
            app.inputs.loc_name_input = data.name.clone();
            app.inputs.lat_input = data.lat.to_string();
            app.inputs.lon_input = data.lon.to_string();
            app.inputs.tz_input = format!("{:.1}", data.offset);
            app.inputs.elv_input = data.elevation.to_string();
            app.location.name = data.name;
            app.location.coordinates.latitude = data.lat;
            app.location.coordinates.longitude = data.lon;
            app.location.timezone_offset = data.offset;
            app.location.dst = false;
            app.location.elevation = data.elevation;
            app.recalculate();
        }
        Message::LocationDetected(Err(e)) => {
            app.is_detecting = false;
            app.error = Some(e.to_string());
        }
        Message::ToggleDst => {
            app.location.dst = !app.location.dst;
            app.recalculate();
        }
        Message::ToggleSettings => {
            app.settings_open = !app.settings_open;
            if !app.settings_open {
                app.adjustments_open = false;
                app.reset_inputs();
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
                app.reset_inputs();
            }
        }
    }
    Task::none()
}
