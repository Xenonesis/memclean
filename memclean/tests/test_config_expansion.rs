use memclean::config::{clamp_idle_seconds, clamp_islc_mb, AppConfig};
use memclean::core::timer_res::TimerResolutionMode;

#[test]
fn test_legacy_config_backward_compatibility() {
    let legacy_toml = r#"
[general]
auto_elevate = true
minimize_to_tray = true
start_minimized = false
global_hotkey = true

[triggers]
auto_clean_enabled = true
threshold_percent = 80
timer_seconds = 15

[policy]
purge_standby = true
purge_low_priority_standby = false
flush_modified = true
trim_file_cache = true
trim_working_sets = true
game_guard_enabled = true

[exclusions]
processes = ["test.exe"]
"#;

    let cfg: AppConfig = toml::from_str(legacy_toml).expect("Legacy config should deserialize cleanly");
    assert!(!cfg.triggers.islc_enabled);
    assert_eq!(cfg.triggers.islc_standby_mb, 1024);
    assert_eq!(cfg.triggers.islc_free_mb, 1024);
    assert!(!cfg.triggers.idle_clean_enabled);
    assert_eq!(cfg.triggers.idle_seconds, 300);
    assert!(!cfg.tuning.timer_resolution_enabled);
    assert_eq!(cfg.tuning.timer_mode_enum(), TimerResolutionMode::Default);
}

#[test]
fn test_expanded_config_roundtrip() {
    let mut cfg = AppConfig::default();
    cfg.triggers.islc_enabled = true;
    cfg.triggers.islc_standby_mb = 2048;
    cfg.triggers.islc_free_mb = 1500;
    cfg.triggers.idle_clean_enabled = true;
    cfg.triggers.idle_seconds = 180;
    cfg.tuning.timer_resolution_enabled = true;
    cfg.tuning.timer_resolution_mode = "0.5ms".to_string();
    cfg.tuning.combine_pages_on_clean = true;

    let serialized = toml::to_string(&cfg).expect("Serialization should succeed");
    let deserialized: AppConfig = toml::from_str(&serialized).expect("Deserialization should succeed");

    assert!(deserialized.triggers.islc_enabled);
    assert_eq!(deserialized.triggers.islc_standby_mb, 2048);
    assert_eq!(deserialized.triggers.islc_free_mb, 1500);
    assert_eq!(deserialized.triggers.idle_seconds, 180);
    assert!(deserialized.tuning.timer_resolution_enabled);
    assert_eq!(deserialized.tuning.timer_mode_enum(), TimerResolutionMode::HalfMs);
    assert!(deserialized.tuning.combine_pages_on_clean);
}

#[test]
fn test_clamping_helpers() {
    assert_eq!(clamp_islc_mb(50), 128);
    assert_eq!(clamp_islc_mb(100000), 65536);
    assert_eq!(clamp_islc_mb(2048), 2048);

    assert_eq!(clamp_idle_seconds(10), 30);
    assert_eq!(clamp_idle_seconds(10000), 3600);
    assert_eq!(clamp_idle_seconds(600), 600);
}
