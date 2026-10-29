#[test]
fn test_enable_privileges() {
    let result = memclean::core::privileges::enable_memory_privileges();
    assert!(result.is_ok(), "Privilege acquisition failed: {:?}", result);
}