use memclean::core::timer_res::{TimerResolutionGuard, TimerResolutionMode};

#[test]
fn test_timer_resolution_guard_lifecycle() {
    let current_before = TimerResolutionGuard::get_current_resolution_ms();
    assert!(current_before > 0.0);

    let mut guard = TimerResolutionGuard::new();
    let res = guard.set_mode(TimerResolutionMode::OneMs);
    assert!(res.is_ok());
    assert_eq!(guard.current_mode(), TimerResolutionMode::OneMs);

    let cur = TimerResolutionGuard::get_current_resolution_ms();
    assert!(cur <= 1.05);

    guard.revert();
    assert_eq!(guard.current_mode(), TimerResolutionMode::Default);
}
