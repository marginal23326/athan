use super::*;

#[test]
fn month_names_cover_valid_months() {
    let date = HijriDate {
        year: 1447,
        month: 9,
        day: 1,
    };

    assert_eq!(date.month_name(), "Ramadan");
    assert_eq!(date.arabic_month_name(), "رمضان");
    assert_eq!(date.display(), "Ramadan 1, 1447 AH");
    assert_eq!(date.arabic_display(), "1 رمضان 1447 هـ");
}

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
fn gregorian_conversion_returns_valid_hijri_ranges() {
    let date = time::Date::from_calendar_date(2026, time::Month::May, 23).unwrap();
    let hijri = HijriDate::from_gregorian(date);

    assert!(hijri.year > 1400);
    assert!((1..=12).contains(&hijri.month));
    assert!((1..=30).contains(&hijri.day));
}

#[test]
fn ramadan_detection_matches_umm_al_qura_calendar() {
    let ramadan = time::Date::from_calendar_date(2026, time::Month::February, 18).unwrap();
    let outside_ramadan = time::Date::from_calendar_date(2026, time::Month::May, 23).unwrap();

    assert!(is_ramadan(ramadan));
    assert!(!is_ramadan(outside_ramadan));
}
