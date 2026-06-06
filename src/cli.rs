use clap::{Parser, ValueEnum};
use std::fmt::Write;

use crate::core::*;

#[derive(Parser)]
#[command(name = "athan", version, about = "Islamic prayer times calculator")]
pub struct Cli {
    /// Latitude in degrees
    #[arg(long)]
    pub lat: Option<f64>,

    /// Longitude in degrees
    #[arg(long)]
    pub lon: Option<f64>,

    /// Timezone offset from UTC in hours (e.g. 3, -4, 5.5)
    #[arg(long)]
    pub tz: Option<f64>,

    /// Elevation in meters
    #[arg(long)]
    pub elevation: Option<f64>,

    /// Location name
    #[arg(long, default_value = "Custom")]
    pub location: String,

    /// Calculation method
    #[arg(long, value_enum, default_value_t = MethodArg::UmmAlQura)]
    pub method: MethodArg,

    /// Asr juristic method
    #[arg(long, value_enum, default_value_t = AsrArg::Shafi)]
    pub asr: AsrArg,

    /// Auto-detect location from IP (requires network)
    #[arg(long)]
    pub detect: bool,

    /// Hijri date display
    #[arg(long)]
    pub hijri: bool,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum MethodArg {
    Mwl,
    Egypt,
    Karachi,
    UmmAlQura,
    Isna,
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

#[derive(Clone, Copy, ValueEnum)]
pub enum AsrArg {
    Shafi,
    Hanafi,
}

impl From<AsrArg> for AsrMethod {
    fn from(a: AsrArg) -> Self {
        match a {
            AsrArg::Shafi => AsrMethod::Shafi,
            AsrArg::Hanafi => AsrMethod::Hanafi,
        }
    }
}

fn format_duration(d: time::Duration) -> String {
    let secs = d.whole_seconds().max(0);
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let location = if cli.detect {
        let detected = detect_location()?;
        Location {
            name: detected.name,
            coordinates: Coordinates::new(
                cli.lat.unwrap_or(detected.lat),
                cli.lon.unwrap_or(detected.lon),
            ),
            timezone_offset: cli.tz.unwrap_or(detected.offset),
            elevation: cli.elevation.unwrap_or(detected.elevation),
        }
    } else {
        let (lat, lon, tz) = match (cli.lat, cli.lon, cli.tz) {
            (Some(lat), Some(lon), Some(tz)) => (lat, lon, tz),
            _ => return Err("--lat, --lon, and --tz are required (or use --detect)".into()),
        };
        Location {
            name: cli.location,
            coordinates: Coordinates::new(lat, lon),
            timezone_offset: tz,
            elevation: cli.elevation.unwrap_or(0.0),
        }
    };

    let method: CalculationMethod = cli.method.into();
    let asr: AsrMethod = cli.asr.into();
    let adjustments = PrayerAdjustments::prayer_start_safety();
    let now = time::OffsetDateTime::now_utc();

    let data = calculate_daily_prayer_data(now, &location, method, asr, adjustments);

    let prayer_times = data
        .prayer_times
        .ok_or("Cannot compute prayer times for this location/date.")?;

    let offset = time::UtcOffset::from_whole_seconds((location.timezone_offset * 3600.0) as i32)
        .unwrap_or(time::UtcOffset::UTC);
    let now_local = now.to_offset(offset);

    let (next_prayer, _next_time) = next_prayer(&prayer_times, now_local.time());

    let date_fmt = time::format_description::parse("[weekday], [month repr:long] [day], [year]").unwrap();
    let date_str = now_local.format(&date_fmt).unwrap_or_default();

    let mut out = String::new();

    writeln!(out, "  {}", &location.name).unwrap();
    if cli.hijri {
        writeln!(out, "  {}", data.hijri_date.display()).unwrap();
    } else {
        writeln!(out, "  {date_str}").unwrap();
    }
    writeln!(out).unwrap();

    let times = prayer_times.as_array();
    let name_width = 10;
    let time_width = 10;

    writeln!(
        out,
        "  {:<name_width$} {:>time_width$}   {}",
        "PRAYER", "TIME", "STATUS"
    )
    .unwrap();
    writeln!(
        out,
        "  {: <name_width$} {: >time_width$}   {}",
        "----------", "----------", "------"
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
            format_time(time),
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
    writeln!(
        out,
        "  Method: {} | Asr: {}",
        method.description(),
        asr.label()
    )
    .unwrap();
    if cli.detect {
        writeln!(out, "  (Location auto-detected from IP)").unwrap();
    }

    print!("{out}");
    Ok(())
}
