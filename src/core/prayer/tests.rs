use super::spa::{SpaFunction, SpaInputs, spa_calculate};
use super::*;

fn make_date(y: i32, m: time::Month, d: u8) -> time::Date {
    time::Date::from_calendar_date(y, m, d).unwrap()
}

fn hm_to_min(h: u8, m: u8) -> i64 {
    h as i64 * 60 + m as i64
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

fn assert_times_ordered(times: &PrayerTimes, label: &str) {
    let secs: Vec<i64> = times.as_array().iter().map(|(_, t)| time_to_secs(*t)).collect();
    assert!(
        secs.windows(2).all(|w| w[0] < w[1]),
        "{label}: order violated: {secs:?}"
    );
}

fn assert_all_times_valid(times: &PrayerTimes, label: &str) {
    for (prayer, t) in times.as_array() {
        let s = time_to_secs(t);
        assert!(
            (0..86400).contains(&s),
            "{label}: {} seconds={s} out of range",
            prayer.name(),
        );
    }
}

fn calc_times(date: time::Date, coords: Coordinates, tz: f64, method: CalculationMethod) -> PrayerTimes {
    calculate_prayer_times(
        date,
        coords,
        tz,
        0.0,
        method,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap_or_else(|| panic!("{method:?}: calculation returned None"))
}

fn dhaka_may_24_2026_times(adjustments: PrayerAdjustments) -> PrayerTimes {
    calculate_prayer_times(
        time::Date::from_calendar_date(2026, time::Month::May, 24).unwrap(),
        Coordinates::new(23.757283, 90.369712),
        6.0,
        0.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        adjustments,
    )
    .unwrap()
}

// API Cross-Verification Tests

struct ApiRef {
    name: &'static str,
    date: time::Date,
    coords: Coordinates,
    tz: f64,
    method: CalculationMethod,
    expected: [(u8, u8); 6],
    tolerance_minutes: i64,
}

fn run_api_ref_tests(refs: &[ApiRef]) {
    for ref_data in refs {
        let times = calc_times(ref_data.date, ref_data.coords, ref_data.tz, ref_data.method);

        let array = times.as_array();
        let mut failures = Vec::new();

        for (prayer_ref, expected) in array.iter().zip(ref_data.expected.iter()) {
            let expected_mins = hm_to_min(expected.0, expected.1);
            let actual_mins = hm_to_min(hm(&prayer_ref.1).0, hm(&prayer_ref.1).1);
            if (actual_mins - expected_mins).abs() > ref_data.tolerance_minutes {
                failures.push((prayer_ref.0, actual_mins, expected_mins));
            }
        }

        if !failures.is_empty() {
            for (prayer, actual, expected) in &failures {
                eprintln!(
                    "  {}: got {:02}:{:02}, expected {:02}:{:02} (diff={}min)",
                    prayer.name(),
                    actual / 60,
                    actual % 60,
                    expected / 60,
                    expected % 60,
                    (actual - expected).abs()
                );
            }
            panic!("{}: {} timings outside tolerance", ref_data.name, failures.len());
        }
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
    let spa_inputs = SpaInputs::new(
        date.year(),
        date.month() as u8 as i32,
        date.day() as i32,
        coords.latitude,
        coords.longitude,
    )
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
            0.0,
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
            0.0,
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
            0.0,
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
            0.0,
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
        0.0,
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
        0.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();
    let adjusted = calculate_prayer_times(
        date,
        coords,
        6.0,
        0.0,
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
        0.0,
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
        0.0,
        CalculationMethod::UmmAlQura,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();
    let hanafi = calculate_prayer_times(
        date,
        coords,
        6.0,
        0.0,
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
        0.0,
        CalculationMethod::Mwl,
        AsrMethod::Shafi,
        PrayerAdjustments::zero(),
    )
    .unwrap();

    for (_, time) in times.as_array() {
        assert!((0..86_400).contains(&time_to_secs(time)));
    }
}

#[test]
fn api_verification_all_locations() {
    run_api_ref_tests(&[
        // Dhaka: UmmAlQura/Shafi
        ApiRef {
            name: "Dhaka",
            date: make_date(2026, time::Month::May, 24),
            coords: Coordinates::new(23.757283, 90.369712),
            tz: 6.0,
            method: CalculationMethod::UmmAlQura,
            expected: [(3, 45), (5, 13), (11, 55), (15, 16), (18, 38), (20, 8)],
            tolerance_minutes: 2,
        },
        // London: ISNA/Shafi
        ApiRef {
            name: "London",
            date: make_date(2026, time::Month::May, 24),
            coords: Coordinates::new(51.5074, -0.1278),
            tz: 1.0,
            method: CalculationMethod::Isna,
            expected: [(2, 57), (4, 57), (12, 57), (17, 14), (20, 59), (22, 58)],
            tolerance_minutes: 2,
        },
        // New York: Egypt/Shafi
        ApiRef {
            name: "NewYork",
            date: make_date(2026, time::Month::May, 24),
            coords: Coordinates::new(40.7128, -74.0060),
            tz: -4.0,
            method: CalculationMethod::Egypt,
            expected: [(3, 21), (5, 32), (12, 53), (16, 50), (20, 15), (22, 8)],
            tolerance_minutes: 2,
        },
        // Makkah: UmmAlQura/Shafi
        ApiRef {
            name: "Makkah",
            date: make_date(2026, time::Month::May, 24),
            coords: Coordinates::new(21.422487, 39.826206),
            tz: 3.0,
            method: CalculationMethod::UmmAlQura,
            expected: [(4, 14), (5, 39), (12, 18), (15, 32), (18, 56), (20, 26)],
            tolerance_minutes: 2,
        },
        // Makkah Ramadan: UmmAlQura uses 120min after Maghrib for Isha
        ApiRef {
            name: "Makkah Ramadan",
            date: make_date(2026, time::Month::February, 18),
            coords: Coordinates::new(21.422487, 39.826206),
            tz: 3.0,
            method: CalculationMethod::UmmAlQura,
            expected: [(5, 33), (6, 50), (12, 35), (15, 52), (18, 20), (20, 20)],
            tolerance_minutes: 2,
        },
        // Sydney: Karachi/Shafi
        ApiRef {
            name: "Sydney",
            date: make_date(2026, time::Month::May, 24),
            coords: Coordinates::new(-33.8688, 151.2093),
            tz: 10.0,
            method: CalculationMethod::Karachi,
            expected: [(5, 19), (6, 46), (11, 52), (14, 39), (16, 57), (18, 25)],
            tolerance_minutes: 2,
        },
    ]);
}

#[test]
fn london_high_lat_winter_vs_aladhan() {
    // Regression: previously angle/60 was always applied above 48.5°,
    // giving Fajr=04:01, Isha=19:56 in winter. Should be 06:20, 17:38.
    let dates = [
        (make_date(2026, time::Month::December, 21), 0.0, (6, 20), (17, 38)),
        (make_date(2026, time::Month::June, 21), 1.0, (2, 53), (23, 12)),
    ];
    let coords = Coordinates::new(51.5074, -0.1278);
    for (date, tz, (efh, efm), (eih, eim)) in dates {
        let times = calc_times(date, coords, tz, CalculationMethod::Isna);
        let (fh, fm) = hm(&times.fajr);
        let (ih, im) = hm(&times.isha);
        assert!(
            ((fh as i32 * 60 + fm as i32) - (efh * 60 + efm)).abs() <= 5,
            "Fajr: got {fh:02}:{fm:02}, expected {efh:02}:{efm:02}"
        );
        assert!(
            ((ih as i32 * 60 + im as i32) - (eih * 60 + eim)).abs() <= 5,
            "Isha: got {ih:02}:{im:02}, expected {eih:02}:{eim:02}"
        );
    }
}

// Edge Cases

#[test]
fn equator_times_are_symmetric() {
    let times = calc_times(
        make_date(2026, time::Month::March, 20),
        Coordinates::new(0.0, 0.0),
        0.0,
        CalculationMethod::Mwl,
    );

    let (sh, sm) = hm(&times.sunrise);
    let (mh, mm) = hm(&times.maghrib);
    let sunrise_mins = sh as i64 * 60 + sm as i64;
    let maghrib_mins = mh as i64 * 60 + mm as i64;

    assert!((sunrise_mins - 360).abs() <= 15, "Sunrise at equator: {sh:02}:{sm:02}");
    assert!(
        ((maghrib_mins - sunrise_mins) - 720).abs() <= 60,
        "Day length: {}min",
        maghrib_mins - sunrise_mins
    );
}

#[test]
fn non_integer_timezone_kathmandu() {
    let times = calc_times(
        make_date(2026, time::Month::May, 24),
        Coordinates::new(27.7172, 85.3240),
        5.75,
        CalculationMethod::Karachi,
    );
    assert_times_ordered(&times, "Kathmandu UTC+5:45");
}

#[test]
fn negative_timezone_west_hemisphere() {
    let times = calc_times(
        make_date(2026, time::Month::May, 24),
        Coordinates::new(34.0522, -118.2437),
        -7.0,
        CalculationMethod::Isna,
    );
    assert_times_ordered(&times, "LA UTC-7");
}

#[test]
fn near_date_line_works() {
    let times = calc_times(
        make_date(2026, time::Month::May, 24),
        Coordinates::new(-18.0, 178.0),
        12.0,
        CalculationMethod::Mwl,
    );
    assert_times_ordered(&times, "Fiji");
}

#[test]
fn southern_hemisphere_antarctica_edge() {
    let times = calc_times(
        make_date(2026, time::Month::June, 21),
        Coordinates::new(-66.5, 0.0),
        0.0,
        CalculationMethod::Mwl,
    );
    assert_all_times_valid(&times, "Antarctic");
}

#[test]
fn prayer_times_throughout_year() {
    let coords = Coordinates::new(23.757283, 90.369712);
    for &(y, m, d) in &[
        (2026, time::Month::January, 1),
        (2026, time::Month::March, 20),
        (2026, time::Month::June, 21),
        (2026, time::Month::September, 23),
        (2026, time::Month::December, 21),
    ] {
        let times = calc_times(make_date(y, m, d), coords, 6.0, CalculationMethod::UmmAlQura);
        assert_times_ordered(&times, &format!("{y}-{m:?}-{d}"));
    }
}

// Fajr/Isha angle edge cases

#[test]
fn custom_fajr_angle_extremes() {
    let date = make_date(2026, time::Month::May, 24);
    let coords = Coordinates::new(23.757283, 90.369712);
    let shallow = calc_times(
        date,
        coords,
        6.0,
        CalculationMethod::Custom {
            fajr_angle: 12.0,
            isha_angle: 12.0,
        },
    );
    let steep = calc_times(
        date,
        coords,
        6.0,
        CalculationMethod::Custom {
            fajr_angle: 21.0,
            isha_angle: 21.0,
        },
    );

    assert!(
        time_to_secs(shallow.fajr) > time_to_secs(steep.fajr),
        "Shallow (12°) Fajr should be later than steep (21°)"
    );
    assert!(
        time_to_secs(shallow.isha) < time_to_secs(steep.isha),
        "Shallow (12°) Isha should be earlier than steep (21°)"
    );
}

// Physical constraints

#[test]
fn fajr_is_before_sunrise_worldwide() {
    let date = make_date(2026, time::Month::June, 21);
    for (name, coords, tz) in [
        ("Tokyo", Coordinates::new(35.6762, 139.6503), 9.0),
        ("Cairo", Coordinates::new(30.0444, 31.2357), 2.0),
        ("Moscow", Coordinates::new(55.7558, 37.6173), 3.0),
        ("Delhi", Coordinates::new(28.6139, 77.2090), 5.5),
        ("Jakarta", Coordinates::new(-6.2088, 106.8456), 7.0),
        ("Cape Town", Coordinates::new(-33.9249, 18.4241), 2.0),
        ("Buenos Aires", Coordinates::new(-34.6037, -58.3816), -3.0),
    ] {
        let times = calc_times(date, coords, tz, CalculationMethod::Mwl);
        assert!(
            time_to_secs(times.fajr) < time_to_secs(times.sunrise),
            "{name}: Fajr not before Sunrise"
        );
        assert!(
            time_to_secs(times.asr) < time_to_secs(times.maghrib),
            "{name}: Asr not before Maghrib"
        );
        assert!(
            time_to_secs(times.isha) > time_to_secs(times.maghrib),
            "{name}: Isha not after Maghrib"
        );
    }
}

#[test]
fn dhuhr_tracks_solar_noon_across_longitudes() {
    let date = make_date(2026, time::Month::May, 24);
    for (name, coords, tz) in [
        ("GMT", Coordinates::new(51.5, 0.0), 0.0),
        ("Perth", Coordinates::new(-31.9, 115.9), 8.0),
        ("NYC", Coordinates::new(40.7, -75.0), -5.0),
    ] {
        let times = calc_times(date, coords, tz, CalculationMethod::Mwl);
        let (h, m) = hm(&times.dhuhr);
        let mins = h as i64 * 60 + m as i64;
        assert!((mins - 720).abs() <= 20, "{name} Dhuhr: {h:02}:{m:02}");
    }
}

#[test]
fn latitude_extremes_return_none_or_sensible() {
    let date = make_date(2026, time::Month::May, 24);
    for (label, lat) in [("North Pole", 90.0), ("South Pole", -90.0)] {
        if let Some(times) = calculate_prayer_times(
            date,
            Coordinates::new(lat, 0.0),
            0.0,
            0.0,
            CalculationMethod::Mwl,
            AsrMethod::Shafi,
            PrayerAdjustments::zero(),
        ) {
            assert_all_times_valid(&times, label);
        }
    }
}

// SPA transit computation

#[test]
fn spa_transit_known_values() {
    for (label, lat, lon, tz, lo, hi) in [
        ("Dhaka", 23.757283, 90.369712, 6.0, 11.5, 12.5),
        ("Makkah", 21.422487, 39.826206, 3.0, 12.0, 12.4),
        ("London", 51.5074, -0.1278, 1.0, 12.5, 13.5),
    ] {
        let spa = SpaInputs::new(2026, 5, 24, lat, lon)
            .timezone(tz)
            .function(SpaFunction::ZaRts);
        let transit = spa_calculate(&spa).unwrap().suntransit.unwrap();
        assert!(transit > lo && transit < hi, "{label} transit: {transit}");
    }
}

#[test]
fn umm_al_qura_fajr_is_angle_based_not_interval() {
    let date = make_date(2026, time::Month::May, 24);
    let coords = Coordinates::new(23.757283, 90.369712);
    let uaq = calc_times(date, coords, 6.0, CalculationMethod::UmmAlQura);
    let custom = calc_times(
        date,
        coords,
        6.0,
        CalculationMethod::Custom {
            fajr_angle: 18.5,
            isha_angle: 18.5,
        },
    );
    let diff = (time_to_secs(uaq.fajr) - time_to_secs(custom.fajr)).abs();
    assert!(diff <= 120, "UmmAlQura Fajr differs from Custom(18.5°) by {diff}s");
}

#[test]
fn bulk_calculation_smoke_test() {
    let coords = Coordinates::new(23.757283, 90.369712);
    let start = make_date(2026, time::Month::January, 1);
    for offset in 0..365 {
        let _ = calc_times(
            start.saturating_add(time::Duration::days(offset)),
            coords,
            6.0,
            CalculationMethod::UmmAlQura,
        );
    }
}

#[test]
#[rustfmt::skip]
fn verify_solar_noon_against_api_reference() {
    for (name, lat, lon, tz, method, date, (eh, em)) in [
        ("Dhaka",   23.757283, 90.369712, 6.0, CalculationMethod::UmmAlQura, make_date(2026, time::Month::May, 24), (11, 55)),
        ("Makkah",  21.422487, 39.826206, 3.0, CalculationMethod::UmmAlQura, make_date(2026, time::Month::May, 24), (12, 18)),
        ("London",  51.5074,   -0.1278,   1.0, CalculationMethod::Isna,      make_date(2026, time::Month::May, 24), (12, 57)),
        ("NewYork", 40.7128,  -74.0060,  -4.0, CalculationMethod::Egypt,     make_date(2026, time::Month::May, 24), (12, 53)),
        ("Sydney", -33.8688,  151.2093,  10.0, CalculationMethod::Karachi,   make_date(2026, time::Month::May, 24), (11, 52)),
        ("Tromsø",  69.6492,   18.9553,   2.0, CalculationMethod::Mwl,       make_date(2026, time::Month::June, 21), (12, 46)),
    ] {
        let times = calc_times(date, Coordinates::new(lat, lon), tz, method);
        let (ah, am) = hm(&times.dhuhr);
        let diff = (ah as i64 * 60 + am as i64) - (eh as i64 * 60 + em as i64);
        assert!(
            diff.abs() <= 2,
            "{name}: Dhuhr {ah:02}:{am:02} vs API {eh:02}:{em:02} (diff={diff}min)"
        );
    }
}
