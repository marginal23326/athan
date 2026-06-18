use athan::core::types::{AsrMethod, CalculationMethod, Location, PrayerAdjustments};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub location: Location,
    pub calculation_method: CalculationMethod,
    pub asr_method: AsrMethod,
    pub prayer_adjustments: PrayerAdjustments,
    pub show_arabic: bool,
    pub use_24h: bool,
    pub volume: f32,
    #[cfg(feature = "hijri")]
    pub show_hijri: bool,
    #[serde(default)]
    pub window_pos: Option<(i32, i32)>,
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
            let temp_path = path.with_extension("tmp");
            if std::fs::write(&temp_path, &content).is_ok() {
                let _ = std::fs::rename(temp_path, path);
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod autostart {
    use std::env;
    use winreg::RegKey;
    use winreg::enums::*;

    const REG_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const APP_NAME: &str = "Athan";

    fn get_cmd() -> Option<String> {
        let exe = env::current_exe().ok()?;
        Some(format!("\"{}\" --minimized", exe.display()))
    }

    pub fn is_enabled() -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(REG_KEY)
            && let Ok(val) = key.get_value::<String, _>(APP_NAME)
            && let Some(cmd) = get_cmd()
        {
            return val == cmd;
        }
        false
    }

    pub fn set_enabled(enable: bool) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if enable {
            if let Some(cmd) = get_cmd()
                && let Ok((key, _)) = hkcu.create_subkey(REG_KEY)
            {
                let _ = key.set_value(APP_NAME, &cmd);
            }
        } else {
            if let Ok(key) = hkcu.open_subkey_with_flags(REG_KEY, KEY_SET_VALUE) {
                let _ = key.delete_value(APP_NAME);
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod autostart {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn desktop_file_path() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|dirs| dirs.config_dir().join("autostart").join("athan.desktop"))
    }

    pub fn is_enabled() -> bool {
        desktop_file_path().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn set_enabled(enable: bool) {
        if let Some(path) = desktop_file_path() {
            if enable {
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(exe) = env::current_exe() {
                    let content = format!(
                        "[Desktop Entry]\n\
                        Type=Application\n\
                        Name=Athan\n\
                        Exec=\"{}\" --minimized\n\
                        Terminal=false\n\
                        Hidden=false\n",
                        exe.display()
                    );
                    let _ = fs::write(path, content);
                }
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod autostart {
    pub fn is_enabled() -> bool {
        false
    }
    pub fn set_enabled(_enable: bool) {}
}

pub fn is_autostart() -> bool {
    autostart::is_enabled()
}

pub fn set_autostart(enabled: bool) {
    autostart::set_enabled(enabled);
}
