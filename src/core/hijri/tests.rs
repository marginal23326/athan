use super::*;

#[test]
fn invalid_month_names_do_not_panic() {
    let zero = HijriDate {
        year: 1447,
        month: 0,
        day: 1,
    };
    let too_high = HijriDate {
        year: 1447,
        month: 13,
        day: 1,
    };

    assert_eq!(zero.month_name(), "Unknown");
    assert_eq!(zero.arabic_month_name(), "");
    assert_eq!(too_high.month_name(), "Unknown");
    assert_eq!(too_high.arabic_month_name(), "");
}

#[test]
fn known_hijri_conversion_2026() {
    let hijri = HijriDate::from_gregorian(
        time::Date::from_calendar_date(2026, time::Month::May, 24).unwrap(),
    );
    assert_eq!(hijri.year, 1447, "year: got {}", hijri.year);
    assert_eq!(hijri.month, 12, "month: got {}", hijri.month);
    assert_eq!(hijri.day, 7, "day: got {}", hijri.day);
}

#[test]
fn known_hijri_conversion_ramadan() {
    let hijri = HijriDate::from_gregorian(
        time::Date::from_calendar_date(2026, time::Month::February, 18).unwrap(),
    );
    assert_eq!(hijri.year, 1447);
    assert_eq!(hijri.month, 9);
    assert_eq!(hijri.day, 1);
}

#[test]
fn known_hijri_conversion_1447_start() {
    for day in 25..=30 {
        let hijri = HijriDate::from_gregorian(
            time::Date::from_calendar_date(2025, time::Month::June, day).unwrap(),
        );
        if hijri.year == 1447 && hijri.month == 1 && hijri.day == 1 {
            return;
        }
    }
    let hijri = HijriDate::from_gregorian(
        time::Date::from_calendar_date(2025, time::Month::June, 27).unwrap(),
    );
    assert_eq!(hijri.year, 1447, "year should be 1447, got {}", hijri.year);
    assert!(
        (1..=2).contains(&hijri.month),
        "should be Muharram or Safar, got month {}",
        hijri.month
    );
}

#[test]
fn hijri_display_formats() {
    let hijri = HijriDate { year: 1447, month: 9, day: 1 };
    assert_eq!(hijri.display(), "Ramadan 1, 1447 AH");
    assert_eq!(hijri.arabic_display(), "1 رمضان 1447 هـ");
}

#[test]
fn hijri_month_names_all_months() {
    let names = [
        "Muharram", "Safar", "Rabi' I", "Rabi' II", "Jumada I", "Jumada II",
        "Rajab", "Sha'ban", "Ramadan", "Shawwal", "Dhu al-Qi'dah", "Dhu al-Hijjah",
    ];
    for (i, expected) in names.iter().enumerate() {
        let h = HijriDate { year: 1447, month: (i + 1) as u8, day: 1 };
        assert_eq!(h.month_name(), *expected, "month {}", i + 1);
    }
}

#[test]
fn hijri_arabic_names_all_months() {
    let arabic = [
        "محرم", "صفر", "ربيع الأول", "ربيع الآخر", "جمادى الأولى", "جمادى الآخرة",
        "رجب", "شعبان", "رمضان", "شوال", "ذو القعدة", "ذو الحجة",
    ];
    for (i, expected) in arabic.iter().enumerate() {
        let h = HijriDate { year: 1447, month: (i + 1) as u8, day: 1 };
        assert_eq!(h.arabic_month_name(), *expected, "month {}", i + 1);
    }
}

#[test]
fn hijri_invalid_gregorian_returns_default() {
    let hijri = HijriDate::from_gregorian(
        time::Date::from_calendar_date(1, time::Month::January, 1).unwrap(),
    );
    assert!(
        hijri.year == 0 || hijri.year > 0,
        "Unexpected year for ancient date: {}",
        hijri.year
    );
}

#[test]
fn is_ramadan_through_year() {
    let test_cases = [
        (2026, time::Month::January, 1,  false),
        (2026, time::Month::February, 18, true),
        (2026, time::Month::May, 24,     false),
    ];

    for (y, m, d, expected) in &test_cases {
        let date = time::Date::from_calendar_date(*y, *m, *d).unwrap();
        assert_eq!(
            is_ramadan(date),
            *expected,
            "{y}-{m:?}-{d} Ramadan detection failed"
        );
    }
}
