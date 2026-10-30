use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Preset {
    SmartSafe,
    Aggressive,
    Custom,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TrayStyle {
    Floating,
    Capsule,
    Logo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TrayColorMode {
    Dynamic,
    Monochrome,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TrayClickAction {
    OpenDashboard,
    QuickOptimize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrayConfig {
    pub style: TrayStyle,
    pub color_mode: TrayColorMode,
    pub click_action: TrayClickAction,
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            style: TrayStyle::Floating,
            color_mode: TrayColorMode::Dynamic,
            click_action: TrayClickAction::OpenDashboard,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeneralConfig {
    pub run_on_startup: bool,
    pub start_minimized: bool,
    pub tray_update_interval_sec: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolicyConfig {
    pub preset: Preset,
    pub purge_standby: bool,
    pub purge_low_priority_standby: bool,
    pub flush_modified: bool,
    pub trim_file_cache: bool,
    pub trim_working_sets: bool,
    pub game_guard_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TriggersConfig {
    pub auto_threshold_enabled: bool,
    pub threshold_percent: u32,
    pub auto_timer_enabled: bool,
    pub timer_minutes: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub policy: PolicyConfig,
    pub triggers: TriggersConfig,
    #[serde(default)]
    pub tray: TrayConfig,
    pub exclusions: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                run_on_startup: true,
                start_minimized: true,
                tray_update_interval_sec: 2,
            },
            policy: PolicyConfig {
                preset: Preset::SmartSafe,
                purge_standby: true,
                purge_low_priority_standby: false,
                flush_modified: true,
                trim_file_cache: true,
                trim_working_sets: true,
                game_guard_enabled: true,
            },
            triggers: TriggersConfig {
                auto_threshold_enabled: true,
                threshold_percent: 80,
                auto_timer_enabled: true,
                timer_minutes: 15,
            },
            tray: TrayConfig::default(),
            exclusions: vec![
                "explorer.exe".to_string(),
                "steam.exe".to_string(),
                "obs64.exe".to_string(),
            ],
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("MemClean").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str(&content) {
                    return cfg;
                }
            }
        }
        let default = Self::default();
        let _ = default.save();
        default
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let data = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| e.to_string())
    }
}
