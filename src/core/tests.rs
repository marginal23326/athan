use super::*;

fn utc_datetime(year: i32, month: time::Month, day: u8, hour: u8, minute: u8) -> time::OffsetDateTime {
    time::OffsetDateTime::new_utc(
        time::Date::from_calendar_date(year, month, day).unwrap(),
        time::Time::from_hms(hour, minute, 0).unwrap(),
    )
}

#[test]
fn daily_prayer_data_uses_location_date_ahead_of_utc() {
    let location = Location {
        name: "Dhaka".into(),
        coordinates: Coordinates::new(23.757283, 90.369712),
        timezone_offset: 6.0,
    };
    let now = utc_datetime(2026, time::Month::May, 22, 19, 0);
    let expected_date = time::Date::from_calendar_date(2026, time::Month::May, 23).unwrap();

    let data = calculate_daily_prayer_data(
        now,
        &location,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    );

    assert_eq!(data.date, expected_date);
    assert_eq!(data.hijri_date, HijriDate::from_gregorian(expected_date));
    assert_eq!(
        data.prayer_times,
        calculate_prayer_times(
            expected_date,
            location.coordinates,
            location.timezone_offset,
            CalculationMethod::UmmAlQura,
            AsrMethod::Shafi,
            PrayerAdjustments::zero(),
        )
    );
}

#[test]
fn daily_prayer_data_uses_location_date_behind_utc() {
    let location = Location {
        name: "New York".into(),
        coordinates: Coordinates::new(40.7128, -74.0060),
        timezone_offset: -4.0,
    };
    let now = utc_datetime(2026, time::Month::May, 23, 2, 0);
    let expected_date = time::Date::from_calendar_date(2026, time::Month::May, 22).unwrap();

    let data = calculate_daily_prayer_data(
        now,
        &location,
        CalculationMethod::Isna,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    );

    assert_eq!(data.date, expected_date);
    assert_eq!(data.hijri_date, HijriDate::from_gregorian(expected_date));
    assert_eq!(
        data.prayer_times,
        calculate_prayer_times(
            expected_date,
            location.coordinates,
            location.timezone_offset,
            CalculationMethod::Isna,
            AsrMethod::Shafi,
            PrayerAdjustments::zero(),
        )
    );
}
