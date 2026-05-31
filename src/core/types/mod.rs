use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalculationMethod {
    Mwl,
    Egypt,
    Karachi,
    UmmAlQura,
    Isna,
    Custom { fajr_angle: f64, isha_angle: f64 },
    CustomMinutes { fajr_angle: f64, isha_minutes: f64 },
}

impl CalculationMethod {
    pub fn description(&self) -> &str {
        match self {
            Self::Mwl => "Muslim World League",
            Self::Egypt => "Egyptian General Authority for Surveying",
            Self::Karachi => "University of Islamic Sciences, Karachi",
            Self::Isna => "ISNA (North America)",
            Self::UmmAlQura => "Umm Al-Qura",
            Self::Custom { .. } => "Custom angles",
            Self::CustomMinutes { .. } => "Custom minutes after Maghrib",
        }
    }

    pub fn prayer_params(&self) -> (f64, f64, f64) {
        match self {
            Self::Mwl => (18.0, 17.0, 0.0),
            Self::Egypt => (19.5, 17.5, 0.0),
            Self::Karachi => (18.0, 18.0, 0.0),
            Self::UmmAlQura => (18.5, 0.0, 1.5),
            Self::Isna => (15.0, 15.0, 0.0),
            Self::Custom { fajr_angle, isha_angle } => (*fajr_angle, *isha_angle, 0.0),
            Self::CustomMinutes {
                fajr_angle,
                isha_minutes,
            } => (*fajr_angle, 0.0, *isha_minutes / 60.0),
        }
    }

    pub fn variants() -> &'static [CalculationMethod] {
        &[Self::Mwl, Self::Egypt, Self::Karachi, Self::UmmAlQura, Self::Isna]
    }
}

impl fmt::Display for CalculationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsrMethod {
    Shafi,
    Hanafi,
}

impl AsrMethod {
    pub fn shadow_ratio(&self) -> f64 {
        match self {
            Self::Shafi => 1.0,
            Self::Hanafi => 2.0,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Shafi => "Shafi'i / Hanbali / Maliki",
            Self::Hanafi => "Hanafi",
        }
    }

    pub fn variants() -> &'static [AsrMethod] {
        &[Self::Shafi, Self::Hanafi]
    }
}

impl fmt::Display for AsrMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrayerTimes {
    pub fajr: time::Time,
    pub sunrise: time::Time,
    pub dhuhr: time::Time,
    pub asr: time::Time,
    pub maghrib: time::Time,
    pub isha: time::Time,
}

impl PrayerTimes {
    pub fn as_array(&self) -> [(Prayer, time::Time); 6] {
        [
            (Prayer::Fajr, self.fajr),
            (Prayer::Sunrise, self.sunrise),
            (Prayer::Dhuhr, self.dhuhr),
            (Prayer::Asr, self.asr),
            (Prayer::Maghrib, self.maghrib),
            (Prayer::Isha, self.isha),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prayer {
    Fajr,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
}

impl Prayer {
    pub const ALL: [Prayer; 6] = [
        Self::Fajr,
        Self::Sunrise,
        Self::Dhuhr,
        Self::Asr,
        Self::Maghrib,
        Self::Isha,
    ];

    pub const COUNT: usize = Self::ALL.len();

    pub fn index(self) -> usize {
        match self {
            Self::Fajr => 0,
            Self::Sunrise => 1,
            Self::Dhuhr => 2,
            Self::Asr => 3,
            Self::Maghrib => 4,
            Self::Isha => 5,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Fajr => "Fajr",
            Self::Sunrise => "Sunrise",
            Self::Dhuhr => "Dhuhr",
            Self::Asr => "Asr",
            Self::Maghrib => "Maghrib",
            Self::Isha => "Isha",
        }
    }

    pub fn arabic_name(&self) -> &'static str {
        match self {
            Self::Fajr => "الفجر",
            Self::Sunrise => "الشروق",
            Self::Dhuhr => "الظهر",
            Self::Asr => "العصر",
            Self::Maghrib => "المغرب",
            Self::Isha => "العشاء",
        }
    }

    pub fn all() -> &'static [Prayer] {
        &Self::ALL
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrayerAdjustments {
    minutes: [i32; Prayer::COUNT],
}

impl PrayerAdjustments {
    pub const fn new(minutes: [i32; Prayer::COUNT]) -> Self {
        Self { minutes }
    }

    pub const fn zero() -> Self {
        Self::new([0; Prayer::COUNT])
    }

    pub const fn prayer_start_safety() -> Self {
        Self::new([0, 0, 1, 1, 1, 1])
    }

    pub fn get(&self, prayer: Prayer) -> i32 {
        self.minutes[prayer.index()]
    }

    pub fn set(&mut self, prayer: Prayer, minutes: i32) {
        self.minutes[prayer.index()] = minutes;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self { latitude, longitude }
    }

    pub fn is_valid(&self) -> bool {
        self.latitude.is_finite()
            && self.longitude.is_finite()
            && (-90.0..=90.0).contains(&self.latitude)
            && (-180.0..=180.0).contains(&self.longitude)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub name: String,
    pub coordinates: Coordinates,
    pub timezone_offset: f64,
    pub elevation: f64,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            name: "Makkah".into(),
            coordinates: Coordinates::new(21.422_487, 39.826_206),
            timezone_offset: 3.0,
            elevation: 0.0,
        }
    }
}

impl Location {
    pub fn local_date(&self, now: time::OffsetDateTime) -> time::Date {
        if self.timezone_offset.is_finite() && (-24.0..=24.0).contains(&self.timezone_offset) {
            let seconds = (self.timezone_offset * 3600.0).round() as i64;
            (now + time::Duration::seconds(seconds)).date()
        } else {
            now.date()
        }
    }
}

#[cfg(test)]
mod tests;
