use memclean::core::memory;

#[test]
fn test_extended_memory_stats_query() {
    let stats = memory::get_extended_memory_stats();
    assert!(stats.is_ok(), "Extended memory query should succeed on Windows");
    let s = stats.unwrap();
    assert!(s.total_phys_mb > 0);
    assert!(s.used_phys_mb > 0);
    assert!(s.total_phys_mb >= s.used_phys_mb);
    assert!(s.standby_total_mb <= s.total_phys_mb);
}
