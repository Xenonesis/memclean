use crate::core::memory::ExtendedMemoryStats;

/// Evaluates whether the standby list should be purged based on ISLC dual conditions.
/// Triggers when `standby_total_mb >= standby_threshold_mb` AND available free/zeroed RAM `<= free_threshold_mb`.
pub fn should_islc_purge(
    stats: &ExtendedMemoryStats,
    standby_threshold_mb: u64,
    free_threshold_mb: u64,
) -> bool {
    if standby_threshold_mb == 0 || free_threshold_mb == 0 {
        return false;
    }

    let total_free_mb = stats.free_phys_mb + stats.zeroed_phys_mb;
    stats.standby_total_mb >= standby_threshold_mb && total_free_mb <= free_threshold_mb
}
