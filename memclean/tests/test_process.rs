#[test]
fn test_enumerate_processes() {
    let procs = memclean::core::process::enumerate_processes().expect("Failed to enumerate processes");
    assert!(!procs.is_empty());
    assert!(procs.iter().any(|p| p.name.to_lowercase() == "explorer.exe" || p.name.to_lowercase() == "cargo.exe"));
}
