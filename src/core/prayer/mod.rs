use crate::core::types::*;

const EARTH_RADIUS: f64 = 6378140.0;
const AU_IN_METERS: f64 = 149597870700.0;
const ER_PER_AU: f64 = AU_IN_METERS / EARTH_RADIUS;
const J2000_JD: f64 = 2451545.0;
const JULIAN_CENTURY_DAYS: f64 = 36525.0;
const EARTH_B_OVER_A: f64 = 0.99664719; // WGS84 polar-to-equatorial radius ratio (1 - f)
const HIGH_LAT_THRESHOLD: f64 = 48.5; // degrees

fn julian_centuries(jd: f64) -> f64 {
    (jd - J2000_JD) / JULIAN_CENTURY_DAYS
}

#[derive(Copy, Clone)]
struct SphericalCoords {
    lon: f64,
    distance: f64,
}

struct TopocentricSun {
    declination: f64,
    equation_of_time: f64,
}

fn cos_hour_angle_rad(alt_rad: f64, lat_rad: f64, decl_rad: f64) -> f64 {
    let (sin_lat, cos_lat) = lat_rad.sin_cos();
    let (sin_decl, cos_decl) = decl_rad.sin_cos();
    (alt_rad.sin() - sin_lat * sin_decl) / (cos_lat * cos_decl)
}

fn get_ha_or_fallback(cos_ha: f64) -> f64 {
    cos_ha.clamp(-1.0, 1.0).acos().to_degrees() / 15.0
}

fn get_julian_date(year: i32, month: i32, day: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let b = 2 - y.div_euclid(100) + y.div_euclid(400);
    let days_y = (1461 * (y + 4716)).div_euclid(4); // 1461/4 = 365.25
    let days_m = (153 * (m + 1)) / 5; // 153/5 = 30.6

    (days_y + days_m + b) as f64 + day - 1524.0
}

fn get_apparent_sun_position(jd: f64) -> SphericalCoords {
    let jc = julian_centuries(jd);
    let jc2 = jc * jc;

    let mean_longitude = (280.46646 + 36000.76983 * jc + 0.0003032 * jc2).rem_euclid(360.0);
    let mean_anomaly_rad = (357.52911 + 35999.05029 * jc - 0.0001537 * jc2)
        .rem_euclid(360.0)
        .to_radians();

    let (sin_ma, cos_ma) = mean_anomaly_rad.sin_cos();
    let sin_ma2 = sin_ma * sin_ma;
    let sin_2ma = 2.0 * sin_ma * cos_ma;
    let cos_2ma = 1.0 - 2.0 * sin_ma2;
    let sin_3ma = sin_ma * (3.0 - 4.0 * sin_ma2);

    let equation_of_center = (1.914602 - 0.004817 * jc - 0.000014 * jc2) * sin_ma
        + (0.019993 - 0.000101 * jc) * sin_2ma
        + 0.000289 * sin_3ma;

    let l_true = mean_longitude + equation_of_center;
    let distance = 1.00014 - 0.01671 * cos_ma - 0.00014 * cos_2ma;
    let omega = 125.04 - 1934.136 * jc;
    let lon = (l_true - 0.00569 - 0.00478 * omega.to_radians().sin()).to_radians();

    SphericalCoords { lon, distance }
}

fn get_topocentric_sun(jd: f64, lat: f64, lon: f64, elevation: f64) -> TopocentricSun {
    let jc = julian_centuries(jd);
    let jc2 = jc * jc;
    let obliquity_rad = (23.43929111 - 0.013004167 * jc - 0.00000016389 * jc2).to_radians();

    let sun_geo = get_apparent_sun_position(jd);

    let (sin_lon, cos_lon) = sun_geo.lon.sin_cos();
    let (sin_obliq, cos_obliq) = obliquity_rad.sin_cos();

    let ra = (cos_obliq * sin_lon).atan2(cos_lon);

    // Equatorial Cartesian sun coordinates
    let sun_x = sun_geo.distance * cos_lon;
    let sun_y = sun_geo.distance * cos_obliq * sin_lon;
    let sun_z = sun_geo.distance * sin_obliq * sin_lon;

    let jc3 = jc2 * jc;
    let gmst = 280.46061837 + 360.98564736629 * (jd - J2000_JD) + 0.000387933 * jc2 - jc3 / 38710000.0;
    let lst = (gmst + lon).rem_euclid(360.0).to_radians();
    let (sin_lst, cos_lst) = lst.sin_cos();

    let lat_rad = lat.to_radians();
    let (sin_lat, cos_lat) = lat_rad.sin_cos();
    let reduced_latitude = (EARTH_B_OVER_A * sin_lat).atan2(cos_lat);
    let (sin_red, cos_red) = reduced_latitude.sin_cos();

    let rho_cos = cos_red + (elevation / EARTH_RADIUS) * cos_lat;
    let rho_sin = EARTH_B_OVER_A * sin_red + (elevation / EARTH_RADIUS) * sin_lat;

    // Observer Cartesian coordinates
    let obs_x = (rho_cos / ER_PER_AU) * cos_lst;
    let obs_y = (rho_cos / ER_PER_AU) * sin_lst;
    let obs_z = rho_sin / ER_PER_AU;

    let topo_x = sun_x - obs_x;
    let topo_y = sun_y - obs_y;
    let topo_z = sun_z - obs_z;

    let x2_y2 = topo_x * topo_x + topo_y * topo_y; // faster than `hypot`
    let topo_decl = topo_z.atan2(x2_y2.sqrt());

    let mean_longitude = (280.46646 + 36000.76983 * jc).rem_euclid(360.0);
    let equation_of_time = ((mean_longitude - ra.to_degrees()) / 15.0 + 12.0).rem_euclid(24.0) - 12.0;

    TopocentricSun {
        declination: topo_decl,
        equation_of_time,
    }
}

struct NightFractionInput {
    year: i32,
    month: i32,
    day: i32,
    lat: f64,
    lon: f64,
    timezone_hours: f64,
    fajr_angle: f64,
    isha_angle: f64,
    isha_interval_hours: f64,
}

fn get_night_fractions(input: NightFractionInput) -> (f64, f64) {
    let jd = get_julian_date(input.year, input.month, input.day as f64);
    let lat_calc = 45.0_f64.copysign(input.lat);

    let solar = calculate_solar_position(jd, lat_calc, input.lon, input.timezone_hours);
    let lat_rad = solar.lat.to_radians();

    let ha_ss = (solar.sunset - solar.sunrise) / 2.0;
    let night_length = (24.0 - 2.0 * ha_ss).max(0.001);

    let cos_ha_fajr = cos_hour_angle_rad(-input.fajr_angle.to_radians(), lat_rad, solar.topo_decl);
    let fajr_frac = (get_ha_or_fallback(cos_ha_fajr) - ha_ss) / night_length;

    let isha_frac = if input.isha_angle == 0.0 {
        input.isha_interval_hours / night_length
    } else {
        let cos_ha_isha = cos_hour_angle_rad(-input.isha_angle.to_radians(), lat_rad, solar.topo_decl);
        (get_ha_or_fallback(cos_ha_isha) - ha_ss) / night_length
    };

    (fajr_frac, isha_frac)
}

fn dec_hours_to_time(h: f64) -> time::Time {
    let total_secs = (h * 3600.0).round().rem_euclid(86_400.0) as u32;
    let hour = (total_secs / 3600) as u8;
    let minute = ((total_secs / 60) % 60) as u8;
    let second = (total_secs % 60) as u8;
    time::Time::from_hms(hour, minute, second).unwrap()
}

struct PrayerParams {
    fajr_angle: f64,
    isha_angle: f64,
    isha_interval_hours: f64,
}

fn get_prayer_params(method: CalculationMethod, date: time::Date) -> PrayerParams {
    let (fajr_angle, isha_angle, isha_interval_hours) = method.prayer_params();
    PrayerParams {
        fajr_angle,
        isha_angle,
        isha_interval_hours: if method == CalculationMethod::UmmAlQura && crate::core::hijri::is_ramadan(date) {
            2.0
        } else {
            isha_interval_hours
        },
    }
}

fn is_valid_input(coords: &Coordinates, timezone_hours: f64, params: &PrayerParams) -> bool {
    coords.is_valid()
        && (-24.0..=24.0).contains(&timezone_hours)
        && params.fajr_angle > 0.0
        && params.fajr_angle < 90.0
        && (0.0..90.0).contains(&params.isha_angle)
        && params.isha_interval_hours >= 0.0
        && params.isha_interval_hours.is_finite()
}

struct AstroContext {
    original_lat: f64,
    lat: f64,
    topo_decl: f64,
    transit: f64,
    night_length: f64,
}

struct SolarPosition {
    lat: f64,
    topo_decl: f64,
    transit: f64,
    sunrise: f64,
    sunset: f64,
}

fn calculate_solar_position(jd: f64, original_lat: f64, lon: f64, timezone_hours: f64) -> SolarPosition {
    let mut lat = original_lat;
    for i in 0..2 {
        let topocentric = get_topocentric_sun(jd - timezone_hours / 24.0, lat, lon, 0.0);
        let transit = 12.0 + timezone_hours - (lon / 15.0) - topocentric.equation_of_time;

        let cos_ha_ss = cos_hour_angle_rad(-0.833_f64.to_radians(), lat.to_radians(), topocentric.declination);
        let ha_ss = get_ha_or_fallback(cos_ha_ss);

        if i == 1 || (ha_ss > 0.5 && ha_ss < 11.5) {
            return SolarPosition {
                lat,
                topo_decl: topocentric.declination,
                transit,
                sunrise: transit - ha_ss,
                sunset: transit + ha_ss,
            };
        }
        lat = 45.0_f64.copysign(lat);
    }
    unreachable!()
}

fn get_solstice_night_fractions(
    original_lat: f64,
    lon: f64,
    timezone_hours: f64,
    year: i32,
    params: &PrayerParams,
) -> (f64, f64) {
    if original_lat.abs() >= HIGH_LAT_THRESHOLD {
        let sol_month = if original_lat >= 0.0 { 6 } else { 12 };
        get_night_fractions(NightFractionInput {
            year,
            month: sol_month,
            day: 21,
            lat: original_lat,
            lon,
            timezone_hours,
            fajr_angle: params.fajr_angle,
            isha_angle: params.isha_angle,
            isha_interval_hours: params.isha_interval_hours,
        })
    } else {
        (0.0, 0.0)
    }
}

fn calculate_asr(lat: f64, topo_decl: f64, transit: f64, asr_method: AsrMethod) -> f64 {
    let shadow_factor = asr_method.shadow_ratio();
    let lat_rad = lat.to_radians();
    let asr_alt_rad = (1.0 / (shadow_factor + (lat_rad - topo_decl).abs().tan())).atan();
    let cos_ha_asr = cos_hour_angle_rad(asr_alt_rad, lat_rad, topo_decl);
    let ha_asr = if cos_ha_asr.abs() <= 1.0 {
        get_ha_or_fallback(cos_ha_asr)
    } else {
        3.5
    };
    transit + ha_asr
}

fn calc_twilight(ctx: &AstroContext, angle: f64, fraction: f64, border: f64, sign: f64) -> f64 {
    let cos_ha = cos_hour_angle_rad(-angle.to_radians(), ctx.lat.to_radians(), ctx.topo_decl);

    if ctx.original_lat.abs() >= HIGH_LAT_THRESHOLD && cos_ha.abs() > 1.0 {
        border + sign * ctx.night_length * fraction
    } else {
        ctx.transit + sign * get_ha_or_fallback(cos_ha)
    }
}

struct RawTimes {
    fajr: f64,
    sunrise: f64,
    dhuhr: f64,
    asr: f64,
    maghrib: f64,
    isha: f64,
}

fn apply_adjustments_and_format(raw: RawTimes, adjustments: &PrayerAdjustments) -> PrayerTimes {
    let adjust = |time: f64, prayer: Prayer| dec_hours_to_time(time + adjustments.get(prayer) as f64 / 60.0);

    PrayerTimes {
        fajr: adjust(raw.fajr, Prayer::Fajr),
        sunrise: adjust(raw.sunrise, Prayer::Sunrise),
        dhuhr: adjust(raw.dhuhr, Prayer::Dhuhr),
        asr: adjust(raw.asr, Prayer::Asr),
        maghrib: adjust(raw.maghrib, Prayer::Maghrib),
        isha: adjust(raw.isha, Prayer::Isha),
    }
}

pub fn calculate_prayer_times(
    date: time::Date,
    coords: Coordinates,
    timezone_hours: f64,
    method: CalculationMethod,
    asr_method: AsrMethod,
    adjustments: PrayerAdjustments,
) -> Option<PrayerTimes> {
    let params = get_prayer_params(method, date);
    if !is_valid_input(&coords, timezone_hours, &params) {
        return None;
    }

    let jd = get_julian_date(date.year(), date.month() as i32, date.day() as f64);
    let original_lat = coords.latitude;
    let lon = coords.longitude;

    let solar = calculate_solar_position(jd, original_lat, lon, timezone_hours);
    let night_length = 24.0 - (solar.sunset - solar.sunrise);

    let (fajr_frac_solstice, isha_frac_solstice) =
        get_solstice_night_fractions(original_lat, lon, timezone_hours, date.year(), &params);

    let ctx = AstroContext {
        original_lat,
        lat: solar.lat,
        topo_decl: solar.topo_decl,
        transit: solar.transit,
        night_length,
    };

    let asr = calculate_asr(solar.lat, solar.topo_decl, solar.transit, asr_method);
    let fajr = calc_twilight(&ctx, params.fajr_angle, fajr_frac_solstice, solar.sunrise, -1.0);
    let isha = if params.isha_angle == 0.0 {
        solar.sunset + params.isha_interval_hours
    } else {
        calc_twilight(&ctx, params.isha_angle, isha_frac_solstice, solar.sunset, 1.0)
    };

    Some(apply_adjustments_and_format(
        RawTimes {
            fajr,
            sunrise: solar.sunrise,
            dhuhr: solar.transit,
            asr,
            maghrib: solar.sunset,
            isha,
        },
        &adjustments,
    ))
}

pub fn time_to_secs(t: time::Time) -> i64 {
    (t - time::Time::MIDNIGHT).whole_seconds()
}

pub fn next_prayer(times: &PrayerTimes, now: time::Time) -> (Prayer, time::Time) {
    [
        (Prayer::Fajr, times.fajr),
        (Prayer::Sunrise, times.sunrise),
        (Prayer::Dhuhr, times.dhuhr),
        (Prayer::Asr, times.asr),
        (Prayer::Maghrib, times.maghrib),
        (Prayer::Isha, times.isha),
    ]
    .into_iter()
    .find(|&(_, t)| t > now)
    .unwrap_or((Prayer::Fajr, times.fajr))
}

pub fn time_until(target: time::Time, now: time::Time) -> time::Duration {
    if target > now {
        target - now
    } else {
        (target - now) + time::Duration::hours(24)
    }
}

#[cfg(test)]
mod tests;
