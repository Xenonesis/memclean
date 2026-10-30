use memclean::config::AppConfig;
use memclean::core::memory::get_extended_memory_stats;
use memclean::engine::cleaner::{combine_pages_only, perform_clean, purge_standby_only, CleanResult};

#[test]
fn test_perform_clean_with_combine() {
    let mut cfg = AppConfig::default();
    cfg.tuning.combine_pages_on_clean = true;
    cfg.policy.purge_standby = false;
    cfg.policy.flush_modified = false;
    cfg.policy.trim_file_cache = false;
    cfg.policy.trim_working_sets = false;

    let res: CleanResult = perform_clean(&cfg, false);
    // Should run combine step without panic
    assert_eq!(res.processes_trimmed, 0);
    // Verify fields exist
    let _ = res.pages_combined;
    let _ = res.mb_combined;
}

#[test]
fn test_extended_stats_json_serialization() {
    let stats = get_extended_memory_stats();
    assert!(stats.is_ok());
    let s = stats.unwrap();
    let json_str = serde_json::to_string(&s).expect("Serialization to JSON should succeed");
    assert!(json_str.contains("total_phys_mb"));
    assert!(json_str.contains("standby_total_mb"));
}

#[test]
fn test_cleaner_helpers_and_result_fields() {
    let _ = purge_standby_only();
    let _ = combine_pages_only();
    let res = CleanResult::default();
    assert_eq!(res.pages_combined, 0);
    assert_eq!(res.mb_combined, 0.0);
}

#[test]
fn test_cli_help_flag() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_memclean"))
        .arg("--help")
        .output()
        .expect("Failed to execute memclean with --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MemClean - Ultra-Lightweight Pure Rust Memory Optimizer"));
    assert!(stdout.contains("--stats"));
    assert!(stdout.contains("--combine"));
    assert!(stdout.contains("--standby"));
    assert!(stdout.contains("--timer-res"));
}

#[test]
fn test_cli_stats_json_flag() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_memclean"))
        .args(["--stats", "--json"])
        .output()
        .expect("Failed to execute memclean with --stats --json");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"timestamp\""));
    assert!(stdout.contains("\"memory\""));
    assert!(stdout.contains("\"timer_resolution\""));
    assert!(stdout.contains("\"top_consumers\""));

    // Validate it is valid JSON
    let val: serde_json::Value = serde_json::from_str(&stdout).expect("Stdout must be valid JSON");
    assert!(val.get("memory").is_some());
    assert!(val.get("timer_resolution").is_some());
}

#[test]
fn test_cli_silent_flag() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_memclean"))
        .args(["--silent", "--stats"])
        .output()
        .expect("Failed to execute memclean with --silent --stats");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty());
}
