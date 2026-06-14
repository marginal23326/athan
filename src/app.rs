use athan::core::*;
use iced::Task;
use std::sync::Arc;

pub type TrayRxHandle = Arc<std::sync::Mutex<Option<futures_channel::mpsc::UnboundedReceiver<crate::tray::TrayEvent>>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Location,
    Calculation,
    Adjustments,
    Audio,
    Interface,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(time::OffsetDateTime),
    ToggleStartOnBoot,
    MethodChanged(CalculationMethod),
    AsrMethodChanged(AsrMethod),
    LatitudeChanged(String),
    LongitudeChanged(String),
    TimezoneChanged(String),
    ElevationChanged(String),
    ToggleDst,
    ToggleSettings,
    ToggleArabic,
    #[cfg(feature = "hijri")]
    ToggleHijri,
    ToggleTimeFormat,
    SetSettingsTab(SettingsTab),
    LocationNameChanged(String),
    AdjustmentChanged(Prayer, String),
    FocusNext,
    FocusPrevious,
    EscapePressed,
    #[cfg(feature = "detect")]
    DetectLocation,
    #[cfg(feature = "detect")]
    LocationDetected(Result<LocationData, DetectError>),
    HideToTray(iced::window::Id),
    WindowClosed(iced::window::Id),
    TrayEvent(crate::tray::TrayEvent),
    PlayAdhan(Option<Prayer>),
    StopAdhan,
    VolumeChanged(f32),
    WindowMoved(i32, i32),
}

pub struct SettingsState {
    pub active_tab: SettingsTab,
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
    pub show_arabic: bool,
    pub use_24h: bool,
    #[cfg(feature = "hijri")]
    pub show_hijri: bool,
    #[cfg(feature = "detect")]
    pub is_detecting: bool,
    pub inputs: SettingsState,
    pub error: Option<String>,
    pub window_id: Option<iced::window::Id>,
    pub tray: Option<crate::tray::TrayHandle>,
    pub tray_rx: Option<TrayRxHandle>,
    pub start_on_boot: bool,
    pub volume: f32,
    pub last_prayer_announced: Option<(time::Date, Prayer)>,
    pub audio: Option<crate::audio::AudioPlayer>,
    pub last_tray_state: Option<(u8, bool, bool)>,
    pub local_offset: time::UtcOffset,
    pub adhan_path: std::path::PathBuf,
    pub fajr_path: std::path::PathBuf,
    pub window_pos: Option<(i32, i32)>,
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
            show_arabic: false,
            use_24h: false,
            #[cfg(feature = "hijri")]
            show_hijri: false,
            #[cfg(feature = "detect")]
            is_detecting: false,
            inputs: SettingsState {
                active_tab: SettingsTab::default(),
                lat_input: loc.coordinates.latitude.to_string(),
                lon_input: loc.coordinates.longitude.to_string(),
                tz_input: loc.timezone_offset.to_string(),
                elv_input: loc.elevation.to_string(),
                loc_name_input: loc.name.clone(),
                adjustment_inputs: Prayer::ALL.map(|prayer| adjustments.get(prayer).to_string()),
            },
            error: err,
            window_id: None,
            tray: None,
            tray_rx: None,
            start_on_boot: crate::config::is_autostart(),
            volume: 0.5,
            last_prayer_announced: None,
            audio: None,
            last_tray_state: None,
            local_offset: Self::compute_local_offset(&loc),
            adhan_path: crate::audio::audio_dir().join("adhan.ogg"),
            fajr_path: crate::audio::audio_dir().join("fajr.ogg"),
            window_pos: None,
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
        self.local_offset = Self::compute_local_offset(&self.location);
    }

    fn calculation_error(prayer_times: Option<PrayerTimes>) -> Option<String> {
        prayer_times
            .is_none()
            .then(|| "Cannot compute times for this date/location.".into())
    }

    pub fn set_adjustment_input(&mut self, prayer: Prayer, value: String) {
        self.inputs.adjustment_inputs[prayer.index()] = value;
    }

    pub fn main_window_settings(pos: Option<(i32, i32)>) -> iced::window::Settings {
        let icon = iced::window::icon::from_rgba(crate::tray::cached_icon_rgba_64().to_vec(), 64, 64).ok();

        iced::window::Settings {
            size: iced::Size::new(450.0, 640.0),
            position: pos.map_or(iced::window::Position::Centered, |(x, y)| {
                iced::window::Position::Specific(iced::Point::new(x as f32, y as f32))
            }),
            icon,
            ..Default::default()
        }
    }

    fn compute_local_offset(location: &Location) -> time::UtcOffset {
        time::UtcOffset::from_whole_seconds((location.effective_timezone_offset() * 3600.0) as i32)
            .unwrap_or(time::UtcOffset::UTC)
    }

    fn filter_numeric(s: &str, allow_decimal: bool) -> String {
        let mut result = String::with_capacity(s.len());
        let mut has_decimal = false;

        for c in s.chars() {
            if c.is_ascii_digit() {
                result.push(c);
            } else if c == '-' && result.is_empty() {
                result.push(c);
            } else if allow_decimal && c == '.' && !has_decimal {
                has_decimal = true;
                result.push(c);
            }
        }
        result
    }

    fn reset_inputs(&mut self) {
        self.inputs.lat_input = self.location.coordinates.latitude.to_string();
        self.inputs.lon_input = self.location.coordinates.longitude.to_string();
        self.inputs.tz_input = self.location.timezone_offset.to_string();
        self.inputs.elv_input = self.location.elevation.to_string();
        self.inputs.loc_name_input = self.location.name.clone();
        self.inputs.adjustment_inputs = Prayer::ALL.map(|prayer| self.prayer_adjustments.get(prayer).to_string());
    }

    pub fn apply_config(&mut self, config: crate::config::Config) {
        self.location = config.location;
        self.calculation_method = config.calculation_method;
        self.asr_method = config.asr_method;
        self.prayer_adjustments = config.prayer_adjustments;
        self.show_arabic = config.show_arabic;
        self.use_24h = config.use_24h;
        self.volume = config.volume;
        self.window_pos = config.window_pos;
        if let Some(audio) = &self.audio {
            audio.set_volume(self.volume);
        }
        #[cfg(feature = "hijri")]
        {
            self.show_hijri = config.show_hijri;
        }
        self.recalculate();
        self.reset_inputs();
    }

    pub fn to_config(&self) -> crate::config::Config {
        crate::config::Config {
            location: self.location.clone(),
            calculation_method: self.calculation_method,
            asr_method: self.asr_method,
            prayer_adjustments: self.prayer_adjustments,
            show_arabic: self.show_arabic,
            use_24h: self.use_24h,
            volume: self.volume,
            window_pos: self.window_pos,
            #[cfg(feature = "hijri")]
            show_hijri: self.show_hijri,
        }
    }

    pub fn save_config(&self) {
        crate::config::save(&self.to_config());
    }

    fn handle_tick(&mut self, now: time::OffsetDateTime) -> Task<Message> {
        let old_date = self.location.local_date(self.now);
        self.now = now;
        let current_date = self.location.local_date(self.now);

        if current_date != old_date {
            self.recalculate();
        }

        let mut out_task = Task::none();

        let mut playing = false;
        if let Some(audio) = &self.audio {
            if audio.is_playing() {
                playing = true;
            } else {
                self.audio = None;
            }
        }

        if let Some(times) = &self.prayer_times {
            let offset = self.local_offset;
            let now_local = self.now.to_offset(offset);

            let mut latest_passed = None;
            for &(prayer, ptime) in &times.as_array() {
                if ptime <= now_local.time() {
                    latest_passed = Some(prayer);
                }
            }

            if self.last_prayer_announced.is_none() {
                self.last_prayer_announced = Some(
                    latest_passed
                        .map(|p| (current_date, p))
                        .unwrap_or_else(|| (current_date.previous_day().unwrap_or(current_date), Prayer::Isha)),
                );
            } else if let Some(prayer) = latest_passed {
                let key = (current_date, prayer);
                if self.last_prayer_announced != Some(key) {
                    self.last_prayer_announced = Some(key);
                    if prayer != Prayer::Sunrise {
                        out_task = Task::done(Message::PlayAdhan(Some(prayer)));
                    }
                }
            }

            if let Some(tray) = &self.tray {
                let current_minute = now_local.time().minute();
                let is_open = self.window_id.is_some();

                let new_state = (current_minute, playing, is_open);
                if self.last_tray_state != Some(new_state) {
                    let tooltip = format_tray_tooltip(times, self.now, offset, self.use_24h);
                    tray.update(&tooltip, playing, is_open);
                    self.last_tray_state = Some(new_state);
                }
            }
        }

        out_task
    }
}

pub fn update(app: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::Tick(now) => return app.handle_tick(now),
        Message::PlayAdhan(prayer) => {
            if app.audio.is_none() {
                app.audio = crate::audio::AudioPlayer::new();
                if let Some(audio) = &app.audio {
                    audio.set_volume(app.volume);
                }
            }
            if let Some(audio) = &app.audio {
                let path = if prayer == Some(Prayer::Fajr) {
                    &app.fajr_path
                } else {
                    &app.adhan_path
                };
                audio.play(path);
            }
        }
        Message::StopAdhan => {
            app.audio = None;
        }
        Message::VolumeChanged(v) => {
            app.volume = v;
            if let Some(audio) = &app.audio {
                audio.set_volume(v);
            }
        }
        Message::WindowMoved(x, y) => {
            app.window_pos = Some((x, y));
        }
        Message::HideToTray(id) => {
            if app.window_id == Some(id) {
                return iced::window::close(id);
            }
        }
        Message::WindowClosed(id) => {
            if app.window_id == Some(id) {
                app.window_id = None;
                app.settings_open = false;
                app.reset_inputs();
                app.save_config();

                #[cfg(target_os = "windows")]
                {
                    unsafe extern "system" {
                        fn GetCurrentProcess() -> isize;
                        fn SetProcessWorkingSetSize(hProcess: isize, min: usize, max: usize) -> i32;
                    }
                    unsafe {
                        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
                    }
                }

                return Task::done(Message::Tick(time::OffsetDateTime::now_utc()));
            }
        }
        Message::TrayEvent(crate::tray::TrayEvent::ToggleWindow) => {
            if let Some(id) = app.window_id {
                return iced::window::close(id);
            } else {
                let now = app.now;
                let (id, task) = iced::window::open(App::main_window_settings(app.window_pos));
                app.window_id = Some(id);
                return task.map(move |_| Message::Tick(now));
            }
        }
        Message::TrayEvent(crate::tray::TrayEvent::Exit) => {
            app.save_config();
            return iced::exit();
        }
        Message::TrayEvent(crate::tray::TrayEvent::StopAdhan) => {
            app.audio = None;
        }
        Message::MethodChanged(m) => {
            app.calculation_method = m;
            app.recalculate();
            app.save_config();
        }
        Message::AsrMethodChanged(m) => {
            app.asr_method = m;
            app.recalculate();
            app.save_config();
        }
        Message::LatitudeChanged(s) => {
            app.is_detecting = false;
            let s = App::filter_numeric(&s, true);
            app.inputs.lat_input = s.clone();
            if let Ok(v) = s.parse::<f64>() {
                if (-90.0..=90.0).contains(&v) {
                    app.location.coordinates.latitude = v;
                    app.recalculate();
                }
            }
        }
        Message::LongitudeChanged(s) => {
            app.is_detecting = false;
            let s = App::filter_numeric(&s, true);
            app.inputs.lon_input = s.clone();
            if let Ok(v) = s.parse::<f64>() {
                if (-180.0..=180.0).contains(&v) {
                    app.location.coordinates.longitude = v;
                    app.recalculate();
                }
            }
        }
        Message::TimezoneChanged(s) => {
            app.is_detecting = false;
            let s = App::filter_numeric(&s, true);
            app.inputs.tz_input = s.clone();
            if let Ok(v) = s.parse::<f64>() {
                if (-14.0..=14.0).contains(&v) {
                    app.location.timezone_offset = v;
                    app.recalculate();
                }
            }
        }
        Message::ElevationChanged(s) => {
            app.is_detecting = false;
            let s = App::filter_numeric(&s, true);
            app.inputs.elv_input = s.clone();
            if let Ok(v) = s.parse::<f64>() {
                if (0.0..=9000.0).contains(&v) {
                    app.location.elevation = v;
                    app.recalculate();
                }
            }
        }
        Message::LocationNameChanged(s) => {
            app.is_detecting = false;
            app.inputs.loc_name_input = s.clone();
            app.location.name = s;
        }
        Message::AdjustmentChanged(prayer, s) => {
            let s = App::filter_numeric(&s, false);
            app.set_adjustment_input(prayer, s.clone());
            if let Ok(minutes) = s.parse::<i32>()
                && (-120..=120).contains(&minutes)
            {
                app.prayer_adjustments.set(prayer, minutes);
                app.recalculate();
            }
        }
        #[cfg(feature = "detect")]
        Message::DetectLocation => {
            app.is_detecting = true;
            let (tx, rx) = futures_channel::oneshot::channel();

            std::thread::spawn(move || {
                let _ = tx.send(detect_location());
            });

            return Task::perform(
                async move {
                    rx.await
                        .unwrap_or_else(|_| Err(DetectError::Internal("Thread aborted".into())))
                },
                Message::LocationDetected,
            );
        }
        #[cfg(feature = "detect")]
        Message::LocationDetected(res) => {
            if !app.is_detecting {
                return Task::none();
            }
            app.is_detecting = false;
            match res {
                Ok(data) => {
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
                    app.save_config();
                }
                Err(e) => {
                    app.error = Some(e.to_string());
                }
            }
        }
        Message::ToggleDst => {
            app.location.dst = !app.location.dst;
            app.recalculate();
            app.save_config();
        }
        Message::SetSettingsTab(tab) => {
            app.inputs.active_tab = tab;
        }
        Message::ToggleSettings => {
            app.settings_open = !app.settings_open;
            if !app.settings_open {
                app.reset_inputs();
                app.save_config();
            }
        }
        Message::ToggleStartOnBoot => {
            app.start_on_boot = !app.start_on_boot;
            crate::config::set_autostart(app.start_on_boot);
        }
        Message::ToggleArabic => {
            app.show_arabic = !app.show_arabic;
            app.save_config();
        }
        #[cfg(feature = "hijri")]
        Message::ToggleHijri => {
            app.show_hijri = !app.show_hijri;
            app.save_config();
        }
        Message::ToggleTimeFormat => {
            app.use_24h = !app.use_24h;
            app.save_config();
        }
        Message::FocusNext => return iced::widget::operation::focus_next(),
        Message::FocusPrevious => return iced::widget::operation::focus_previous(),
        Message::EscapePressed => {
            if app.settings_open {
                app.settings_open = false;
                app.reset_inputs();
                app.save_config();
            } else if let Some(id) = app.window_id {
                return Task::done(Message::HideToTray(id));
            }
        }
    }

    Task::none()
}

fn format_tray_tooltip(
    prayer_times: &PrayerTimes,
    now: time::OffsetDateTime,
    offset: time::UtcOffset,
    use_24h: bool,
) -> String {
    let now_local = now.to_offset(offset);
    let (prayer, ptime) = next_prayer(prayer_times, now_local.time());
    let remaining = time_until(ptime, now_local.time());

    let total_minutes = remaining.whole_minutes();
    let hours = total_minutes / 60;
    let mins = total_minutes % 60;

    if hours > 0 {
        format!(
            "{} at {} — in {}h {}m",
            prayer.name(),
            format_time(ptime, use_24h),
            hours,
            mins
        )
    } else {
        format!("{} at {} — in {}m", prayer.name(), format_time(ptime, use_24h), mins)
    }
}
