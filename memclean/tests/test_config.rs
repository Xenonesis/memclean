#[test]
fn test_default_config_serialization() {
    let config = memclean::config::AppConfig::default();
    let serialized = toml::to_string(&config).expect("Serialization failed");
    let deserialized: memclean::config::AppConfig = toml::from_str(&serialized).expect("Deserialization failed");
    assert_eq!(config.triggers.threshold_percent, deserialized.triggers.threshold_percent);
    assert_eq!(config.policy.purge_standby, deserialized.policy.purge_standby);
}
