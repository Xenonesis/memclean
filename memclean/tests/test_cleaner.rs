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
