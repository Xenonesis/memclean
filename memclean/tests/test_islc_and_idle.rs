use memclean::core::memory::ExtendedMemoryStats;
use memclean::engine::idle::{get_user_idle_seconds, should_idle_clean};
use memclean::engine::islc::should_islc_purge;

#[test]
fn test_islc_dual_conditions() {
    let mut stats = ExtendedMemoryStats {
        total_phys_mb: 16384,
        used_phys_mb: 8000,
        free_phys_mb: 800,
        zeroed_phys_mb: 100,
        standby_total_mb: 5000,
        standby_by_priority_mb: [0; 8],
        modified_mb: 50,
        used_percent: 50,
    };

    // Free total = 900 <= 1024, Standby = 5000 >= 1024 -> Trigger!
    assert!(should_islc_purge(&stats, 1024, 1024));

    // Abundant free RAM (2000 + 100 = 2100 > 1024) -> No trigger
    stats.free_phys_mb = 2000;
    assert!(!should_islc_purge(&stats, 1024, 1024));

    // Low standby RAM (500 < 1024) -> No trigger
    stats.free_phys_mb = 800;
    stats.standby_total_mb = 500;
    assert!(!should_islc_purge(&stats, 1024, 1024));

    // Zero threshold disables trigger
    assert!(!should_islc_purge(&stats, 0, 1024));
    assert!(!should_islc_purge(&stats, 1024, 0));
}

#[test]
#[allow(unused_comparisons)]
fn test_idle_clean_logic() {
    let idle_sec = get_user_idle_seconds();
    assert!(idle_sec >= 0);

    // If threshold is 0, never clean
    assert!(!should_idle_clean(0, false));

    // If already cleaned in this idle session, never clean again until reset
    assert!(!should_idle_clean(1, true));
}
