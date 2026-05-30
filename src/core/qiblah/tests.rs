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

// =============================================================================
// Comprehensive Qiblah Tests
// =============================================================================

fn assert_bearing_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual:.3}, expected {expected:.3}, diff {}",
        (actual - expected).abs()
    );
}

#[test]
fn qiblah_more_cities() {
    // Makkah itself
    assert_bearing_close(qiblah_direction(Coordinates::new(21.422487, 39.826206)), 0.0, 0.01);

    // Major world cities — bearings computed via great-circle formula.
    // These are verified by manual recalculation of the formula used in the code.
    // Riyadh → Makkah: ~244° (WSW, since Makkah is SW of Riyadh)
    assert_bearing_close(qiblah_direction(Coordinates::new(24.7136, 46.6753)), 243.8, 1.0);
    // Cairo → Makkah: ~136° (SE)
    assert_bearing_close(qiblah_direction(Coordinates::new(30.0444, 31.2357)), 136.2, 1.0);
    // Istanbul → Makkah: ~152° (SSE)
    assert_bearing_close(qiblah_direction(Coordinates::new(41.0082, 28.9784)), 151.6, 1.0);
    // Delhi → Makkah: ~267° (W, almost due west since Makkah is farther west)
    assert_bearing_close(qiblah_direction(Coordinates::new(28.6139, 77.2090)), 266.6, 1.0);
    // Tokyo → Makkah: ~293° (NW)
    assert_bearing_close(qiblah_direction(Coordinates::new(35.6762, 139.6503)), 292.9, 1.0);
    // Moscow → Makkah: ~176° (S)
    assert_bearing_close(qiblah_direction(Coordinates::new(55.7558, 37.6173)), 176.4, 1.0);
    // Sydney → Makkah: ~277° (WNW)
    assert_bearing_close(qiblah_direction(Coordinates::new(-33.8688, 151.2093)), 277.4, 1.0);
}

#[test]
fn qiblah_antipode_of_makkah() {
    // Antipode of Makkah (approx 21.42°S, 140.17°W) - bearing should be ~0 or 360
    let antipode = Coordinates::new(-21.422_487, -140.173_794);
    let bearing = qiblah_direction(antipode);
    // From the antipode, every direction leads to Makkah, but the great circle
    // formula should give a finite bearing
    assert!(bearing.is_finite(), "Antipode bearing should be finite");
}

#[test]
fn qiblah_at_poles() {
    // North Pole: every direction is south; qiblah should point south-ish
    let np = qiblah_direction(Coordinates::new(89.9, 0.0));
    assert!(np.is_finite(), "Near North Pole bearing should be finite");
    // Should be roughly south
    assert!(
        (np - 180.0).abs() < 90.0,
        "Near North Pole bearing {np:.1} should be roughly south"
    );

    // South Pole
    let sp = qiblah_direction(Coordinates::new(-89.9, 0.0));
    assert!(sp.is_finite(), "Near South Pole bearing should be finite");
}

#[test]
fn qiblah_on_equator_at_mecca_longitude() {
    // On the equator at Mecca's longitude, qiblah should be due north (0°)
    let eq = qiblah_direction(Coordinates::new(0.0, 39.826206));
    assert_bearing_close(eq, 0.0, 1.0);
}



#[test]
fn qiblah_compass_all_directions() {
    // Verify all 16 compass points
    let dirs = ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
                "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
    for (i, dir) in dirs.iter().enumerate() {
        let angle = i as f64 * 22.5;
        assert_eq!(qiblah_compass_direction(angle), *dir, "angle {angle}");
    }
}

#[test]
fn qiblah_nearby_locations_point_roughly_towards_mecca() {
    // Cities near Makkah should point roughly towards Makkah
    // Jeddah: 1.5° west of Makkah → bearing should be east-ish (90°-120°)
    let jeddah = qiblah_direction(Coordinates::new(21.4858, 39.1925));
    assert_bearing_close(jeddah, 96.0, 5.0); // Makkah is east of Jeddah

    // Taif: ~70km east of Makkah → bearing should be west-ish (270°-300°)
    let taif = qiblah_direction(Coordinates::new(21.2700, 40.4158));
    assert_bearing_close(taif, 288.8, 10.0); // roughly west
}


