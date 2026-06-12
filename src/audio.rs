use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct AudioPlayer {
    _handle: MixerDeviceSink,
    player: Player,
}

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        let mut handle = DeviceSinkBuilder::open_default_sink().ok()?;
        handle.log_on_drop(false);
        let player = Player::connect_new(handle.mixer());
        Some(Self {
            _handle: handle,
            player,
        })
    }

    pub fn play(&self, path: impl AsRef<Path>) {
        if let Ok(file) = File::open(path)
            && let Ok(source) = Decoder::try_from(file)
        {
            self.player.clear();
            self.player.append(source);
            self.player.play();
        }
    }

    pub fn stop(&self) {
        self.player.clear();
    }

    pub fn set_volume(&self, volume: f32) {
        self.player.set_volume(volume);
    }

    pub fn is_playing(&self) -> bool {
        !self.player.empty() && !self.player.is_paused()
    }
}

fn try_audio_dir(base: &Path) -> Option<PathBuf> {
    let path = base.join("data").join("audio");
    path.exists().then_some(path)
}

pub fn audio_dir() -> PathBuf {
    // Current directory
    if let Ok(dir) = std::env::current_dir()
        && let Some(p) = try_audio_dir(&dir)
    {
        return p;
    }

    // Exe directory
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && let Some(p) = try_audio_dir(dir)
    {
        return p;
    }

    // Standard data directory
    if let Some(proj) = directories::ProjectDirs::from("", "", "athan") {
        let path = proj.data_dir().join("audio");
        let _ = std::fs::create_dir_all(&path);
        return path;
    }

    // Fallback
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push("data");
    path.push("audio");
    let _ = std::fs::create_dir_all(&path);
    path
}

pub fn ensure_audio_files() {
    let dest = audio_dir();
    if dest.join("adhan.ogg").exists() {
        return;
    }

    let sources = std::env::current_dir()
        .ok()
        .map(|p| p.join("data").join("audio"))
        .into_iter()
        .chain(
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("data").join("audio"))),
        );

    for src in sources {
        if src.join("adhan.ogg").exists() {
            let _ = std::fs::copy(src.join("adhan.ogg"), dest.join("adhan.ogg"));
            if src.join("fajr.ogg").exists() {
                let _ = std::fs::copy(src.join("fajr.ogg"), dest.join("fajr.ogg"));
            }
            return;
        }
    }
}
