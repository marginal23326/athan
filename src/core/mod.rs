pub mod detect;
pub mod hijri;
pub mod prayer;
pub mod qiblah;
pub mod types;

pub use detect::*;
pub use hijri::*;
pub use prayer::*;
pub use qiblah::*;
pub use types::*;

use std::sync::LazyLock;

pub static DATE_FMT: LazyLock<time::format_description::OwnedFormatItem> = LazyLock::new(|| {
    time::format_description::parse_owned::<1>("[weekday], [month repr:long] [day], [year]").unwrap()
});

pub fn format_duration(d: time::Duration) -> String {
    let secs = d.whole_seconds().max(0);
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct DailyPrayerData {
    pub date: time::Date,
    pub prayer_times: Option<PrayerTimes>,
    pub hijri_date: HijriDate,
    pub qiblah: f64,
}

pub fn calculate_daily_prayer_data(
    now: time::OffsetDateTime,
    location: &Location,
    method: CalculationMethod,
    asr_method: AsrMethod,
    adjustments: PrayerAdjustments,
) -> DailyPrayerData {
    let date = location.local_date(now);

    DailyPrayerData {
        date,
        prayer_times: calculate_prayer_times(
            date,
            location.coordinates,
            location.timezone_offset,
            location.elevation,
            method,
            asr_method,
            adjustments,
        ),
        hijri_date: HijriDate::from_gregorian(date),
        qiblah: qiblah_direction(location.coordinates),
    }
}

#[cfg(test)]
mod tests;
