use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualizerStyle {
    Bars,
    Mirror,
    Radial,
    Scope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub accent_color: [u8; 3],
    pub volume: f32,
    pub muted: bool,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub music_folders: Vec<String>,
    pub visualizer_style: VisualizerStyle,
    pub visualizer_bands: usize,
    pub lyrics_offset_ms: i64,
    pub glass_opacity: f32,
    pub scan_on_startup: bool,
    pub resume_session: bool,
    pub last_played_track_id: Option<i64>,
    pub last_played_position_secs: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut folders = Vec::new();
        let muse_path = Path::new(r"C:\music\Muse");
        if muse_path.exists() {
            folders.push(muse_path.to_string_lossy().to_string());
        } else if let Some(user_dirs) = dirs_home() {
            let music = user_dirs.join("Music");
            if music.exists() {
                folders.push(music.to_string_lossy().to_string());
            }
        }

        Self {
            theme: "Eclipse".to_string(),
            accent_color: [179, 136, 255], // Radiant Neon Violet #b388ff (Purple Glass)
            volume: 0.8,
            muted: false,
            shuffle: false,
            repeat: RepeatMode::Off,
            music_folders: folders,
            visualizer_style: VisualizerStyle::Bars,
            visualizer_bands: 48,
            lyrics_offset_ms: 0,
            glass_opacity: 0.85,
            scan_on_startup: true,
            resume_session: true,
            last_played_track_id: None,
            last_played_position_secs: 0.0,
        }
    }
}

pub fn app_data_dir() -> PathBuf {
    let dir = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(dirs_home)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Umbra");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        app_data_dir().join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&content) {
                    // If loading config with previous [255, 178, 92] default, normalize to [179, 136, 255]
                    // so the purple glass theme takes effect immediately.
                    if cfg.accent_color == [255, 178, 92] {
                        cfg.accent_color = [179, 136, 255];
                        let _ = cfg.save();
                    }
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }
}

