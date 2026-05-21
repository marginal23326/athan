use super::*;

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual}, expected {expected}, tolerance {tolerance}"
    );
}

#[test]
fn qiblah_bearings_match_known_city_values() {
    assert_close(qiblah_direction(Coordinates::new(23.8103, 90.4125)), 277.6, 0.3);
    assert_close(qiblah_direction(Coordinates::new(51.5074, -0.1278)), 118.99, 0.3);
    assert_close(qiblah_direction(Coordinates::new(40.7128, -74.0060)), 58.5, 0.3);
}

#[test]
fn qiblah_at_makkah_is_stable() {
    assert_close(qiblah_direction(Coordinates::new(MECCA_LAT, MECCA_LON)), 0.0, 0.0001);
}

#[test]
fn invalid_qiblah_coordinates_return_nan() {
    assert!(qiblah_direction(Coordinates::new(91.0, 0.0)).is_nan());
    assert!(qiblah_direction(Coordinates::new(0.0, f64::NAN)).is_nan());
}

#[test]
fn compass_direction_uses_centered_16_point_sectors() {
    assert_eq!(qiblah_compass_direction(0.0), "N");
    assert_eq!(qiblah_compass_direction(11.24), "N");
    assert_eq!(qiblah_compass_direction(11.25), "NNE");
    assert_eq!(qiblah_compass_direction(33.75), "NE");
    assert_eq!(qiblah_compass_direction(348.74), "NNW");
    assert_eq!(qiblah_compass_direction(348.75), "N");
    assert_eq!(qiblah_compass_direction(360.0), "N");
    assert_eq!(qiblah_compass_direction(-10.0), "N");
    assert_eq!(qiblah_compass_direction(f64::NAN), "N");
}
