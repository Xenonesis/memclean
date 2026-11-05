#[test]
fn test_clean_orchestration() {
    let mut config = memclean::config::AppConfig::default();
    // Disable process trimming in test to avoid interfering with test runner
    config.policy.trim_working_sets = false;
    let res = memclean::engine::cleaner::perform_clean(&config, false);
    // Verify perform_clean executes orchestration safely without panicking.
    // Standby purge depends on OS privilege level (admin vs user).
    let _ = res;
}

#[test]
fn test_auto_clean_is_safe_and_zero_stutter() {
    let config = memclean::config::AppConfig::default();
    let res = memclean::engine::cleaner::perform_auto_clean(&config);
    // Auto-clean MUST NEVER flush modified pages or trim processes in background
    assert_eq!(res.processes_trimmed, 0);
    assert!(!res.modified_flushed);
}

#[test]
fn test_system_whitelist_protects_shell_processes() {
    let exclusions: Vec<String> = vec![];
    assert!(memclean::engine::guard::is_protected_process("explorer.exe", 9999, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("svchost.exe", 9998, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("audiodg.exe", 9997, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("dwm.exe", 9996, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("ctfmon.exe", 9995, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("sihost.exe", 9994, None, &exclusions));
}
