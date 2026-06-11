use std::fmt::Write;
use thiserror::Error;

use crate::core::*;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("{0}")]
    Usage(&'static str),
    #[error("Cannot compute prayer times for this location/date.")]
    NoPrayerTimes,
}

/// Islamic prayer times calculator
#[derive(argp::FromArgs)]
pub struct Cli {
    /// Latitude in degrees.
    #[argp(option)]
    pub lat: Option<f64>,

    /// Longitude in degrees.
    #[argp(option)]
    pub lon: Option<f64>,

    /// Timezone offset from UTC in hours (e.g. 3, -4, 5.5).
    #[argp(option)]
    pub tz: Option<f64>,

    /// Elevation in meters.
    #[argp(option)]
    pub elevation: Option<f64>,

    /// Location name.
    #[argp(option, default = "String::from(\"Custom\")")]
    pub location: String,

    /// Calculation method.
    #[argp(option, default = "MethodArg::UmmAlQura")]
    pub method: MethodArg,

    /// Asr juristic method.
    #[argp(option, default = "AsrArg::Shafi")]
    pub asr: AsrArg,

    /// Auto-detect location from IP (requires network).
    #[cfg(feature = "detect")]
    #[argp(switch)]
    pub detect: bool,

    /// Hijri date display.
    #[cfg(feature = "hijri")]
    #[argp(switch)]
    pub hijri: bool,

    /// Enable Daylight Saving Time (+1 hr).
    #[argp(switch)]
    pub dst: bool,

    /// Format output times using a 24-hour clock.
    #[argp(switch)]
    pub use_24h: bool,
}

#[derive(Clone, Copy)]
pub enum MethodArg {
    Mwl,
    Egypt,
    Karachi,
    UmmAlQura,
    Isna,
}

impl argp::FromArgValue for MethodArg {
    fn from_arg_value(value: &std::ffi::OsStr) -> Result<Self, String> {
        match value.to_str() {
            Some("mwl") => Ok(MethodArg::Mwl),
            Some("egypt") => Ok(MethodArg::Egypt),
            Some("karachi") => Ok(MethodArg::Karachi),
            Some("umm-al-qura" | "ummalqura") => Ok(MethodArg::UmmAlQura),
            Some("isna") => Ok(MethodArg::Isna),
            Some(other) => Err(format!(
                "unknown method '{other}'. Valid values: mwl, egypt, karachi, umm-al-qura, isna"
            )),
            None => Err("method value is not valid UTF-8".into()),
        }
    }
}

impl From<MethodArg> for CalculationMethod {
    fn from(m: MethodArg) -> Self {
        match m {
            MethodArg::Mwl => CalculationMethod::Mwl,
            MethodArg::Egypt => CalculationMethod::Egypt,
            MethodArg::Karachi => CalculationMethod::Karachi,
            MethodArg::UmmAlQura => CalculationMethod::UmmAlQura,
            MethodArg::Isna => CalculationMethod::Isna,
        }
    }
}

#[derive(Clone, Copy)]
pub enum AsrArg {
    Shafi,
    Hanafi,
}

impl argp::FromArgValue for AsrArg {
    fn from_arg_value(value: &std::ffi::OsStr) -> Result<Self, String> {
        match value.to_str() {
            Some("shafi") => Ok(AsrArg::Shafi),
            Some("hanafi") => Ok(AsrArg::Hanafi),
            Some(other) => Err(format!("unknown asr method '{other}'. Valid values: shafi, hanafi")),
            None => Err("asr value is not valid UTF-8".into()),
        }
    }
}

impl From<AsrArg> for AsrMethod {
    fn from(a: AsrArg) -> Self {
        match a {
            AsrArg::Shafi => AsrMethod::Shafi,
            AsrArg::Hanafi => AsrMethod::Hanafi,
        }
    }
}

pub fn run() -> Result<(), CliError> {
    let cli: Cli = argp::parse_args_or_exit(argp::DEFAULT);

    let location = {
        #[cfg(feature = "detect")]
        if cli.detect {
            match detect_location() {
                Ok(detected) => Location {
                    name: detected.name,
                    coordinates: Coordinates::new(cli.lat.unwrap_or(detected.lat), cli.lon.unwrap_or(detected.lon)),
                    timezone_offset: cli.tz.unwrap_or(detected.offset),
                    dst: cli.dst,
                    elevation: cli.elevation.unwrap_or(detected.elevation),
                },
                Err(e) => {
                    eprintln!("Warning: could not detect location from IP ({e}). Falling back to Makkah.");
                    Location::default()
                }
            }
        } else {
            let (lat, lon, tz) = match (cli.lat, cli.lon, cli.tz) {
                (Some(lat), Some(lon), Some(tz)) => (lat, lon, tz),
                _ => return Err(CliError::Usage("--lat, --lon, and --tz are required (or use --detect)")),
            };
            Location {
                name: cli.location,
                coordinates: Coordinates::new(lat, lon),
                timezone_offset: tz,
                dst: cli.dst,
                elevation: cli.elevation.unwrap_or(0.0),
            }
        }
        #[cfg(not(feature = "detect"))]
        {
            let (lat, lon, tz) = match (cli.lat, cli.lon, cli.tz) {
                (Some(lat), Some(lon), Some(tz)) => (lat, lon, tz),
                _ => return Err(CliError::Usage("--lat, --lon, and --tz are required")),
            };
            Location {
                name: cli.location,
                coordinates: Coordinates::new(lat, lon),
                timezone_offset: tz,
                dst: cli.dst,
                elevation: cli.elevation.unwrap_or(0.0),
            }
        }
    };

    let method: CalculationMethod = cli.method.into();
    let asr: AsrMethod = cli.asr.into();
    let adjustments = PrayerAdjustments::prayer_start_safety();
    let now = time::OffsetDateTime::now_utc();

    let data = calculate_daily_prayer_data(now, &location, method, asr, adjustments);

    let prayer_times = data.prayer_times.ok_or(CliError::NoPrayerTimes)?;

    let offset = time::UtcOffset::from_whole_seconds((location.effective_timezone_offset() * 3600.0) as i32)
        .unwrap_or(time::UtcOffset::UTC);
    let now_local = now.to_offset(offset);

    let (next_prayer, _next_time) = next_prayer(&prayer_times, now_local.time());

    let date_str = now_local.format(DATE_FMT).unwrap_or_default();

    let mut out = String::new();

    writeln!(out, "  {}", &location.name).unwrap();
    #[cfg(feature = "hijri")]
    if cli.hijri {
        match &data.hijri_date {
            Some(h) => writeln!(out, "  {}", h.display()).unwrap(),
            None => writeln!(out, "  (Hijri date unavailable)").unwrap(),
        }
    } else {
        writeln!(out, "  {date_str}").unwrap();
    }
    #[cfg(not(feature = "hijri"))]
    writeln!(out, "  {date_str}").unwrap();
    writeln!(out).unwrap();

    let times = prayer_times.as_array();
    let name_width = 10;
    let time_width = 10;

    writeln!(out, "  {:<name_width$} {:>time_width$}   STATUS", "PRAYER", "TIME").unwrap();
    writeln!(
        out,
        "  {: <name_width$} {: >time_width$}   ------",
        "----------", "----------"
    )
    .unwrap();

    for &(prayer, time) in &times {
        let is_next = prayer == next_prayer;
        let status = if is_next {
            format!("{:>8}", format_duration(time_until(time, now_local.time())))
        } else if time < now_local.time() {
            "    past".to_string()
        } else {
            String::new()
        };
        let marker = if is_next { " *" } else { "  " };
        writeln!(
            out,
            "  {:<name_width$} {:>time_width$}   {}{}",
            prayer.name(),
            format_time(time, cli.use_24h),
            marker,
            status
        )
        .unwrap();
    }

    writeln!(out).unwrap();
    writeln!(
        out,
        "  Qiblah: {:.1}° {}",
        data.qiblah,
        qiblah_compass_direction(data.qiblah)
    )
    .unwrap();
    writeln!(out, "  Method: {} | Asr: {}", method.description(), asr.label()).unwrap();
    #[cfg(feature = "detect")]
    if cli.detect {
        writeln!(out, "  (Location auto-detected from IP)").unwrap();
    }

    print!("{out}");
    Ok(())
}
