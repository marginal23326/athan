use std::fmt;

pub const SUN_RADIUS: f64 = 0.26667;
const EARTH_RADIUS: f64 = 6378137.0;
const EARTH_B_OVER_A: f64 = 0.9966471893352525;

const SUN_TRANSIT: usize = 0;
const SUN_RISE: usize = 1;
const SUN_SET: usize = 2;

const JD_MINUS: usize = 0;
const JD_ZERO: usize = 1;
const JD_PLUS: usize = 2;

type SunEvents = [f64; 3];
type JulianDayWindow = [f64; 3];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpaFunction {
    Za,
    ZaInc,
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
    InvalidSlope,
    InvalidAzmRotation,
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
            Self::InvalidSlope => "Slope out of range (|slope| <= 360)",
            Self::InvalidAzmRotation => "Azm rotation out of range (|azm_rotation| <= 360)",
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
    pub slope: f64,
    pub azm_rotation: f64,
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
            slope: 0.0,
            azm_rotation: 0.0,
            atmos_refract: 0.5667,
            function: SpaFunction::All,
        }
    }

    pub fn time(mut self, hour: i32, minute: i32, second: f64) -> Self {
        self.hour = hour;
        self.minute = minute;
        self.second = second;
        self
    }

    pub fn timezone(mut self, tz: f64) -> Self {
        self.timezone = tz;
        self
    }

    pub fn elevation(mut self, elevation: f64) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn environment(mut self, pressure: f64, temperature: f64) -> Self {
        self.pressure = pressure;
        self.temperature = temperature;
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

        if matches!(self.function, SpaFunction::ZaInc | SpaFunction::All) {
            if self.slope.abs() > 360.0 {
                return Err(SpaError::InvalidSlope);
            }
            if self.azm_rotation.abs() > 360.0 {
                return Err(SpaError::InvalidAzmRotation);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SpaOutputs {
    pub zenith: f64,
    pub azimuth_astro: f64,
    pub azimuth: f64,
    pub eot: f64,
    pub delta_prime: f64,
    pub incidence: Option<f64>,
    pub suntransit: Option<f64>,
    pub sunrise: Option<f64>,
    pub sunset: Option<f64>,
}

//  Earth Periodic Terms
const L_TERMS_0: [[f64; 3]; 64] = [
    [175347046.0, 0.0, 0.0], [3341656.0, 4.6692568, 6283.07585], [34894.0, 4.6261, 12566.1517],
    [3497.0, 2.7441, 5753.3849], [3418.0, 2.8289, 3.5231], [3136.0, 3.6277, 77713.7715],
    [2676.0, 4.4181, 7860.4194], [2343.0, 6.1352, 3930.2097], [1324.0, 0.7425, 11506.7698],
    [1273.0, 2.0371, 529.691], [1199.0, 1.1096, 1577.3435], [990.0, 5.233, 5884.927],
    [902.0, 2.045, 26.298], [857.0, 3.508, 398.149], [780.0, 1.179, 5223.694],
    [753.0, 2.533, 5507.553], [505.0, 4.583, 18849.228], [492.0, 4.205, 775.523],
    [357.0, 2.92, 0.067], [317.0, 5.849, 11790.629], [284.0, 1.899, 796.298],
    [271.0, 0.315, 10977.079], [243.0, 0.345, 5486.778], [206.0, 4.806, 2544.314],
    [205.0, 1.869, 5573.143], [202.0, 2.458, 6069.777], [156.0, 0.833, 213.299],
    [132.0, 3.411, 2942.463], [126.0, 1.083, 20.775], [115.0, 0.645, 0.98],
    [103.0, 0.636, 4694.003], [102.0, 0.976, 15720.839], [102.0, 4.267, 7.114],
    [99.0, 6.21, 2146.17], [98.0, 0.68, 155.42], [86.0, 5.98, 161000.69],
    [85.0, 1.3, 6275.96], [85.0, 3.67, 71430.7], [80.0, 1.81, 17260.15],
    [79.0, 3.04, 12036.46], [75.0, 1.76, 5088.63], [74.0, 3.5, 3154.69],
    [74.0, 4.68, 801.82], [70.0, 0.83, 9437.76], [62.0, 3.98, 8827.39],
    [61.0, 1.82, 7084.9], [57.0, 2.78, 6286.6], [56.0, 4.39, 14143.5],
    [56.0, 3.47, 6279.55], [52.0, 0.19, 12139.55], [52.0, 1.33, 1748.02],
    [51.0, 0.28, 5856.48], [49.0, 0.49, 1194.45], [41.0, 5.37, 8429.24],
    [41.0, 2.4, 19651.05], [39.0, 6.17, 10447.39], [37.0, 6.04, 10213.29],
    [37.0, 2.57, 1059.38], [36.0, 1.71, 2352.87], [36.0, 1.78, 6812.77],
    [33.0, 0.59, 17789.85], [30.0, 0.44, 83996.85], [30.0, 2.74, 1349.87],
    [25.0, 3.16, 4690.48]
];
const L_TERMS_1: [[f64; 3]; 34] = [
    [628331966747.0, 0.0, 0.0], [206059.0, 2.678235, 6283.07585], [4303.0, 2.6351, 12566.1517],
    [425.0, 1.59, 3.523], [119.0, 5.796, 26.298], [109.0, 2.966, 1577.344],
    [93.0, 2.59, 18849.23], [72.0, 1.14, 529.69], [68.0, 1.87, 398.15],
    [67.0, 4.41, 5507.55], [59.0, 2.89, 5223.69], [56.0, 2.17, 155.42],
    [45.0, 0.4, 796.3], [36.0, 0.47, 775.52], [29.0, 2.65, 7.11],
    [21.0, 5.34, 0.98], [19.0, 1.85, 5486.78], [19.0, 4.97, 213.3],
    [17.0, 2.99, 6275.96], [16.0, 0.03, 2544.31], [16.0, 1.43, 2146.17],
    [15.0, 1.21, 10977.08], [12.0, 2.83, 1748.02], [12.0, 3.26, 5088.63],
    [12.0, 5.27, 1194.45], [12.0, 2.08, 4694.0], [11.0, 0.77, 553.57],
    [10.0, 1.3, 6286.6], [10.0, 4.24, 1349.87], [9.0, 2.7, 242.73],
    [9.0, 5.64, 951.72], [8.0, 5.3, 2352.87], [6.0, 2.65, 9437.76],
    [6.0, 4.67, 4690.48]
];
const L_TERMS_2: [[f64; 3]; 20] = [
    [52919.0, 0.0, 0.0], [8720.0, 1.0721, 6283.0758], [309.0, 0.867, 12566.152],
    [27.0, 0.05, 3.52], [16.0, 5.19, 26.3], [16.0, 3.68, 155.42],
    [10.0, 0.76, 18849.23], [9.0, 2.06, 77713.77], [7.0, 0.83, 775.52],
    [5.0, 4.66, 1577.34], [4.0, 1.03, 7.11], [4.0, 3.44, 5573.14],
    [3.0, 5.14, 796.3], [3.0, 6.05, 5507.55], [3.0, 1.19, 242.73],
    [3.0, 6.12, 529.69], [3.0, 0.31, 398.15], [3.0, 2.28, 553.57],
    [2.0, 4.38, 5223.69], [2.0, 3.75, 0.98]
];
const L_TERMS_3: [[f64; 3]; 7] = [
    [289.0, 5.844, 6283.076], [35.0, 0.0, 0.0], [17.0, 5.49, 12566.15],
    [3.0, 5.2, 155.42], [1.0, 4.72, 3.52], [1.0, 5.3, 18849.23], [1.0, 5.97, 242.73]
];
const L_TERMS_4: [[f64; 3]; 3] = [[114.0, 3.142, 0.0], [8.0, 4.13, 6283.08], [1.0, 3.84, 12566.15]];
const L_TERMS_5: [[f64; 3]; 1] = [[1.0, 3.14, 0.0]];
const L_TERMS: &[&[[f64; 3]]] = &[&L_TERMS_0, &L_TERMS_1, &L_TERMS_2, &L_TERMS_3, &L_TERMS_4, &L_TERMS_5];

const B_TERMS_0: [[f64; 3]; 5] = [
    [280.0, 3.199, 84334.662], [102.0, 5.422, 5507.553], [80.0, 3.88, 5223.69],
    [44.0, 3.7, 2352.87], [32.0, 4.0, 1577.34]
];
const B_TERMS_1: [[f64; 3]; 2] = [[9.0, 3.9, 5507.55], [6.0, 1.73, 5223.69]];
const B_TERMS: &[&[[f64; 3]]] = &[&B_TERMS_0, &B_TERMS_1];

const R_TERMS_0: [[f64; 3]; 40] = [
    [100013989.0, 0.0, 0.0], [1670700.0, 3.0984635, 6283.07585], [13956.0, 3.05525, 12566.1517],
    [3084.0, 5.1985, 77713.7715], [1628.0, 1.1739, 5753.3849], [1576.0, 2.8469, 7860.4194],
    [925.0, 5.453, 11506.77], [542.0, 4.564, 3930.21], [472.0, 3.661, 5884.927],
    [346.0, 0.964, 5507.553], [329.0, 5.9, 5223.694], [307.0, 0.299, 5573.143],
    [243.0, 4.273, 11790.629], [212.0, 5.847, 1577.344], [186.0, 5.022, 10977.079],
    [175.0, 3.012, 18849.228], [110.0, 5.055, 5486.778], [98.0, 0.89, 6069.78],
    [86.0, 5.69, 15720.84], [86.0, 1.27, 161000.69], [65.0, 0.27, 17260.15],
    [63.0, 0.92, 529.69], [57.0, 2.01, 83996.85], [56.0, 5.24, 71430.7],
    [49.0, 3.25, 2544.31], [47.0, 2.58, 775.52], [45.0, 5.54, 9437.76],
    [43.0, 6.01, 6275.96], [39.0, 5.36, 4694.0], [38.0, 2.39, 8827.39],
    [37.0, 0.83, 19651.05], [37.0, 4.9, 12139.55], [36.0, 1.67, 12036.46],
    [35.0, 1.84, 2942.46], [33.0, 0.24, 7084.9], [32.0, 0.18, 5088.63],
    [32.0, 1.78, 398.15], [28.0, 1.21, 6286.6], [28.0, 1.9, 6279.55],
    [26.0, 4.59, 10447.39]
];
const R_TERMS_1: [[f64; 3]; 10] = [
    [103019.0, 1.10749, 6283.07585], [1721.0, 1.0644, 12566.1517], [702.0, 3.142, 0.0],
    [32.0, 1.02, 18849.23], [31.0, 2.84, 5507.55], [25.0, 1.32, 5223.69],
    [18.0, 1.42, 1577.34], [10.0, 5.91, 10977.08], [9.0, 1.42, 6275.96], [9.0, 0.27, 5486.78]
];
const R_TERMS_2: [[f64; 3]; 6] = [
    [4359.0, 5.7846, 6283.0758], [124.0, 5.579, 12566.152], [12.0, 3.14, 0.0],
    [9.0, 3.63, 77713.77], [6.0, 1.87, 5573.14], [3.0, 5.47, 18849.23]
];
const R_TERMS_3: [[f64; 3]; 2] = [[145.0, 4.273, 6283.076], [7.0, 3.92, 12566.15]];
const R_TERMS_4: [[f64; 3]; 1] = [[4.0, 2.56, 6283.08]];
const R_TERMS: &[&[[f64; 3]]] = &[&R_TERMS_0, &R_TERMS_1, &R_TERMS_2, &R_TERMS_3, &R_TERMS_4];

const Y_TERMS: [[i32; 5]; 63] = [
    [0, 0, 0, 0, 1], [-2, 0, 0, 2, 2], [0, 0, 0, 2, 2], [0, 0, 0, 0, 2], [0, 1, 0, 0, 0],
    [0, 0, 1, 0, 0], [-2, 1, 0, 2, 2], [0, 0, 0, 2, 1], [0, 0, 1, 2, 2], [-2, -1, 0, 2, 2],
    [-2, 0, 1, 0, 0], [-2, 0, 0, 2, 1], [0, 0, -1, 2, 2], [2, 0, 0, 0, 0], [0, 0, 1, 0, 1],
    [2, 0, -1, 2, 2], [0, 0, -1, 0, 1], [0, 0, 1, 2, 1], [-2, 0, 2, 0, 0], [0, 0, -2, 2, 1],
    [2, 0, 0, 2, 2], [0, 0, 2, 2, 2], [0, 0, 2, 0, 0], [-2, 0, 1, 2, 2], [0, 0, 0, 2, 0],
    [-2, 0, 0, 2, 0], [0, 0, -1, 2, 1], [0, 2, 0, 0, 0], [2, 0, -1, 0, 1], [-2, 2, 0, 2, 2],
    [0, 1, 0, 0, 1], [-2, 0, 1, 0, 1], [0, -1, 0, 0, 1], [0, 0, 2, -2, 0], [2, 0, -1, 2, 1],
    [2, 0, 1, 2, 2], [0, 1, 0, 2, 2], [-2, 1, 1, 0, 0], [0, -1, 0, 2, 2], [2, 0, 0, 2, 1],
    [2, 0, 1, 0, 0], [-2, 0, 2, 2, 2], [-2, 0, 1, 2, 1], [2, 0, -2, 0, 1], [2, 0, 0, 0, 1],
    [0, -1, 1, 0, 0], [-2, -1, 0, 2, 1], [-2, 0, 0, 0, 1], [0, 0, 2, 2, 1], [-2, 0, 2, 0, 1],
    [-2, 1, 0, 2, 1], [0, 0, 1, -2, 0], [-1, 0, 1, 0, 0], [-2, 1, 0, 0, 0], [1, 0, 0, 0, 0],
    [0, 0, 1, 2, 0], [0, 0, -2, 2, 2], [-1, -1, 1, 0, 0], [0, 1, 1, 0, 0], [0, -1, 1, 2, 2],
    [2, -1, -1, 2, 2], [0, 0, 3, 2, 2], [2, -1, 0, 2, 2]
];

const PE_TERMS: [[f64; 4]; 63] = [
    [-171996.0, -174.2, 92025.0, 8.9], [-13187.0, -1.6, 5736.0, -3.1], [-2274.0, -0.2, 977.0, -0.5],
    [2062.0, 0.2, -895.0, 0.5], [1426.0, -3.4, 54.0, -0.1], [712.0, 0.1, -7.0, 0.0],
    [-517.0, 1.2, 224.0, -0.6], [-386.0, -0.4, 200.0, 0.0], [-301.0, 0.0, 129.0, -0.1],
    [217.0, -0.5, -95.0, 0.3], [-158.0, 0.0, 0.0, 0.0], [129.0, 0.1, -70.0, 0.0],
    [123.0, 0.0, -53.0, 0.0], [63.0, 0.0, 0.0, 0.0], [63.0, 0.1, -33.0, 0.0],
    [-59.0, 0.0, 26.0, 0.0], [-58.0, -0.1, 32.0, 0.0], [-51.0, 0.0, 27.0, 0.0],
    [48.0, 0.0, 0.0, 0.0], [46.0, 0.0, -24.0, 0.0], [-38.0, 0.0, 16.0, 0.0],
    [-31.0, 0.0, 13.0, 0.0], [29.0, 0.0, 0.0, 0.0], [29.0, 0.0, -12.0, 0.0],
    [26.0, 0.0, 0.0, 0.0], [-22.0, 0.0, 0.0, 0.0], [21.0, 0.0, -10.0, 0.0],
    [17.0, -0.1, 0.0, 0.0], [16.0, 0.0, -8.0, 0.0], [-16.0, 0.1, 7.0, 0.0],
    [-15.0, 0.0, 9.0, 0.0], [-13.0, 0.0, 7.0, 0.0], [-12.0, 0.0, 6.0, 0.0],
    [11.0, 0.0, 0.0, 0.0], [-10.0, 0.0, 5.0, 0.0], [-8.0, 0.0, 3.0, 0.0],
    [7.0, 0.0, -3.0, 0.0], [-7.0, 0.0, 0.0, 0.0], [-7.0, 0.0, 3.0, 0.0],
    [-7.0, 0.0, 3.0, 0.0], [6.0, 0.0, 0.0, 0.0], [6.0, 0.0, -3.0, 0.0],
    [6.0, 0.0, -3.0, 0.0], [-6.0, 0.0, 3.0, 0.0], [-6.0, 0.0, 3.0, 0.0],
    [5.0, 0.0, 0.0, 0.0], [-5.0, 0.0, 3.0, 0.0], [-5.0, 0.0, 3.0, 0.0],
    [-5.0, 0.0, 3.0, 0.0], [4.0, 0.0, 0.0, 0.0], [4.0, 0.0, 0.0, 0.0],
    [4.0, 0.0, 0.0, 0.0], [-4.0, 0.0, 0.0, 0.0], [-4.0, 0.0, 0.0, 0.0],
    [-4.0, 0.0, 0.0, 0.0], [3.0, 0.0, 0.0, 0.0], [-3.0, 0.0, 0.0, 0.0],
    [-3.0, 0.0, 0.0, 0.0], [-3.0, 0.0, 0.0, 0.0], [-3.0, 0.0, 0.0, 0.0],
    [-3.0, 0.0, 0.0, 0.0], [-3.0, 0.0, 0.0, 0.0], [-3.0, 0.0, 0.0, 0.0]
];

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

pub fn sun_mean_longitude(jme: f64) -> f64 {
    limit_degrees(280.4664567 + jme * (360007.6982779 + jme * (0.03032028 + jme * (1.0 / 49931.0 + jme * (-1.0 / 15300.0 + jme * (-1.0 / 2_000_000.0))))))
}

pub fn equation_of_time(m: f64, alpha: f64, del_psi: f64, epsilon: f64) -> f64 {
    let mut e = 4.0 * (m - 0.0057183 - alpha + del_psi * epsilon.to_radians().cos());
    if e < -20.0 { e += 1440.0; } else if e > 20.0 { e -= 1440.0; }
    e
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
    jme: f64,
    del_psi: f64,
    epsilon: f64,
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

    GeocentricCoords {
        alpha,
        delta,
        nu,
        jme,
        del_psi,
        epsilon,
        r,
    }
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

    let rise = m_rts[SUN_RISE]
        + (h_rts[SUN_RISE] - h0_prime)
            / (360.0 * cos_delta_p[SUN_RISE] * cos_lat * sin_h_p[SUN_RISE]);

    let set = m_rts[SUN_SET]
        + (h_rts[SUN_SET] - h0_prime)
            / (360.0 * cos_delta_p[SUN_SET] * cos_lat * sin_h_p[SUN_SET]);

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

    let m = sun_mean_longitude(geo.jme);
    let eot = equation_of_time(m, geo.alpha, geo.del_psi, geo.epsilon);

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

    let h_prime = h - del_alpha_rad.to_degrees();
    let dp_rad = delta_prime.to_radians();
    let (sin_dp, cos_dp) = dp_rad.sin_cos();
    let hp_rad = h_prime.to_radians();
    let (sin_hp, cos_hp) = hp_rad.sin_cos();
    let e0 = (sin_lat * sin_dp + cos_lat * cos_dp * cos_hp)
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();

    let del_e = if e0 >= -(SUN_RADIUS + inputs.atmos_refract) {
        (inputs.pressure / 1010.0) * (283.0 / (273.0 + inputs.temperature)) * 1.02
            / (60.0 * (e0 + 10.3 / (e0 + 5.11)).to_radians().tan())
    } else {
        0.0
    };

    let e = e0 + del_e;
    let zenith = 90.0 - e;
    let azimuth_astro = limit_degrees(
        sin_hp.atan2(cos_hp * sin_lat - dp_rad.tan() * cos_lat).to_degrees(),
    );

    let incidence = matches!(inputs.function, SpaFunction::ZaInc | SpaFunction::All).then(|| {
        let (sin_zenith, cos_zenith) = zenith.to_radians().sin_cos();
        let (sin_slope, cos_slope) = inputs.slope.to_radians().sin_cos();
        let cos_azm_diff = (azimuth_astro - inputs.azm_rotation).to_radians().cos();

        (cos_zenith * cos_slope + sin_slope * sin_zenith * cos_azm_diff)
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    });

    let rts = if matches!(inputs.function, SpaFunction::ZaRts | SpaFunction::All) {
        calculate_sun_rise_transit_set(inputs)
    } else { None };

    Ok(SpaOutputs {
        zenith,
        azimuth_astro,
        azimuth: limit_degrees(azimuth_astro + 180.0),
        eot,
        delta_prime,
        incidence,
        suntransit: rts.as_ref().map(|r| r.transit),
        sunrise: rts.as_ref().map(|r| r.rise),
        sunset: rts.as_ref().map(|r| r.set),
    })
}
