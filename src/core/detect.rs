use crate::core::types::{Coordinates, Location};

#[derive(Debug, Clone)]
pub struct LocationData {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub offset: f64,
    pub elevation: f64,
}

impl LocationData {
    pub fn into_location(self) -> Location {
        Location {
            name: self.name,
            coordinates: Coordinates::new(self.lat, self.lon),
            timezone_offset: self.offset,
            elevation: self.elevation,
        }
    }
}

#[derive(serde::Deserialize)]
struct IpApiResponse {
    status: String,
    message: Option<String>,
    city: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    offset: Option<i32>,
}

#[derive(serde::Deserialize)]
struct ElevationResponse {
    elevation: Vec<f64>,
}

pub fn detect_location() -> Result<LocationData, String> {
    let response = minreq::get("http://ip-api.com/json/?fields=status,message,city,lat,lon,timezone,offset")
        .send()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let api: IpApiResponse = response.json().map_err(|e| format!("Failed to parse response: {e}"))?;

    if api.status != "success" {
        return Err(api.message.unwrap_or_else(|| "Unknown error".into()));
    }

    let lat = api.lat.unwrap_or(0.0);
    let lon = api.lon.unwrap_or(0.0);

    let elevation = minreq::get(format!(
        "https://api.open-meteo.com/v1/elevation?latitude={lat}&longitude={lon}"
    ))
    .send()
    .ok()
    .and_then(|r| r.json::<ElevationResponse>().ok())
    .and_then(|e| e.elevation.into_iter().next())
    .unwrap_or(0.0);

    Ok(LocationData {
        name: api.city.unwrap_or_else(|| "Unknown".into()),
        lat,
        lon,
        offset: api.offset.unwrap_or(0) as f64 / 3600.0,
        elevation,
    })
}

pub fn format_time(t: time::Time) -> String {
    let (h, m, _) = t.as_hms();
    let ampm = if h < 12 { "AM" } else { "PM" };
    let h12 = if h == 0 {
        12
    } else if h > 12 {
        h - 12
    } else {
        h
    };
    format!("{}:{:02} {}", h12, m, ampm)
}
