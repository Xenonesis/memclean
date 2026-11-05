use crate::config::AppConfig;
use windows_sys::Win32::Foundation::{HWND, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, SendMessageW, WM_SETFONT, WS_CHILD, WS_VISIBLE,
};

use super::common::{create_chk, create_radio, font_desc, font_ui, is_checked, set_checked};

pub const CTRLS: &[i32] = &[240, 241, 242, 243, 251, 250, 270, 275, 260, 261, 262];

/// Creates all controls for Tab 2 (Kernel & Gaming Tuning).
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog window handle on the UI thread.
pub unsafe fn create(hwnd: HWND) {
    // --- TAB 2: KERNEL & GAMING ---
    // Card 1: Scheduler Timer Resolution
    create_radio(
        hwnd,
        windows_sys::w!("Default Timer Resolution (15.6 ms - Standard)"),
        30,
        84,
        420,
        240,
    );
    create_radio(
        hwnd,
        windows_sys::w!("1.0 ms Multimedia Latency (1,000 µs - Audio / Video Sync)"),
        30,
        106,
        420,
        241,
    );
    create_radio(
        hwnd,
        windows_sys::w!("0.5 ms Competitive Gaming (500 µs - Minimum Latency)"),
        30,
        128,
        420,
        242,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Unpark 100% CPU Cores (Zero-Latency Burst Performance)"),
        30,
        152,
        420,
        243,
    );

    let lbl_res_desc = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!(
            "Locks Windows kernel timer resolution to 0.5ms and unparks all cores for zero-stutter gaming."
        ),
        WS_CHILD | WS_VISIBLE,
        30,
        178,
        420,
        36,
        hwnd,
        251 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    let f_desc = font_desc();
    if !lbl_res_desc.is_null() && !f_desc.is_null() {
        SendMessageW(lbl_res_desc, WM_SETFONT, f_desc as WPARAM, 1);
    }

    // Card 2: Kernel Page Combining, VRAM, CPU & Network Latency
    create_chk(
        hwnd,
        windows_sys::w!("Enable Kernel Page Combining (RAM De-duplication)"),
        30,
        264,
        420,
        250,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Enable GPU / VRAM Decommitment on Clean (WDDM 2.1+)"),
        30,
        285,
        420,
        270,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Enable Hybrid CPU Scheduling (Games to P-Cores, Bloat to E-Cores)"),
        30,
        306,
        420,
        275,
    );

    create_chk(
        hwnd,
        windows_sys::w!("Enable Competitive Gaming TCP Stack (NoDelay && No Throttling)"),
        30,
        350,
        420,
        260,
    );

    let btn_net_restore = CreateWindowExW(
        0,
        windows_sys::w!("BUTTON"),
        windows_sys::w!("Restore Network Defaults"),
        WS_CHILD | WS_VISIBLE,
        30,
        374,
        165,
        22,
        hwnd,
        261 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    let f_ui = font_ui();
    if !btn_net_restore.is_null() && !f_ui.is_null() {
        SendMessageW(btn_net_restore, WM_SETFONT, f_ui as WPARAM, 1);
    }

    let lbl_net_desc = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("No delayed ACKs, 100% CPU priority."),
        WS_CHILD | WS_VISIBLE,
        205,
        377,
        245,
        18,
        hwnd,
        262 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_net_desc.is_null() && !f_desc.is_null() {
        SendMessageW(lbl_net_desc, WM_SETFONT, f_desc as WPARAM, 1);
    }
}

/// Loads configuration state into Tab 2 controls.
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog handle containing Tab 2 controls.
pub unsafe fn load(hwnd: HWND, cfg: &AppConfig) {
    match cfg.tuning.timer_mode_enum() {
        crate::core::timer_res::TimerResolutionMode::HalfMs => set_checked(hwnd, 242, true),
        crate::core::timer_res::TimerResolutionMode::OneMs => set_checked(hwnd, 241, true),
        crate::core::timer_res::TimerResolutionMode::Default => set_checked(hwnd, 240, true),
    }

    set_checked(hwnd, 243, cfg.tuning.unpark_cores_enabled);
    set_checked(hwnd, 250, cfg.tuning.combine_pages_on_clean);
    set_checked(hwnd, 270, cfg.tuning.vram_trim_on_clean);
    set_checked(hwnd, 275, cfg.tuning.hybrid_cpu_optimizer_enabled);
    set_checked(hwnd, 260, cfg.network.low_latency_enabled);
}

/// Saves Tab 2 control values back into the application configuration and applies hardware changes.
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog handle containing Tab 2 controls.
pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig) {
    if is_checked(hwnd, 242) {
        cfg.tuning.timer_resolution_enabled = true;
        cfg.tuning.timer_resolution_mode = "0.5ms".to_string();
    } else if is_checked(hwnd, 241) {
        cfg.tuning.timer_resolution_enabled = true;
        cfg.tuning.timer_resolution_mode = "1.0ms".to_string();
    } else {
        cfg.tuning.timer_resolution_enabled = false;
        cfg.tuning.timer_resolution_mode = "default".to_string();
    }

    let unpark_checked = is_checked(hwnd, 243);
    if unpark_checked != cfg.tuning.unpark_cores_enabled {
        cfg.tuning.unpark_cores_enabled = unpark_checked;
        if unpark_checked {
            let _ = crate::core::cpu::unpark_all_cores();
        } else {
            let _ = crate::core::cpu::restore_core_parking_default();
        }
    }

    cfg.tuning.combine_pages_on_clean = is_checked(hwnd, 250);
    cfg.tuning.vram_trim_on_clean = is_checked(hwnd, 270);
    cfg.tuning.hybrid_cpu_optimizer_enabled = is_checked(hwnd, 275);

    let net_opt_checked = is_checked(hwnd, 260);
    if net_opt_checked != cfg.network.low_latency_enabled {
        cfg.network.low_latency_enabled = net_opt_checked;
        if net_opt_checked {
            let _ = crate::core::network::optimize_network();
        } else {
            let _ = crate::core::network::restore_network_defaults();
        }
    }
}

/// Handles Tab 2 specific command actions like "Restore Network Defaults" (ID 261).
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog handle.
pub unsafe fn handle_command(hwnd: HWND, id: i32, cfg: &mut AppConfig) -> bool {
    if id == 261 {
        let _ = crate::core::network::restore_network_defaults();
        set_checked(hwnd, 260, false);
        cfg.network.low_latency_enabled = false;
        let _ = cfg.save();
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_tab_kernel_ctrls_unique_and_count() {
        assert_eq!(CTRLS.len(), 11);
        let mut set = HashSet::new();
        for &id in CTRLS {
            assert!(set.insert(id), "Duplicate control ID {id} in Tab 2 CTRLS");
        }
        assert!(CTRLS.contains(&240));
        assert!(CTRLS.contains(&241));
        assert!(CTRLS.contains(&242));
        assert!(CTRLS.contains(&243));
        assert!(CTRLS.contains(&251));
        assert!(CTRLS.contains(&250));
        assert!(CTRLS.contains(&270));
        assert!(CTRLS.contains(&275));
        assert!(CTRLS.contains(&260));
        assert!(CTRLS.contains(&261));
        assert!(CTRLS.contains(&262));
    }
}
