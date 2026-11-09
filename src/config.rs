use cpal::traits::{DeviceTrait, HostTrait};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AudioSettings {
    pub output_device: String,
    pub exclusive_mode: bool,
    pub buffer_ms: u32,
    pub gapless_playback: bool,
    pub crossfade_sec: f32,
    pub fade_pause_resume_ms: u32,
    pub volume_leveling: String,
    pub smart_target_lufs: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            output_device: "System default".to_string(),
            exclusive_mode: false,
            buffer_ms: 120,
            gapless_playback: true,
            crossfade_sec: 0.0,
            fade_pause_resume_ms: 180,
            volume_leveling: "off".to_string(),
            smart_target_lufs: -14.0,
        }
    }
}

impl AudioSettings {
    pub const BUFFER_MIN_MS: u32 = 20;
    pub const BUFFER_MAX_MS: u32 = 500;
    pub const CROSSFADE_MIN_SEC: f32 = 0.0;
    pub const CROSSFADE_MAX_SEC: f32 = 12.0;
    pub const FADE_PAUSE_RESUME_MIN_MS: u32 = 0;
    pub const FADE_PAUSE_RESUME_MAX_MS: u32 = 500;
    pub const SMART_LUFS_MIN: f32 = -24.0;
    pub const SMART_LUFS_MAX: f32 = -9.0;
    pub const VOLUME_LEVELING_MODES: [&'static str; 4] = ["off", "rg_track", "rg_album", "smart_lufs"];
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct EqualizerSettings {
    pub enabled: bool,
    pub preamp_db: f32,
    pub bands: [f32; 10],
    pub preset: String,
}

impl Default for EqualizerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            preamp_db: 0.0,
            bands: [0.0; 10],
            preset: "Flat".to_string(),
        }
    }
}

impl EqualizerSettings {
    pub const BAND_MIN_DB: f32 = -12.0;
    pub const BAND_MAX_DB: f32 = 12.0;
    pub const PREAMP_MIN_DB: f32 = -12.0;
    pub const PREAMP_MAX_DB: f32 = 12.0;
    pub const FREQUENCIES: [&'static str; 10] = [
        "32Hz", "64Hz", "125Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "16kHz",
    ];
    pub const PRESETS: [&'static str; 5] = ["Flat", "Bass Boost", "Vocal", "Harman", "V-Shape"];

    pub fn apply_preset(&mut self, preset_name: &str) {
        match preset_name {
            "Flat" => {
                self.bands = [0.0; 10];
                self.preamp_db = 0.0;
                self.preset = "Flat".to_string();
            }
            "Bass Boost" => {
                self.bands = [5.5, 4.5, 3.2, 1.8, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0];
                self.preamp_db = -2.5;
                self.preset = "Bass Boost".to_string();
            }
            "Vocal" => {
                self.bands = [-1.5, -1.0, 0.0, 1.5, 3.5, 4.0, 2.5, 1.0, 0.0, -1.0];
                self.preamp_db = 0.0;
                self.preset = "Vocal".to_string();
            }
            "Harman" => {
                self.bands = [4.5, 3.5, 2.0, 0.5, 0.0, 0.5, 1.5, 2.5, 2.0, 1.0];
                self.preamp_db = -1.5;
                self.preset = "Harman".to_string();
            }
            "V-Shape" => {
                self.bands = [5.0, 3.5, 1.0, -1.5, -2.5, -2.0, -0.5, 1.5, 3.5, 4.5];
                self.preamp_db = -2.0;
                self.preset = "V-Shape".to_string();
            }
            "Custom" => {
                self.preset = "Custom".to_string();
            }
            other => {
                self.preset = other.to_string();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AppearanceSettings {
    pub theme: String,
    pub accent_color: [u8; 3],
    pub glass_opacity: f32,
    pub compact_sidebar: bool,
    pub show_spectrum_default: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "Eclipse".to_string(),
            accent_color: [179, 136, 255],
            glass_opacity: 0.85,
            compact_sidebar: false,
            show_spectrum_default: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LibrarySettings {
    pub music_folders: Vec<String>,
    pub scan_on_startup: bool,
    pub ignore_short_clips: bool,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            music_folders: default_music_folders(),
            scan_on_startup: true,
            ignore_short_clips: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LyricsSettings {
    pub auto_fetch_lrclib: bool,
    pub save_lrc_locally: bool,
    pub synced_autoscroll: bool,
    pub lyrics_font_size: f32,
}

impl Default for LyricsSettings {
    fn default() -> Self {
        Self {
            auto_fetch_lrclib: true,
            save_lrc_locally: true,
            synced_autoscroll: true,
            lyrics_font_size: 18.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct StreamingSettings {
    pub show_spotify: bool,
    pub show_youtube_music: bool,
    pub enable_streaming: bool,
}

impl Default for StreamingSettings {
    fn default() -> Self {
        Self {
            show_spotify: true,
            show_youtube_music: true,
            enable_streaming: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct ShortcutsSettings {
    pub play_pause: String,
    pub next_track: String,
    pub prev_track: String,
    pub volume_up: String,
    pub volume_down: String,
    pub mute: String,
    pub search: String,
    pub toggle_sidebar: String,
    pub toggle_fullscreen: String,
    pub toggle_lyrics: String,
}

impl Default for ShortcutsSettings {
    fn default() -> Self {
        Self {
            play_pause: "Space".to_string(),
            next_track: "Ctrl+Right".to_string(),
            prev_track: "Ctrl+Left".to_string(),
            volume_up: "Ctrl+Up".to_string(),
            volume_down: "Ctrl+Down".to_string(),
            mute: "M".to_string(),
            search: "Ctrl+F".to_string(),
            toggle_sidebar: "Ctrl+B".to_string(),
            toggle_fullscreen: "F11".to_string(),
            toggle_lyrics: "L".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    // Top-level fields maintained for backwards compatibility
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

    // Modular settings sub-structs
    pub audio: AudioSettings,
    pub equalizer: EqualizerSettings,
    pub appearance: AppearanceSettings,
    pub library: LibrarySettings,
    pub lyrics: LyricsSettings,
    pub streaming: StreamingSettings,
    pub shortcuts: ShortcutsSettings,
}

pub type AppSettings = AppConfig;

pub fn default_music_folders() -> Vec<String> {
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
    folders
}

impl Default for AppConfig {
    fn default() -> Self {
        let folders = default_music_folders();
        let theme = "Eclipse".to_string();
        let accent_color = [179, 136, 255]; // Radiant Neon Violet #b388ff (Purple Glass)
        let glass_opacity = 0.85;
        let scan_on_startup = true;

        Self {
            theme: theme.clone(),
            accent_color,
            volume: 0.8,
            muted: false,
            shuffle: false,
            repeat: RepeatMode::Off,
            music_folders: folders.clone(),
            visualizer_style: VisualizerStyle::Bars,
            visualizer_bands: 48,
            lyrics_offset_ms: 0,
            glass_opacity,
            scan_on_startup,
            resume_session: true,
            last_played_track_id: None,
            last_played_position_secs: 0.0,

            audio: AudioSettings::default(),
            equalizer: EqualizerSettings::default(),
            appearance: AppearanceSettings {
                theme,
                accent_color,
                glass_opacity,
                compact_sidebar: false,
                show_spectrum_default: true,
            },
            library: LibrarySettings {
                music_folders: folders,
                scan_on_startup,
                ignore_short_clips: false,
            },
            lyrics: LyricsSettings::default(),
            streaming: StreamingSettings::default(),
            shortcuts: ShortcutsSettings::default(),
        }
    }
}

pub fn app_data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(dirs_home)
        .unwrap_or_else(|| PathBuf::from("."));
    let eclipse_dir = base.join("Eclipse");
    let umbra_dir = base.join("Umbra");

    // Seamless migration from legacy Umbra directory if Eclipse doesn't exist yet
    if !eclipse_dir.exists() && umbra_dir.exists() {
        let _ = fs::create_dir_all(&eclipse_dir);
        for item in ["settings.json", "config.json", "library.db"] {
            let src = umbra_dir.join(item);
            let dst = eclipse_dir.join(item);
            if src.is_file() && !dst.exists() {
                let _ = fs::copy(&src, &dst);
            }
        }
    } else {
        let _ = fs::create_dir_all(&eclipse_dir);
    }
    eclipse_dir
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

pub fn list_audio_devices() -> Vec<String> {
    let mut devices = vec!["System default".to_string()];
    let host = cpal::default_host();
    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(name) = device.name() {
                if !devices.contains(&name) {
                    devices.push(name);
                }
            }
        }
    }
    devices
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        app_data_dir().join("settings.json")
    }

    pub fn normalize(&mut self) {
        // If loading config with previous [255, 178, 92] default, normalize to [179, 136, 255]
        if self.accent_color == [255, 178, 92] || self.appearance.accent_color == [255, 178, 92] {
            self.accent_color = [179, 136, 255];
            self.appearance.accent_color = [179, 136, 255];
        }

        // Synchronize appearance
        if self.appearance.theme != "Eclipse" && self.theme == "Eclipse" {
            self.theme = self.appearance.theme.clone();
        } else {
            self.appearance.theme = self.theme.clone();
        }

        if self.appearance.accent_color != [179, 136, 255] && self.accent_color == [179, 136, 255] {
            self.accent_color = self.appearance.accent_color;
        } else {
            self.appearance.accent_color = self.accent_color;
        }

        if (self.appearance.glass_opacity - 0.85).abs() > 0.001 && (self.glass_opacity - 0.85).abs() <= 0.001 {
            self.glass_opacity = self.appearance.glass_opacity;
        } else {
            self.appearance.glass_opacity = self.glass_opacity;
        }

        // Synchronize library
        if !self.music_folders.is_empty() && self.library.music_folders.is_empty() {
            self.library.music_folders = self.music_folders.clone();
        } else if !self.library.music_folders.is_empty() && self.music_folders.is_empty() {
            self.music_folders = self.library.music_folders.clone();
        } else {
            self.library.music_folders = self.music_folders.clone();
        }
        self.library.scan_on_startup = self.scan_on_startup;
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&content) {
                    let old_accent = cfg.accent_color;
                    cfg.normalize();
                    if old_accent == [255, 178, 92] {
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
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut to_save = self.clone();
        to_save.normalize();
        let json = serde_json::to_string_pretty(&to_save)?;
        fs::write(path, json)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut to_save = self.clone();
        to_save.normalize();
        let json = serde_json::to_string_pretty(&to_save)?;
        fs::write(path, json)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut cfg: AppConfig = serde_json::from_str(&content)?;
        cfg.normalize();
        Ok(cfg)
    }

    pub fn set_theme(&mut self, theme: impl Into<String>) {
        let t = theme.into();
        self.theme = t.clone();
        self.appearance.theme = t;
    }

    pub fn set_accent_color(&mut self, rgb: [u8; 3]) {
        self.accent_color = rgb;
        self.appearance.accent_color = rgb;
    }

    pub fn set_glass_opacity(&mut self, opacity: f32) {
        self.glass_opacity = opacity;
        self.appearance.glass_opacity = opacity;
    }

    pub fn set_music_folders(&mut self, folders: Vec<String>) {
        self.music_folders = folders.clone();
        self.library.music_folders = folders;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_app_config_values() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.theme, "Eclipse");
        assert_eq!(cfg.accent_color, [179, 136, 255]);
        assert_eq!(cfg.volume, 0.8);
        assert!(!cfg.muted);
        assert!(!cfg.shuffle);
        assert_eq!(cfg.repeat, RepeatMode::Off);
        assert_eq!(cfg.visualizer_style, VisualizerStyle::Bars);
        assert_eq!(cfg.visualizer_bands, 48);
        assert_eq!(cfg.lyrics_offset_ms, 0);
        assert!((cfg.glass_opacity - 0.85).abs() < 1e-6);
        assert!(cfg.scan_on_startup);
        assert!(cfg.resume_session);

        // Sub-structs
        assert_eq!(cfg.audio.output_device, "System default");
        assert!(!cfg.audio.exclusive_mode);
        assert_eq!(cfg.audio.buffer_ms, 120);
        assert!(cfg.audio.gapless_playback);
        assert!((cfg.audio.crossfade_sec - 0.0).abs() < 1e-6);
        assert_eq!(cfg.audio.fade_pause_resume_ms, 180);
        assert_eq!(cfg.audio.volume_leveling, "off");
        assert!((cfg.audio.smart_target_lufs - (-14.0)).abs() < 1e-6);

        assert!(!cfg.equalizer.enabled);
        assert!((cfg.equalizer.preamp_db - 0.0).abs() < 1e-6);
        assert_eq!(cfg.equalizer.bands, [0.0; 10]);
        assert_eq!(cfg.equalizer.preset, "Flat");

        assert_eq!(cfg.appearance.theme, "Eclipse");
        assert_eq!(cfg.appearance.accent_color, [179, 136, 255]);
        assert!((cfg.appearance.glass_opacity - 0.85).abs() < 1e-6);
        assert!(!cfg.appearance.compact_sidebar);
        assert!(cfg.appearance.show_spectrum_default);

        assert!(cfg.library.scan_on_startup);
        assert!(!cfg.library.ignore_short_clips);

        assert!(cfg.lyrics.auto_fetch_lrclib);
        assert!(cfg.lyrics.save_lrc_locally);
        assert!(cfg.lyrics.synced_autoscroll);
        assert!((cfg.lyrics.lyrics_font_size - 18.0).abs() < 1e-6);

        assert!(cfg.streaming.show_spotify);
        assert!(cfg.streaming.show_youtube_music);
        assert!(!cfg.streaming.enable_streaming);

        assert_eq!(cfg.shortcuts.play_pause, "Space");
        assert_eq!(cfg.shortcuts.next_track, "Ctrl+Right");
        assert_eq!(cfg.shortcuts.prev_track, "Ctrl+Left");
        assert_eq!(cfg.shortcuts.volume_up, "Ctrl+Up");
        assert_eq!(cfg.shortcuts.volume_down, "Ctrl+Down");
        assert_eq!(cfg.shortcuts.mute, "M");
        assert_eq!(cfg.shortcuts.search, "Ctrl+F");
        assert_eq!(cfg.shortcuts.toggle_sidebar, "Ctrl+B");
        assert_eq!(cfg.shortcuts.toggle_fullscreen, "F11");
        assert_eq!(cfg.shortcuts.toggle_lyrics, "L");
    }

    #[test]
    fn test_equalizer_presets() {
        let mut eq = EqualizerSettings::default();
        assert_eq!(eq.preset, "Flat");

        eq.apply_preset("Bass Boost");
        assert_eq!(eq.preset, "Bass Boost");
        assert!(eq.bands[0] > 4.0);
        assert!(eq.preamp_db < 0.0);

        eq.apply_preset("Vocal");
        assert_eq!(eq.preset, "Vocal");
        assert!(eq.bands[5] > 3.0);

        eq.apply_preset("Harman");
        assert_eq!(eq.preset, "Harman");
        assert!(eq.bands[0] > 3.0);

        eq.apply_preset("V-Shape");
        assert_eq!(eq.preset, "V-Shape");
        assert!(eq.bands[0] > 3.0);
        assert!(eq.bands[4] < 0.0);
        assert!(eq.bands[9] > 3.0);

        let bands_before = eq.bands;
        eq.apply_preset("Custom");
        assert_eq!(eq.preset, "Custom");
        assert_eq!(eq.bands, bands_before);

        eq.apply_preset("Flat");
        assert_eq!(eq.preset, "Flat");
        assert_eq!(eq.bands, [0.0; 10]);
        assert_eq!(eq.preamp_db, 0.0);
    }

    #[test]
    fn test_serialization_round_trip() {
        let mut cfg = AppConfig::default();
        cfg.audio.buffer_ms = 250;
        cfg.equalizer.enabled = true;
        cfg.equalizer.apply_preset("Harman");
        cfg.appearance.compact_sidebar = true;
        cfg.lyrics.lyrics_font_size = 20.0;
        cfg.streaming.enable_streaming = true;

        let json = serde_json::to_string_pretty(&cfg).expect("Serialization failed");
        let decoded: AppConfig = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(decoded.audio.buffer_ms, 250);
        assert!(decoded.equalizer.enabled);
        assert_eq!(decoded.equalizer.preset, "Harman");
        assert!(decoded.appearance.compact_sidebar);
        assert!((decoded.lyrics.lyrics_font_size - 20.0).abs() < 1e-6);
        assert!(decoded.streaming.enable_streaming);
    }

    #[test]
    fn test_backward_compatibility_deserialization() {
        let legacy_json = r#"{
            "theme": "Obsidian",
            "accent_color": [100, 181, 246],
            "volume": 0.65,
            "muted": true,
            "shuffle": true,
            "repeat": "All",
            "music_folders": ["C:\\CustomMusic"],
            "visualizer_style": "Mirror",
            "visualizer_bands": 32,
            "lyrics_offset_ms": 150,
            "glass_opacity": 0.9,
            "scan_on_startup": false,
            "resume_session": false,
            "last_played_track_id": 42,
            "last_played_position_secs": 123.4
        }"#;

        let mut decoded: AppConfig = serde_json::from_str(legacy_json).expect("Legacy deserialization failed");
        decoded.normalize();

        assert_eq!(decoded.theme, "Obsidian");
        assert_eq!(decoded.accent_color, [100, 181, 246]);
        assert!((decoded.volume - 0.65).abs() < 1e-6);
        assert!(decoded.muted);
        assert!(decoded.shuffle);
        assert_eq!(decoded.repeat, RepeatMode::All);
        assert_eq!(decoded.music_folders, vec!["C:\\CustomMusic".to_string()]);
        assert_eq!(decoded.visualizer_style, VisualizerStyle::Mirror);
        assert_eq!(decoded.visualizer_bands, 32);

        // Verify sub-struct defaults and synced fields
        assert_eq!(decoded.appearance.theme, "Obsidian");
        assert_eq!(decoded.appearance.accent_color, [100, 181, 246]);
        assert!((decoded.appearance.glass_opacity - 0.9).abs() < 1e-6);
        assert_eq!(decoded.library.music_folders, vec!["C:\\CustomMusic".to_string()]);
        assert!(!decoded.library.scan_on_startup);
        assert_eq!(decoded.audio.output_device, "System default");
        assert_eq!(decoded.audio.buffer_ms, 120);
        assert!(!cfg_is_equalizer_enabled(&decoded));
    }

    fn cfg_is_equalizer_enabled(cfg: &AppConfig) -> bool {
        cfg.equalizer.enabled
    }

    #[test]
    fn test_list_audio_devices() {
        let devices = list_audio_devices();
        assert!(!devices.is_empty(), "Audio devices list must not be empty");
        assert_eq!(devices[0], "System default", "First device must be 'System default'");

        // Check for deduplication
        let mut seen = std::collections::HashSet::new();
        for dev in &devices {
            assert!(seen.insert(dev), "Duplicate device found: {}", dev);
        }
    }

    #[test]
    fn test_save_and_load_from_path() {
        let temp_dir = std::env::temp_dir().join("eclipse_test_config");
        let temp_file = temp_dir.join("test_settings.json");

        let mut cfg = AppConfig::default();
        cfg.set_theme("Nebula");
        cfg.set_accent_color([105, 240, 174]);
        cfg.audio.buffer_ms = 180;
        cfg.audio.exclusive_mode = true;
        cfg.equalizer.enabled = true;
        cfg.equalizer.apply_preset("Bass Boost");

        cfg.save_to_path(&temp_file).expect("Failed to save to temp path");
        assert!(temp_file.exists());

        let loaded = AppConfig::load_from_path(&temp_file).expect("Failed to load from temp path");
        assert_eq!(loaded.theme, "Nebula");
        assert_eq!(loaded.appearance.theme, "Nebula");
        assert_eq!(loaded.accent_color, [105, 240, 174]);
        assert_eq!(loaded.appearance.accent_color, [105, 240, 174]);
        assert_eq!(loaded.audio.buffer_ms, 180);
        assert!(loaded.audio.exclusive_mode);
        assert!(loaded.equalizer.enabled);
        assert_eq!(loaded.equalizer.preset, "Bass Boost");

        let _ = fs::remove_file(&temp_file);
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_app_settings_type_alias() {
        let settings: AppSettings = AppConfig::default();
        assert_eq!(settings.theme, "Eclipse");
    }
}
