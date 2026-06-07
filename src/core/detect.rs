use crate::core::types::{Coordinates, Location};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DetectError {
    #[error("HTTP request failed: {0}")]
    Network(String),
    #[error("Failed to parse response: {0}")]
    Parse(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct LocationData {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub offset: f64,
    pub elevation: f64,
}

impl From<LocationData> for Location {
    fn from(data: LocationData) -> Self {
        Location {
            name: data.name,
            coordinates: Coordinates::new(data.lat, data.lon),
            timezone_offset: data.offset,
            dst: false,
            elevation: data.elevation,
        }
    }
}

#[derive(serde::Deserialize)]
struct IpWhoisResponse {
    success: bool,
    message: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    #[serde(rename = "timezone_gmtOffset")]
    timezone_gmt_offset: Option<i32>,
}

#[derive(serde::Deserialize)]
struct ElevationResponse {
    elevation: Vec<f64>,
}

pub fn detect_location() -> Result<LocationData, DetectError> {
    let response = minreq::get("http://ipwhois.app/json/")
        .send()
        .map_err(|e| DetectError::Network(e.to_string()))?;

    let api: IpWhoisResponse = response.json().map_err(|e| DetectError::Parse(e.to_string()))?;

    if !api.success {
        return Err(DetectError::Api(api.message.unwrap_or_else(|| "Unknown error".into())));
    }

    let lat = api.latitude.unwrap_or(0.0);
    let lon = api.longitude.unwrap_or(0.0);

    let elevation = minreq::get(format!(
        "http://api.open-meteo.com/v1/elevation?latitude={lat}&longitude={lon}"
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
        offset: api.timezone_gmt_offset.unwrap_or(0) as f64 / 3600.0,
        elevation,
    })
}
