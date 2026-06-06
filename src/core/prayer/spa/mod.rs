use std::fmt;

#[rustfmt::skip]
mod spa_data;
use spa_data::*;

pub const SUN_RADIUS: f64 = 0.26667;
const EARTH_RADIUS: f64 = 6378137.0;
const EARTH_B_OVER_A: f64 = 0.9966471893352525;

const SUN_TRANSIT: usize = 0;
const SUN_RISE: usize = 1;
const SUN_SET: usize = 2;

const JD_MINUS: usize = 0;
const JD_ZERO: usize = 1;
const JD_PLUS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpaFunction {
    ZaRts,
    #[default]
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaError {
    InvalidYear,
    InvalidMonth,
    InvalidDay,
    InvalidHour,
    InvalidMinute,
    InvalidSecond,
    InvalidDeltaT,
    InvalidTimezone,
    InvalidLongitude,
    InvalidLatitude,
    InvalidElevation,
    InvalidPressure,
    InvalidTemperature,
    InvalidAtmosRefract,
    InvalidDeltaUt1,
}

impl fmt::Display for SpaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::InvalidYear => "Year out of range (-2000 to 6000)",
            Self::InvalidMonth => "Month out of range (1 to 12)",
            Self::InvalidDay => "Day out of range (1 to 31)",
            Self::InvalidHour => "Hour out of range (0 to 24)",
            Self::InvalidMinute => "Minute out of range (0 to 59)",
            Self::InvalidSecond => "Second out of range (0 to <60)",
            Self::InvalidDeltaT => "Delta T out of range (|delta_t| <= 8000)",
            Self::InvalidTimezone => "Timezone out of range (|timezone| <= 18)",
            Self::InvalidLongitude => "Longitude out of range (|longitude| <= 180)",
            Self::InvalidLatitude => "Latitude out of range (|latitude| <= 90)",
            Self::InvalidElevation => "Elevation out of range (>= -6500000)",
            Self::InvalidPressure => "Pressure out of range (0 to 5000)",
            Self::InvalidTemperature => "Temperature out of range (> -273 and <= 6000)",
            Self::InvalidAtmosRefract => "Atmospheric refraction out of range (|atmos_refract| <= 5)",
            Self::InvalidDeltaUt1 => "Delta UT1 out of range (-1 to 1)",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for SpaError {}

#[derive(Debug, Clone)]
pub struct SpaInputs {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: f64,
    pub delta_ut1: f64,
    pub delta_t: f64,
    pub timezone: f64,
    pub longitude: f64,
    pub latitude: f64,
    pub elevation: f64,
    pub pressure: f64,
    pub temperature: f64,
    pub atmos_refract: f64,
    pub function: SpaFunction,
}

impl SpaInputs {
    pub fn new(year: i32, month: i32, day: i32, latitude: f64, longitude: f64) -> Self {
        Self {
            year,
            month,
            day,
            latitude,
            longitude,
            hour: 12,
            minute: 0,
            second: 0.0,
            timezone: 0.0,
            delta_t: 69.15,
            delta_ut1: 0.0,
            elevation: 0.0,
            pressure: 1013.25,
            temperature: 15.0,
            atmos_refract: 0.5667,
            function: SpaFunction::All,
        }
    }

    pub fn timezone(mut self, tz: f64) -> Self {
        self.timezone = tz;
        self
    }

    pub fn elevation(mut self, elevation: f64) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn function(mut self, function: SpaFunction) -> Self {
        self.function = function;
        self
    }
}

impl SpaInputs {
    pub fn validate(&self) -> Result<(), SpaError> {
        if !(-2000..=6000).contains(&self.year) {
            return Err(SpaError::InvalidYear);
        }
        if !(1..=12).contains(&self.month) {
            return Err(SpaError::InvalidMonth);
        }
        if !(1..=31).contains(&self.day) {
            return Err(SpaError::InvalidDay);
        }
        if !(0..=24).contains(&self.hour) {
            return Err(SpaError::InvalidHour);
        }
        if !(0..=59).contains(&self.minute) {
            return Err(SpaError::InvalidMinute);
        }
        if !(0.0..60.0).contains(&self.second) {
            return Err(SpaError::InvalidSecond);
        }
        if !(0.0..=5000.0).contains(&self.pressure) {
            return Err(SpaError::InvalidPressure);
        }

        if self.temperature <= -273.0 || self.temperature > 6000.0 {
            return Err(SpaError::InvalidTemperature);
        }
        if self.delta_ut1 <= -1.0 || self.delta_ut1 >= 1.0 {
            return Err(SpaError::InvalidDeltaUt1);
        }
        if self.hour == 24 && self.minute > 0 {
            return Err(SpaError::InvalidMinute);
        }
        if self.hour == 24 && self.second > 0.0 {
            return Err(SpaError::InvalidSecond);
        }

        if self.delta_t.abs() > 8000.0 {
            return Err(SpaError::InvalidDeltaT);
        }
        if self.timezone.abs() > 18.0 {
            return Err(SpaError::InvalidTimezone);
        }
        if self.longitude.abs() > 180.0 {
            return Err(SpaError::InvalidLongitude);
        }
        if self.latitude.abs() > 90.0 {
            return Err(SpaError::InvalidLatitude);
        }
        if self.atmos_refract.abs() > 5.0 {
            return Err(SpaError::InvalidAtmosRefract);
        }
        if self.elevation < -6500000.0 {
            return Err(SpaError::InvalidElevation);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SpaOutputs {
    pub delta_prime: f64,
    pub suntransit: Option<f64>,
    pub sunrise: Option<f64>,
    pub sunset: Option<f64>,
}

// Core Math / Utility Functions
pub fn limit_degrees(degrees: f64) -> f64 {
    degrees.rem_euclid(360.0)
}
fn limit_degrees180pm(degrees: f64) -> f64 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}
fn limit_zero2one(value: f64) -> f64 {
    value.rem_euclid(1.0)
}

fn dayfrac_to_local_hr(dayfrac: f64, timezone: f64) -> f64 {
    24.0 * limit_zero2one(dayfrac + timezone / 24.0)
}

pub fn third_order_polynomial(a: f64, b: f64, c: f64, d: f64, x: f64) -> f64 {
    a.mul_add(x, b).mul_add(x, c).mul_add(x, d)
}

// Calculations
fn julian_day(year: i32, month: i32, day: i32, hour: i32, minute: i32, second: f64, dut1: f64, tz: f64) -> f64 {
    let day_decimal = day as f64 + (hour as f64 - tz + (minute as f64 + (second + dut1) / 60.0) / 60.0) / 24.0;

    let (y, m) = if month < 3 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };

    let days_y = (1461 * (y + 4716)) / 4;
    let days_m = (153 * (m + 1)) / 5;

    let mut jd = (days_y + days_m) as f64 + day_decimal - 1524.5;

    if jd > 2299160.0 {
        let a = y / 100;
        jd += f64::from(2 - a + a / 4);
    }
    jd
}

fn earth_periodic_term_summation(terms: &[[f64; 3]], jme: f64) -> f64 {
    terms.iter().map(|t| t[0] * (t[1] + t[2] * jme).cos()).sum()
}

fn earth_values(terms: impl Iterator<Item = f64>, jme: f64) -> f64 {
    let (mut sum, mut power) = (0.0, 1.0);
    for val in terms {
        sum += val * power;
        power *= jme;
    }
    sum / 1.0e8
}

fn earth_heliocentric_longitude(jme: f64) -> f64 {
    let terms = L_TERMS.iter().map(|t| earth_periodic_term_summation(t, jme));
    limit_degrees(earth_values(terms, jme).to_degrees())
}

fn earth_heliocentric_latitude(jme: f64) -> f64 {
    let terms = B_TERMS.iter().map(|t| earth_periodic_term_summation(t, jme));
    earth_values(terms, jme).to_degrees()
}

fn earth_radius_vector(jme: f64) -> f64 {
    let terms = R_TERMS.iter().map(|t| earth_periodic_term_summation(t, jme));
    earth_values(terms, jme)
}

fn nutation_longitude_and_obliquity(jce: f64, x: &[f64; 5]) -> (f64, f64) {
    let (psi, eps) = Y_TERMS
        .iter()
        .zip(&PE_TERMS)
        .fold((0.0, 0.0), |(acc_psi, acc_eps), (y_term, pe_term)| {
            let xy_sum: f64 = x
                .iter()
                .zip(y_term)
                .map(|(x_val, &y_val)| x_val * f64::from(y_val))
                .sum();
            let xy_rad = xy_sum.to_radians();
            (
                acc_psi + (pe_term[0] + jce * pe_term[1]) * xy_rad.sin(),
                acc_eps + (pe_term[2] + jce * pe_term[3]) * xy_rad.cos(),
            )
        });

    (psi / 36_000_000.0, eps / 36_000_000.0)
}

// Pure Functional Pipeline Structs
struct GeocentricCoords {
    alpha: f64,
    delta: f64,
    nu: f64,
    r: f64,
}

struct SunEventsOutputs {
    transit: f64,
    rise: f64,
    set: f64,
}

fn calculate_geocentric_sun_coords(jd: f64, delta_t: f64) -> GeocentricCoords {
    let jc = (jd - 2451545.0) / 36525.0;
    let jde = jd + delta_t / 86400.0;
    let jce = (jde - 2451545.0) / 36525.0;
    let jme = jce / 10.0;

    let l = earth_heliocentric_longitude(jme);
    let b = earth_heliocentric_latitude(jme);
    let r = earth_radius_vector(jme);
    let theta = limit_degrees(l + 180.0);
    let beta = -b;

    let x = [
        third_order_polynomial(1.0 / 189474.0, -0.0019142, 445267.11148, 297.85036, jce), // x0
        third_order_polynomial(-1.0 / 300000.0, -0.0001603, 35999.05034, 357.52772, jce), // x1
        third_order_polynomial(1.0 / 56250.0, 0.0086972, 477198.867398, 134.96298, jce),  // x2
        third_order_polynomial(1.0 / 327270.0, -0.0036825, 483202.017538, 93.27191, jce), // x3
        third_order_polynomial(1.0 / 450000.0, 0.0020708, -1934.136261, 125.04452, jce),  // x4
    ];

    let (del_psi, del_epsilon) = nutation_longitude_and_obliquity(jce, &x);

    let u = jme / 10.0;
    let epsilon0 = 84381.448
        + u * (-4680.93
            + u * (-1.55
                + u * (1999.25
                    + u * (-51.38 + u * (-249.67 + u * (-39.05 + u * (7.12 + u * (27.87 + u * (5.79 + u * 2.45)))))))));

    let epsilon = del_epsilon + epsilon0 / 3600.0;
    let del_tau = -20.4898 / (3600.0 * r);
    let lamda = theta + del_psi + del_tau;

    let nu0 =
        limit_degrees(280.46061837 + 360.98564736629 * (jd - 2451545.0) + jc * jc * (0.000387933 - jc / 38710000.0));
    let epsilon_rad = epsilon.to_radians();
    let (sin_eps, cos_eps) = epsilon_rad.sin_cos();
    let nu = nu0 + del_psi * cos_eps;

    let lamda_rad = lamda.to_radians();
    let (sin_lamda, cos_lamda) = lamda_rad.sin_cos();
    let beta_rad = beta.to_radians();
    let (sin_beta, cos_beta) = beta_rad.sin_cos();

    let alpha = limit_degrees(
        (sin_lamda * cos_eps - beta_rad.tan() * sin_eps)
            .atan2(cos_lamda)
            .to_degrees(),
    );
    let delta = (sin_beta * cos_eps + cos_beta * sin_eps * sin_lamda)
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();

    GeocentricCoords { alpha, delta, nu, r }
}

fn calculate_sun_rise_transit_set(inputs: &SpaInputs) -> Option<SunEventsOutputs> {
    let sun_rts_jd = julian_day(inputs.year, inputs.month, inputs.day, 0, 0, 0.0, 0.0, 0.0);
    let geo_rts = calculate_geocentric_sun_coords(sun_rts_jd, inputs.delta_t);

    let mut alpha = [0.0; 3];
    let mut delta = [0.0; 3];

    // Compute coordinate windows iteratively (-1 day, 0 day, +1 day)
    for (i, offset) in (-1..=1).enumerate() {
        let daily_geo = calculate_geocentric_sun_coords(sun_rts_jd + offset as f64, 0.0);
        alpha[i] = daily_geo.alpha;
        delta[i] = daily_geo.delta;
    }

    // Dip of horizon below astronomical due to elevation h: acos(R/(R+h))
    let dip = (EARTH_RADIUS / (EARTH_RADIUS + inputs.elevation)).acos().to_degrees();
    let h0_prime = -(SUN_RADIUS + inputs.atmos_refract + dip);
    let lat_rad = inputs.latitude.to_radians();
    let (sin_lat, cos_lat) = lat_rad.sin_cos();
    let delta0_rad = delta[JD_ZERO].to_radians();
    let (sin_delta0, cos_delta0) = delta0_rad.sin_cos();
    let argument = (h0_prime.to_radians().sin() - sin_lat * sin_delta0) / (cos_lat * cos_delta0);

    if !(argument.abs() <= 1.0) {
        return None; // Sun never rises or never sets
    }

    let h0 = argument.clamp(-1.0, 1.0).acos().to_degrees();

    let transit_m = (alpha[JD_ZERO] - inputs.longitude - geo_rts.nu) / 360.0;
    let m_rts = [
        limit_zero2one(transit_m),
        limit_zero2one(transit_m - h0 / 360.0),
        limit_zero2one(transit_m + h0 / 360.0),
    ];

    let mut h_prime = [0.0; 3];
    let mut delta_prime = [0.0; 3];
    let mut h_rts = [0.0; 3];
    let mut sin_delta_p = [0.0; 3];
    let mut cos_delta_p = [0.0; 3];
    let mut sin_h_p = [0.0; 3];
    let mut cos_h_p = [0.0; 3];

    for i in 0..3 {
        let nu_rts = geo_rts.nu + 360.985647 * m_rts[i];
        let n = m_rts[i] + inputs.delta_t / 86400.0;

        let a = limit_degrees180pm(alpha[JD_ZERO] - alpha[JD_MINUS]);
        let b = limit_degrees180pm(alpha[JD_PLUS] - alpha[JD_ZERO]);
        let a_prime = alpha[JD_ZERO] + n * (a + b + (b - a) * n) / 2.0;

        let da = limit_degrees180pm(delta[JD_ZERO] - delta[JD_MINUS]);
        let db = limit_degrees180pm(delta[JD_PLUS] - delta[JD_ZERO]);
        delta_prime[i] = delta[JD_ZERO] + n * (da + db + (db - da) * n) / 2.0;

        h_prime[i] = limit_degrees180pm(nu_rts + inputs.longitude - a_prime);
        let dp_rad = delta_prime[i].to_radians();
        (sin_delta_p[i], cos_delta_p[i]) = dp_rad.sin_cos();
        let hp_rad = h_prime[i].to_radians();
        (sin_h_p[i], cos_h_p[i]) = hp_rad.sin_cos();

        h_rts[i] = (sin_lat * sin_delta_p[i] + cos_lat * cos_delta_p[i] * cos_h_p[i])
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees();
    }

    let rise =
        m_rts[SUN_RISE] + (h_rts[SUN_RISE] - h0_prime) / (360.0 * cos_delta_p[SUN_RISE] * cos_lat * sin_h_p[SUN_RISE]);

    let set =
        m_rts[SUN_SET] + (h_rts[SUN_SET] - h0_prime) / (360.0 * cos_delta_p[SUN_SET] * cos_lat * sin_h_p[SUN_SET]);

    Some(SunEventsOutputs {
        transit: dayfrac_to_local_hr(m_rts[SUN_TRANSIT] - h_prime[SUN_TRANSIT] / 360.0, inputs.timezone),
        rise: dayfrac_to_local_hr(rise, inputs.timezone),
        set: dayfrac_to_local_hr(set, inputs.timezone),
    })
}

pub fn spa_calculate(inputs: &SpaInputs) -> Result<SpaOutputs, SpaError> {
    inputs.validate()?;

    let jd = julian_day(
        inputs.year,
        inputs.month,
        inputs.day,
        inputs.hour,
        inputs.minute,
        inputs.second,
        inputs.delta_ut1,
        inputs.timezone,
    );
    let geo = calculate_geocentric_sun_coords(jd, inputs.delta_t);

    let h = limit_degrees(geo.nu + inputs.longitude - geo.alpha);
    let xi = 8.794 / (3600.0 * geo.r);

    let lat_rad = inputs.latitude.to_radians();
    let (sin_lat, cos_lat) = lat_rad.sin_cos();
    let xi_rad = xi.to_radians();
    let sin_xi = xi_rad.sin();
    let h_rad = h.to_radians();
    let (sin_h, cos_h) = h_rad.sin_cos();
    let delta_rad = geo.delta.to_radians();
    let (sin_delta, cos_delta) = delta_rad.sin_cos();

    let u = (EARTH_B_OVER_A * lat_rad.tan()).atan();
    let (sin_u, cos_u) = u.sin_cos();
    let y = EARTH_B_OVER_A * sin_u + inputs.elevation * sin_lat / EARTH_RADIUS;
    let x = cos_u + inputs.elevation * cos_lat / EARTH_RADIUS;

    let del_alpha_rad = (-x * sin_xi * sin_h).atan2(cos_delta - x * sin_xi * cos_h);
    let delta_prime = ((sin_delta - y * sin_xi) * del_alpha_rad.cos())
        .atan2(cos_delta - x * sin_xi * cos_h)
        .to_degrees();

    let rts = if matches!(inputs.function, SpaFunction::ZaRts | SpaFunction::All) {
        calculate_sun_rise_transit_set(inputs)
    } else {
        None
    };

    Ok(SpaOutputs {
        delta_prime,
        suntransit: rts.as_ref().map(|r| r.transit),
        sunrise: rts.as_ref().map(|r| r.rise),
        sunset: rts.as_ref().map(|r| r.set),
    })
}
