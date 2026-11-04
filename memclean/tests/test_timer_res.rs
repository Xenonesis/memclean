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

    let res_half = guard.set_mode(TimerResolutionMode::HalfMs);
    assert!(res_half.is_ok());
    assert_eq!(guard.current_mode(), TimerResolutionMode::HalfMs);

    let cur_half = TimerResolutionGuard::get_current_resolution_ms();
    assert!(cur_half <= 0.6);

    guard.revert();
    assert_eq!(guard.current_mode(), TimerResolutionMode::Default);
}
