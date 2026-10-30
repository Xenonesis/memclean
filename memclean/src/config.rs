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

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeneralConfig {
    pub run_on_startup: bool,
    pub start_minimized: bool,
    pub tray_update_interval_sec: u32,
    #[serde(default = "default_true")]
    pub global_hotkey_enabled: bool,
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

fn default_timer_seconds() -> u32 {
    15
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TriggersConfig {
    pub auto_threshold_enabled: bool,
    pub threshold_percent: u32,
    pub auto_timer_enabled: bool,
    #[serde(default = "default_timer_seconds")]
    pub timer_seconds: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timer_minutes: Option<u32>,
}

pub fn clamp_threshold_percent(percent: u32) -> u32 {
    percent.clamp(10, 95)
}

pub fn clamp_timer_seconds(sec: u32) -> u32 {
    if sec == 0 {
        0
    } else {
        sec.clamp(1, 300)
    }
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
                global_hotkey_enabled: true,
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
                timer_seconds: 15,
                timer_minutes: None,
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
                if let Ok(mut cfg) = toml::from_str::<AppConfig>(&content) {
                    if let Some(min) = cfg.triggers.timer_minutes {
                        if !content.contains("timer_seconds") {
                            cfg.triggers.timer_seconds = min * 60;
                        }
                    }
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
#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn missing_hotkey_setting_defaults_to_enabled_and_explicit_false_round_trips() {
        let mut config = AppConfig::default();
        config.general.global_hotkey_enabled = false;
        let serialized = toml::to_string(&config).unwrap();
        let restored: AppConfig = toml::from_str(&serialized).unwrap();
        assert!(!restored.general.global_hotkey_enabled);

        let legacy = serialized.replace("global_hotkey_enabled = false\n", "");
        let restored: AppConfig = toml::from_str(&legacy).unwrap();
        assert!(restored.general.global_hotkey_enabled);
    }

    #[test]
    fn test_threshold_percent_customization_and_clamping() {
        assert_eq!(super::clamp_threshold_percent(80), 80);
        assert_eq!(super::clamp_threshold_percent(5), 10);
        assert_eq!(super::clamp_threshold_percent(99), 95);
        assert_eq!(super::clamp_threshold_percent(72), 72);

        let mut config = AppConfig::default();
        config.triggers.threshold_percent = 75;
        let serialized = toml::to_string(&config).unwrap();
        let restored: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(restored.triggers.threshold_percent, 75);
    }

    #[test]
    fn test_timer_seconds_customization_and_clamping() {
        assert_eq!(super::clamp_timer_seconds(0), 0); // continuous
        assert_eq!(super::clamp_timer_seconds(1), 1);
        assert_eq!(super::clamp_timer_seconds(15), 15);
        assert_eq!(super::clamp_timer_seconds(300), 300);
        assert_eq!(super::clamp_timer_seconds(999), 300);

        let mut config = AppConfig::default();
        config.triggers.timer_seconds = 0;
        let serialized = toml::to_string(&config).unwrap();
        let restored: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(restored.triggers.timer_seconds, 0);

        let legacy = r#"
exclusions = ["explorer.exe"]

[general]
run_on_startup = true
start_minimized = true
tray_update_interval_sec = 2
global_hotkey_enabled = true

[policy]
preset = "SmartSafe"
purge_standby = true
purge_low_priority_standby = false
flush_modified = true
trim_file_cache = true
trim_working_sets = true
game_guard_enabled = true

[triggers]
auto_threshold_enabled = true
threshold_percent = 80
auto_timer_enabled = true
timer_minutes = 5

[tray]
style = "Floating"
color_mode = "Dynamic"
click_action = "OpenDashboard"
"#;
        let restored_legacy: AppConfig = toml::from_str(legacy).unwrap();
        assert_eq!(restored_legacy.triggers.timer_minutes, Some(5));
    }
}
