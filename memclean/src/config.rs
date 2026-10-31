use crate::core::timer_res::TimerResolutionMode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum Preset {
    #[default]
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
    #[serde(default = "default_true")]
    pub run_on_startup: bool,
    #[serde(default = "default_true")]
    pub start_minimized: bool,
    #[serde(default = "default_tray_update_interval_sec")]
    pub tray_update_interval_sec: u32,
    #[serde(alias = "global_hotkey", default = "default_true")]
    pub global_hotkey_enabled: bool,
    #[serde(default = "default_true")]
    pub auto_elevate: bool,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub gaming_hud_enabled: bool,
}

fn default_tray_update_interval_sec() -> u32 {
    2
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            run_on_startup: true,
            start_minimized: true,
            tray_update_interval_sec: default_tray_update_interval_sec(),
            global_hotkey_enabled: true,
            auto_elevate: true,
            minimize_to_tray: true,
            gaming_hud_enabled: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolicyConfig {
    #[serde(default)]
    pub preset: Preset,
    #[serde(default = "default_true")]
    pub purge_standby: bool,
    #[serde(default)]
    pub purge_low_priority_standby: bool,
    #[serde(default = "default_true")]
    pub flush_modified: bool,
    #[serde(default = "default_true")]
    pub trim_file_cache: bool,
    #[serde(default = "default_true")]
    pub trim_working_sets: bool,
    #[serde(default = "default_true")]
    pub game_guard_enabled: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            preset: Preset::SmartSafe,
            purge_standby: true,
            purge_low_priority_standby: false,
            flush_modified: true,
            trim_file_cache: true,
            trim_working_sets: true,
            game_guard_enabled: true,
        }
    }
}

fn default_threshold_percent() -> u32 {
    80
}

fn default_timer_seconds() -> u32 {
    15
}

fn default_islc_mb() -> u64 {
    1024
}

fn default_idle_seconds() -> u32 {
    300 // 5 minutes
}

pub fn clamp_islc_mb(val: u64) -> u64 {
    val.clamp(128, 65536)
}

pub fn clamp_idle_seconds(val: u32) -> u32 {
    val.clamp(30, 3600)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggersConfig {
    #[serde(default = "default_true")]
    pub auto_clean_enabled: bool,
    #[serde(default = "default_true")]
    pub auto_threshold_enabled: bool,
    #[serde(default = "default_threshold_percent")]
    pub threshold_percent: u32,
    #[serde(default = "default_true")]
    pub auto_timer_enabled: bool,
    #[serde(default = "default_timer_seconds")]
    pub timer_seconds: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timer_minutes: Option<u32>,

    // ISLC Dual-Condition Trigger
    #[serde(default)]
    pub islc_enabled: bool,
    #[serde(default = "default_islc_mb")]
    pub islc_standby_mb: u64,
    #[serde(default = "default_islc_mb")]
    pub islc_free_mb: u64,

    // Inactivity / Idle Trigger
    #[serde(default)]
    pub idle_clean_enabled: bool,
    #[serde(default = "default_idle_seconds")]
    pub idle_seconds: u32,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdatesConfig {
    #[serde(default = "default_true")]
    pub check_on_startup: bool,
    #[serde(default)]
    pub last_check_timestamp: u64,
}

impl Default for UpdatesConfig {
    fn default() -> Self {
        Self {
            check_on_startup: true,
            last_check_timestamp: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct NetworkConfig {
    #[serde(default)]
    pub low_latency_enabled: bool,
}


impl Default for TriggersConfig {
    fn default() -> Self {
        Self {
            auto_clean_enabled: true,
            auto_threshold_enabled: true,
            threshold_percent: 80,
            auto_timer_enabled: true,
            timer_seconds: 15,
            timer_minutes: None,
            islc_enabled: false,
            islc_standby_mb: 1024,
            islc_free_mb: 1024,
            idle_clean_enabled: false,
            idle_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    #[serde(default)]
    pub timer_resolution_enabled: bool,
    #[serde(default = "default_timer_res_mode")]
    pub timer_resolution_mode: String, // "0.5ms", "1.0ms", or "default"
    #[serde(default)]
    pub combine_pages_on_clean: bool,
    #[serde(default)]
    pub vram_trim_on_clean: bool,
    #[serde(default)]
    pub hybrid_cpu_optimizer_enabled: bool,
    #[serde(default)]
    pub unpark_cores_enabled: bool,
}

fn default_timer_res_mode() -> String {
    "0.5ms".to_string()
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            timer_resolution_enabled: false,
            timer_resolution_mode: default_timer_res_mode(),
            combine_pages_on_clean: false,
            vram_trim_on_clean: false,
            hybrid_cpu_optimizer_enabled: false,
            unpark_cores_enabled: false,
        }
    }
}

impl TuningConfig {
    pub fn timer_mode_enum(&self) -> TimerResolutionMode {
        if !self.timer_resolution_enabled {
            return TimerResolutionMode::Default;
        }
        match self.timer_resolution_mode.trim().to_lowercase().as_str() {
            "0.5ms" | "0.5" | "half" => TimerResolutionMode::HalfMs,
            "1.0ms" | "1ms" | "1.0" | "one" => TimerResolutionMode::OneMs,
            _ => TimerResolutionMode::Default,
        }
    }
}

fn default_exclusions() -> Vec<String> {
    vec![
        "explorer.exe".to_string(),
        "steam.exe".to_string(),
        "obs64.exe".to_string(),
    ]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
enum ExclusionsHelper {
    List(Vec<String>),
    Table {
        #[serde(default)]
        processes: Vec<String>,
    },
}

fn deserialize_exclusions<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match ExclusionsHelper::deserialize(deserializer)? {
        ExclusionsHelper::List(list) => Ok(list),
        ExclusionsHelper::Table { processes } => Ok(processes),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
    #[serde(default)]
    pub triggers: TriggersConfig,
    #[serde(default)]
    pub tuning: TuningConfig,
    #[serde(default)]
    pub tray: TrayConfig,
    #[serde(default = "default_exclusions", deserialize_with = "deserialize_exclusions")]
    pub exclusions: Vec<String>,
    #[serde(default)]
    pub updates: UpdatesConfig,
    #[serde(default)]
    pub network: NetworkConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            policy: PolicyConfig::default(),
            triggers: TriggersConfig::default(),
            tuning: TuningConfig::default(),
            tray: TrayConfig::default(),
            exclusions: default_exclusions(),
            updates: UpdatesConfig::default(),
            network: NetworkConfig::default(),
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

    #[test]
    fn test_updates_config_defaults_and_roundtrip() {
        let mut config = AppConfig::default();
        assert!(config.updates.check_on_startup);
        assert_eq!(config.updates.last_check_timestamp, 0);

        config.updates.check_on_startup = false;
        config.updates.last_check_timestamp = 1730310000;
        let serialized = toml::to_string(&config).unwrap();
        let restored: AppConfig = toml::from_str(&serialized).unwrap();
        assert!(!restored.updates.check_on_startup);
        assert_eq!(restored.updates.last_check_timestamp, 1730310000);

        // Legacy config missing [updates] table
        let legacy = r#"
exclusions = ["explorer.exe"]
[general]
run_on_startup = true
"#;
        let restored_legacy: AppConfig = toml::from_str(legacy).unwrap();
        assert!(restored_legacy.updates.check_on_startup);
        assert_eq!(restored_legacy.updates.last_check_timestamp, 0);
    }


    #[test]
    fn test_network_config_defaults_and_roundtrip() {
        let mut config = AppConfig::default();
        assert!(!config.network.low_latency_enabled);

        config.network.low_latency_enabled = true;
        let serialized = toml::to_string(&config).unwrap();
        let restored: AppConfig = toml::from_str(&serialized).unwrap();
        assert!(restored.network.low_latency_enabled);

        // Legacy config missing [network] table
        let legacy = r#"
exclusions = ["explorer.exe"]
[general]
run_on_startup = true
"#;
        let restored_legacy: AppConfig = toml::from_str(legacy).unwrap();
        assert!(!restored_legacy.network.low_latency_enabled);
    }
}
