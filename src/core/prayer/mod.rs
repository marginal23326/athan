use crate::core::types::*;

mod spa;
use spa::{SpaFunction, SpaInputs, spa_calculate};

const HIGH_LAT_THRESHOLD: f64 = 48.5; // degrees
const ASR_FALLBACK_HOURS: f64 = 3.5; // mid-afternoon fallback for extreme high-latitude Asr

fn cos_hour_angle_rad(alt_rad: f64, lat_rad: f64, decl_rad: f64) -> f64 {
    let (sin_lat, cos_lat) = lat_rad.sin_cos();
    let (sin_decl, cos_decl) = decl_rad.sin_cos();
    (alt_rad.sin() - sin_lat * sin_decl) / (cos_lat * cos_decl)
}

fn get_ha_or_fallback(cos_ha: f64) -> f64 {
    cos_ha.clamp(-1.0, 1.0).acos().to_degrees() / 15.0
}

fn dec_hours_to_time(h: f64) -> time::Time {
    let total_minutes = (h * 60.0).round() as i64;
    let wrapped = total_minutes.rem_euclid(24 * 60) as u32;
    let hour = (wrapped / 60) as u8;
    let minute = (wrapped % 60) as u8;
    time::Time::from_hms(hour, minute, 0).unwrap()
}

struct PrayerParams {
    fajr_angle: f64,
    isha_angle: f64,
    isha_interval_hours: f64,
}

fn get_prayer_params(method: CalculationMethod, date: time::Date) -> PrayerParams {
    let (fajr_angle, isha_angle, isha_interval_hours) = method.prayer_params();
    let ramadan_override = method == CalculationMethod::UmmAlQura && crate::core::hijri::is_ramadan(date);

    PrayerParams {
        fajr_angle,
        isha_angle,
        isha_interval_hours: if ramadan_override { 2.0 } else { isha_interval_hours },
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

fn calculate_solar_position(
    date: time::Date,
    original_lat: f64,
    lon: f64,
    timezone_hours: f64,
    elevation: f64,
) -> SolarPosition {
    let mut lat = original_lat;

    for i in 0..2 {
        let inputs = SpaInputs::new(date.year(), date.month() as u8 as i32, date.day() as i32, lat, lon)
            .timezone(timezone_hours)
            .elevation(elevation)
            .function(SpaFunction::ZaRts);

        if let Ok(outputs) = spa_calculate(&inputs)
            && let (Some(transit), Some(sunrise), Some(sunset)) = (outputs.suntransit, outputs.sunrise, outputs.sunset)
        {
            let ha_ss = if sunset >= transit {
                sunset - transit
            } else {
                (sunset + 24.0) - transit
            };

            if i == 1 || (ha_ss > 0.5 && ha_ss < 11.5) {
                return SolarPosition {
                    lat,
                    topo_decl: outputs.delta_prime,
                    transit,
                    sunrise,
                    sunset,
                };
            }
        }
        lat = 45.0_f64.copysign(lat);
    }

    SolarPosition {
        lat: 45.0,
        topo_decl: 0.0,
        transit: 12.0,
        sunrise: 6.0,
        sunset: 18.0,
    }
}

fn calculate_asr(lat: f64, topo_decl: f64, transit: f64, asr_method: AsrMethod) -> f64 {
    let shadow_factor = asr_method.shadow_ratio();
    let lat_rad = lat.to_radians();
    let topo_decl_rad = topo_decl.to_radians();

    let asr_alt_rad = (1.0 / (shadow_factor + (lat_rad - topo_decl_rad).abs().tan())).atan();
    let cos_ha_asr = cos_hour_angle_rad(asr_alt_rad, lat_rad, topo_decl_rad);

    let ha_asr = if cos_ha_asr.abs() <= 1.0 {
        get_ha_or_fallback(cos_ha_asr)
    } else {
        ASR_FALLBACK_HOURS
    };
    transit + ha_asr
}

fn calc_twilight(ctx: &AstroContext, angle: f64, border: f64, sign: f64) -> f64 {
    let cos_ha = cos_hour_angle_rad(-angle.to_radians(), ctx.lat.to_radians(), ctx.topo_decl.to_radians());
    let raw_time = ctx.transit + sign * get_ha_or_fallback(cos_ha);

    if ctx.original_lat.abs() >= HIGH_LAT_THRESHOLD {
        let night_portion = angle / 60.0;
        let angle_based_time = border + sign * ctx.night_length * night_portion;

        if angle_based_time.is_finite() {
            if sign < 0.0 {
                raw_time.max(angle_based_time)
            } else {
                raw_time.min(angle_based_time)
            }
        } else {
            raw_time
        }
    } else {
        raw_time
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
    elevation: f64,
    method: CalculationMethod,
    asr_method: AsrMethod,
    adjustments: PrayerAdjustments,
) -> Option<PrayerTimes> {
    let params = get_prayer_params(method, date);
    if !is_valid_input(&coords, timezone_hours, &params) {
        return None;
    }

    let original_lat = coords.latitude;
    let lon = coords.longitude;

    let solar = calculate_solar_position(date, original_lat, lon, timezone_hours, elevation);
    let night_length = 24.0 - (solar.sunset - solar.sunrise);

    let ctx = AstroContext {
        original_lat,
        lat: solar.lat,
        topo_decl: solar.topo_decl,
        transit: solar.transit,
        night_length,
    };

    let asr = calculate_asr(solar.lat, solar.topo_decl, solar.transit, asr_method);
    let fajr = calc_twilight(&ctx, params.fajr_angle, solar.sunrise, -1.0);
    let isha = if params.isha_angle == 0.0 {
        solar.sunset + params.isha_interval_hours
    } else {
        calc_twilight(&ctx, params.isha_angle, solar.sunset, 1.0)
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
