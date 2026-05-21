use crate::core::types::Coordinates;

const MECCA_LAT: f64 = 21.422_487;
const MECCA_LON: f64 = 39.826_206;

pub fn qiblah_direction(coords: Coordinates) -> f64 {
    if !coords.is_valid() {
        return f64::NAN;
    }
    if (coords.latitude - MECCA_LAT).abs() < f64::EPSILON && (coords.longitude - MECCA_LON).abs() < f64::EPSILON {
        return 0.0;
    }

    let lat_r = coords.latitude.to_radians();
    let lon_r = coords.longitude.to_radians();
    let mecca_lat_r = MECCA_LAT.to_radians();
    let mecca_lon_r = MECCA_LON.to_radians();

    let dlon = mecca_lon_r - lon_r;

    let y = dlon.sin();
    let x = lat_r.cos() * mecca_lat_r.tan() - lat_r.sin() * dlon.cos();

    let bearing = y.atan2(x).to_degrees();
    (bearing + 360.0) % 360.0
}

pub fn qiblah_compass_direction(bearing: f64) -> &'static str {
    let directions = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
    ];

    if !bearing.is_finite() {
        return "N";
    }

    let normalized = bearing.rem_euclid(360.0);
    let index = ((normalized + 11.25) / 22.5).floor() as usize % directions.len();
    directions[index]
}

#[cfg(test)]
mod tests;
