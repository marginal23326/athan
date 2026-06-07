pub mod detect;
pub mod format;
pub mod hijri;
pub mod prayer;
pub mod qiblah;
pub mod types;

pub use detect::*;
pub use format::*;
pub use hijri::*;
pub use prayer::*;
pub use qiblah::*;
pub use types::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DailyPrayerData {
    pub date: time::Date,
    pub prayer_times: Option<PrayerTimes>,
    pub hijri_date: Option<HijriDate>,
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
    let effective_offset = location.effective_timezone_offset();

    DailyPrayerData {
        date,
        prayer_times: calculate_prayer_times(
            date,
            location.coordinates,
            effective_offset,
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
