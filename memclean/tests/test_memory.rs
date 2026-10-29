#[test]
fn test_get_memory_stats() {
    let stats = memclean::core::memory::get_memory_stats().expect("Failed to get memory stats");
    assert!(stats.total_phys_mb > 0);
    assert!(stats.used_percent <= 100);
}