use super::*;
use super::spa::{SpaFunction, SpaInputs, spa_calculate};

fn dhaka_may_24_2026_times(adjustments: PrayerAdjustments) -> PrayerTimes {
    calculate_prayer_times(
        time::Date::from_calendar_date(2026, time::Month::May, 24).unwrap(),
        Coordinates::new(23.757283, 90.369712),
        6.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        adjustments,
    )
    .unwrap()
}

fn hm(t: &time::Time) -> (u8, u8) {
    let (h, m, _) = t.as_hms();
    (h, m)
}

fn assert_times_hm(times: &PrayerTimes, expected: [(u8, u8); Prayer::COUNT]) {
    for ((prayer, time), expected) in times.as_array().into_iter().zip(expected) {
        assert_eq!(hm(&time), expected, "{}", prayer.name());
    }
}

#[test]
fn fajr_angle_sunrise_check() {
    let date = time::Date::from_calendar_date(2026, time::Month::May, 24).unwrap();
    let coords = Coordinates::new(23.757283, 90.369712);
    let times = dhaka_may_24_2026_times(PrayerAdjustments::zero());

    assert_times_hm(&times, [(3, 45), (5, 13), (11, 55), (15, 17), (18, 38), (20, 8)]);

    // Maghrib + 90min should be Isha (Umm al-Qura rule), within 2 min
    let maghrib_mins = times.maghrib.as_hms().0 as i64 * 60 + times.maghrib.as_hms().1 as i64;
    let isha_mins = times.isha.as_hms().0 as i64 * 60 + times.isha.as_hms().1 as i64;
    assert!((isha_mins - maghrib_mins - 90).abs() <= 2);

    // Dhuhr should be raw solar noon when no adjustments are configured.
    let tz = 6.0;
    let spa_inputs = SpaInputs::new(date.year(), date.month() as u8 as i32, date.day() as u8 as i32, coords.latitude, coords.longitude)
        .timezone(tz)
        .function(SpaFunction::ZaRts);
    let spa_outputs = spa_calculate(&spa_inputs).unwrap();
    let transit = spa_outputs.suntransit.unwrap();
    let dhuhr_hr = times.dhuhr.as_hms().0 as f64 + times.dhuhr.as_hms().1 as f64 / 60.0;
    assert!((dhuhr_hr - transit).abs() < 0.03);
}

#[test]
fn decimal_hours_rounding_wraps_midnight_correctly() {
    assert_eq!(dec_hours_to_time(24.0), time::Time::MIDNIGHT);
    assert_eq!(dec_hours_to_time(-0.0001), time::Time::MIDNIGHT);
    assert_eq!(dec_hours_to_time(23.9999), time::Time::MIDNIGHT);
    assert_eq!(dec_hours_to_time(25.5), time::Time::from_hms(1, 30, 0).unwrap());
    assert_eq!(dec_hours_to_time(5.2121), time::Time::from_hms(5, 13, 0).unwrap());
}

#[test]
fn next_prayer_advances_strictly_after_current_time() {
    let times = PrayerTimes {
        fajr: time::Time::from_hms(5, 0, 0).unwrap(),
        sunrise: time::Time::from_hms(6, 0, 0).unwrap(),
        dhuhr: time::Time::from_hms(12, 0, 0).unwrap(),
        asr: time::Time::from_hms(15, 0, 0).unwrap(),
        maghrib: time::Time::from_hms(18, 0, 0).unwrap(),
        isha: time::Time::from_hms(20, 0, 0).unwrap(),
    };

    assert_eq!(
        next_prayer(&times, time::Time::from_hms(5, 0, 0).unwrap()),
        (Prayer::Sunrise, time::Time::from_hms(6, 0, 0).unwrap())
    );
    assert_eq!(
        next_prayer(&times, time::Time::from_hms(23, 59, 59).unwrap()),
        (Prayer::Fajr, time::Time::from_hms(5, 0, 0).unwrap())
    );
}

#[test]
fn time_until_handles_same_day_and_next_day_targets() {
    assert_eq!(
        time_until(
            time::Time::from_hms(10, 0, 0).unwrap(),
            time::Time::from_hms(9, 30, 0).unwrap()
        )
        .whole_minutes(),
        30
    );
    assert_eq!(
        time_until(
            time::Time::from_hms(0, 15, 0).unwrap(),
            time::Time::from_hms(23, 45, 0).unwrap()
        )
        .whole_minutes(),
        30
    );
    assert_eq!(
        time_until(
            time::Time::from_hms(5, 0, 0).unwrap(),
            time::Time::from_hms(5, 0, 0).unwrap()
        )
        .whole_hours(),
        24
    );
}

#[test]
fn invalid_inputs_return_none_instead_of_silent_nonsense() {
    let date = time::Date::from_calendar_date(2026, time::Month::May, 24).unwrap();
    let valid = Coordinates::new(23.757283, 90.369712);

    assert!(
        calculate_prayer_times(
            date,
            Coordinates::new(91.0, 90.369712),
            6.0,
            CalculationMethod::UmmAlQura,
            AsrMethod::Shafi,
            PrayerAdjustments::zero(),
        )
        .is_none()
    );
    assert!(
        calculate_prayer_times(
            date,
            Coordinates::new(23.757283, f64::NAN),
            6.0,
            CalculationMethod::UmmAlQura,
            AsrMethod::Shafi,
            PrayerAdjustments::zero(),
        )
        .is_none()
    );
    assert!(
        calculate_prayer_times(
            date,
            valid,
            f64::INFINITY,
            CalculationMethod::UmmAlQura,
            AsrMethod::Shafi,
            PrayerAdjustments::zero(),
        )
        .is_none()
    );
    assert!(
        calculate_prayer_times(
            date,
            valid,
            6.0,
            CalculationMethod::Custom {
                fajr_angle: 0.0,
                isha_angle: 17.0,
            },
            AsrMethod::Shafi,
            PrayerAdjustments::zero(),
        )
        .is_none()
    );
}

#[test]
fn custom_minutes_isha_is_fixed_after_maghrib() {
    let date = time::Date::from_calendar_date(2026, time::Month::May, 24).unwrap();
    let times = calculate_prayer_times(
        date,
        Coordinates::new(23.757283, 90.369712),
        6.0,
        CalculationMethod::CustomMinutes {
            fajr_angle: 18.5,
            isha_minutes: 120.0,
        },
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();

    let diff = time_to_secs(times.isha) - time_to_secs(times.maghrib);
    assert!((diff - 7_200).abs() <= 1, "diff was {diff}");
}

#[test]
fn prayer_adjustments_apply_after_raw_calculation() {
    let date = time::Date::from_calendar_date(2026, time::Month::May, 24).unwrap();
    let coords = Coordinates::new(23.757283, 90.369712);
    let base = calculate_prayer_times(
        date,
        coords,
        6.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();
    let adjusted = calculate_prayer_times(
        date,
        coords,
        6.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        PrayerAdjustments::new([-2, 0, 1, 3, 1, -1]),
    )
    .unwrap();

    assert_eq!(time_to_secs(adjusted.fajr), time_to_secs(base.fajr) - 120);
    assert_eq!(time_to_secs(adjusted.sunrise), time_to_secs(base.sunrise));
    assert_eq!(time_to_secs(adjusted.dhuhr), time_to_secs(base.dhuhr) + 60);
    assert_eq!(time_to_secs(adjusted.asr), time_to_secs(base.asr) + 180);
    assert_eq!(time_to_secs(adjusted.maghrib), time_to_secs(base.maghrib) + 60);
    assert_eq!(time_to_secs(adjusted.isha), time_to_secs(base.isha) - 60);
}

#[test]
fn umm_al_qura_isha_uses_120_minutes_during_ramadan() {
    let date = time::Date::from_calendar_date(2026, time::Month::February, 18).unwrap();
    let times = calculate_prayer_times(
        date,
        Coordinates::new(21.422_487, 39.826_206),
        3.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();

    let diff = time_to_secs(times.isha) - time_to_secs(times.maghrib);
    assert!((diff - 7_200).abs() <= 1, "diff was {diff}");
}

#[test]
fn hanafi_asr_is_later_than_shafi_asr() {
    let date = time::Date::from_calendar_date(2026, time::Month::May, 24).unwrap();
    let coords = Coordinates::new(23.757283, 90.369712);

    let shafi = calculate_prayer_times(
        date,
        coords,
        6.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();
    let hanafi = calculate_prayer_times(
        date,
        coords,
        6.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Hanafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();

    assert!(time_to_secs(hanafi.asr) > time_to_secs(shafi.asr));
}

#[test]
fn high_latitude_dates_still_return_finite_times() {
    let date = time::Date::from_calendar_date(2026, time::Month::June, 21).unwrap();
    let times = calculate_prayer_times(
        date,
        Coordinates::new(69.6492, 18.9553),
        2.0,
        CalculationMethod::Mwl,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();

    for (_, time) in times.as_array() {
        assert!((0..86_400).contains(&time_to_secs(time)));
    }
}
