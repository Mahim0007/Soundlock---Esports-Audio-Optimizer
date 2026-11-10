use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub run_on_startup: bool,
    pub language: String,
    pub accent_hex: String,
    pub minimize_to_tray: bool,
    pub ducking_enabled: bool,
    pub target_game: String,
    pub duck_level_percent: f32, // e.g. 20%
    pub trigger_threshold: f32,  // e.g. 0.15 (sound level)
    pub fade_out_ms: u32,        // e.g. 120ms
    pub fade_in_ms: u32,         // e.g. 250ms
    pub clutch_hotkey: String,
    pub clutch_hold_mode: bool,  // true = hold to duck, false = toggle
    pub footstep_boost_enabled: bool,
    pub footstep_boost_db: f32,
    pub footstep_preset: String,
    pub target_duck_apps: Vec<String>,
    pub custom_apps: Vec<String>,
    #[serde(default = "default_limiter_enabled")]
    pub sound_limiter_enabled: bool,
    #[serde(default = "default_limiter_ceiling")]
    pub sound_limiter_ceiling: f32, // e.g. 70.0%
}

fn default_limiter_enabled() -> bool {
    true
}

fn default_limiter_ceiling() -> f32 {
    70.0
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            run_on_startup: true,
            language: "English".to_string(),
            accent_hex: "#E3008C".to_string(), // Hot Pink/Magenta matching screenshot!
            minimize_to_tray: true,
            ducking_enabled: true,
            target_game: "Auto (Any Active Game)".to_string(),
            duck_level_percent: 20.0,
            trigger_threshold: 0.025,
            fade_out_ms: 120,
            fade_in_ms: 250,
            clutch_hotkey: "Caps Lock".to_string(),
            clutch_hold_mode: true,
            footstep_boost_enabled: true,
            footstep_boost_db: 6.0,
            footstep_preset: "Footsteps (2.5 kHz - 6.0 kHz)".to_string(),
            target_duck_apps: vec![
                "Spotify.exe".to_string(),
                "Discord.exe".to_string(),
                "chrome.exe".to_string(),
                "brave.exe".to_string(),
                "msedge.exe".to_string(),
                "firefox.exe".to_string(),
                "opera.exe".to_string(),
                "opera_gx.exe".to_string(),
                "arc.exe".to_string(),
                "Amazon Music.exe".to_string(),
                "AppleMusic.exe".to_string(),
                "vlc.exe".to_string(),
                "wmplayer.exe".to_string(),
            ],
            custom_apps: Vec::new(),
            sound_limiter_enabled: true,
            sound_limiter_ceiling: 70.0,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let mut path = dirs_fallback();
        path.push("SoundLock_Suite");
        let _ = fs::create_dir_all(&path);
        path.push("config.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&content) {
                    if cfg.language.eq_ignore_ascii_case("bengali") {
                        cfg.language = "English".to_string();
                        let _ = cfg.save();
                    }
                    if cfg.trigger_threshold > 0.035 {
                        cfg.trigger_threshold = 0.025;
                        let _ = cfg.save();
                    }
                    cfg.accent_hex = "#E3008C".to_string();
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }
}

fn dirs_fallback() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata)
    } else {
        PathBuf::from(".")
    }
}

