#[test]
fn test_protected_process_filtering() {
    let exclusions = vec!["customgame.exe".to_string(), "notepad.exe".to_string()];
    
    // Core system processes must be protected
    assert!(memclean::engine::guard::is_protected_process("csrss.exe", 100, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("dwm.exe", 200, None, &exclusions));
    assert!(memclean::engine::guard::is_protected_process("system", 4, None, &exclusions));

    // Foreground process must be protected
    assert!(memclean::engine::guard::is_protected_process("app.exe", 500, Some(500), &exclusions));

    // Custom exclusion list
    assert!(memclean::engine::guard::is_protected_process("customgame.exe", 600, None, &exclusions));

    // Normal non-protected process
    assert!(!memclean::engine::guard::is_protected_process("idleapp.exe", 700, Some(500), &exclusions));
}
