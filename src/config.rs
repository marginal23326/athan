use athan::core::types::{AsrMethod, CalculationMethod, Location, PrayerAdjustments};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub location: Location,
    pub calculation_method: CalculationMethod,
    pub asr_method: AsrMethod,
    pub prayer_adjustments: PrayerAdjustments,
    pub show_arabic: bool,
    #[cfg(feature = "hijri")]
    pub show_hijri: bool,
}

fn config_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("", "", "athan").map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
}

pub fn load() -> Option<Config> {
    let path = config_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save(config: &Config) {
    if let Some(path) = config_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(config) {
            let _ = std::fs::write(path, content);
        }
    }
}
