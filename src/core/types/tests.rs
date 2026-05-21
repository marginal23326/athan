use super::*;

#[test]
fn calculation_method_parameters_match_named_conventions() {
    assert_eq!(CalculationMethod::Mwl.prayer_params(), (18.0, 17.0, 0.0));
    assert_eq!(CalculationMethod::Egypt.prayer_params(), (19.5, 17.5, 0.0));
    assert_eq!(CalculationMethod::Karachi.prayer_params(), (18.0, 18.0, 0.0));
    assert_eq!(CalculationMethod::UmmAlQura.prayer_params(), (18.5, 0.0, 1.5));
    assert_eq!(CalculationMethod::Isna.prayer_params(), (15.0, 15.0, 0.0));
}

#[test]
fn custom_method_parameters_are_preserved() {
    assert_eq!(
        CalculationMethod::Custom {
            fajr_angle: 16.5,
            isha_angle: 14.0,
        }
        .prayer_params(),
        (16.5, 14.0, 0.0)
    );
    assert_eq!(
        CalculationMethod::CustomMinutes {
            fajr_angle: 15.0,
            isha_minutes: 90.0,
        }
        .prayer_params(),
        (15.0, 0.0, 1.5)
    );
}

#[test]
fn prayer_order_is_chronological_for_display() {
    let names: Vec<_> = Prayer::all().iter().map(Prayer::name).collect();
    assert_eq!(names, ["Fajr", "Sunrise", "Dhuhr", "Asr", "Maghrib", "Isha"]);
}

#[test]
fn prayer_adjustments_can_read_and_write_each_prayer() {
    let mut adjustments = PrayerAdjustments::zero();

    for (index, prayer) in Prayer::all().iter().copied().enumerate() {
        adjustments.set(prayer, index as i32 - 2);
    }

    assert_eq!(adjustments.get(Prayer::Fajr), -2);
    assert_eq!(adjustments.get(Prayer::Sunrise), -1);
    assert_eq!(adjustments.get(Prayer::Dhuhr), 0);
    assert_eq!(adjustments.get(Prayer::Asr), 1);
    assert_eq!(adjustments.get(Prayer::Maghrib), 2);
    assert_eq!(adjustments.get(Prayer::Isha), 3);
}

#[test]
fn prayer_start_safety_delays_only_prayer_start_times() {
    let adjustments = PrayerAdjustments::prayer_start_safety();

    assert_eq!(Prayer::ALL.map(|prayer| adjustments.get(prayer)), [0, 0, 1, 1, 1, 1]);
}

#[test]
fn coordinate_validation_rejects_invalid_ranges_and_nonfinite_values() {
    assert!(Coordinates::new(23.7, 90.4).is_valid());
    assert!(Coordinates::new(-90.0, -180.0).is_valid());
    assert!(Coordinates::new(90.0, 180.0).is_valid());
    assert!(!Coordinates::new(90.0001, 0.0).is_valid());
    assert!(!Coordinates::new(0.0, 180.0001).is_valid());
    assert!(!Coordinates::new(f64::NAN, 0.0).is_valid());
    assert!(!Coordinates::new(0.0, f64::INFINITY).is_valid());
}
