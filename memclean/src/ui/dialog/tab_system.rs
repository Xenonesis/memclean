use crate::config::{AppConfig, TrayClickAction, TrayColorMode, TrayStyle};
use windows_sys::Win32::Foundation::HWND;

use super::common::{create_chk, create_radio, is_checked, set_checked};

pub const CTRLS: &[i32] = &[101, 102, 103, 104, 105, 201, 202, 204, 215, 216, 203, 214];

/// Creates all controls for Tab 0 (Tray & System).
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog window handle on the UI thread.
pub unsafe fn create(hwnd: HWND) {
    // --- TAB 0: TRAY & SYSTEM ---
    // Card 1: Tray Icon Style
    create_radio(
        hwnd,
        windows_sys::w!("Floating Transparent Number (Minimalist)"),
        30,
        84,
        420,
        101,
    );
    create_radio(
        hwnd,
        windows_sys::w!("Rounded Capsule Badge (Dark Matte)"),
        30,
        108,
        420,
        102,
    );
    create_radio(
        hwnd,
        windows_sys::w!("Monogram Logo Icon (Concept B)"),
        30,
        132,
        420,
        103,
    );

    // Card 1: Color Mode
    create_radio(
        hwnd,
        windows_sys::w!("Dynamic Color (Green / Amber / Red)"),
        30,
        180,
        420,
        104,
    );
    create_radio(
        hwnd,
        windows_sys::w!("Monochrome White (Distraction-Free)"),
        30,
        202,
        420,
        105,
    );

    // Card 2: Tray Click Action
    create_radio(
        hwnd,
        windows_sys::w!("Click Opens / Restores Dashboard"),
        30,
        264,
        420,
        201,
    );
    create_radio(
        hwnd,
        windows_sys::w!("Click Optimizes Memory Instantly"),
        30,
        286,
        420,
        202,
    );

    // Card 2: System Behavior & Startup
    create_chk(
        hwnd,
        windows_sys::w!("Enable Global Hotkey (Ctrl + Alt + M)"),
        30,
        318,
        420,
        204,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Enable Gaming HUD Micro-Overlay (Ctrl + Shift + O)"),
        30,
        336,
        420,
        215,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Launch MemClean automatically on Windows startup"),
        30,
        354,
        420,
        216,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Start Minimized to System Tray on Login"),
        30,
        372,
        420,
        203,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Automatically check for updates on startup"),
        30,
        390,
        420,
        214,
    );
}

/// Populates Tab 0 controls from application configuration.
///
/// # Safety
/// Caller must pass a valid dialog window handle where Tab 0 controls exist.
pub unsafe fn load(hwnd: HWND, cfg: &AppConfig) {
    match cfg.tray.style {
        TrayStyle::Floating => set_checked(hwnd, 101, true),
        TrayStyle::Capsule => set_checked(hwnd, 102, true),
        TrayStyle::Logo => set_checked(hwnd, 103, true),
    }
    match cfg.tray.color_mode {
        TrayColorMode::Dynamic => set_checked(hwnd, 104, true),
        TrayColorMode::Monochrome => set_checked(hwnd, 105, true),
    }
    match cfg.tray.click_action {
        TrayClickAction::OpenDashboard => set_checked(hwnd, 201, true),
        TrayClickAction::QuickOptimize => set_checked(hwnd, 202, true),
    }
    set_checked(hwnd, 203, cfg.general.start_minimized);
    set_checked(hwnd, 204, cfg.general.global_hotkey_enabled);
    set_checked(hwnd, 214, cfg.updates.check_on_startup);
    set_checked(hwnd, 215, cfg.general.gaming_hud_enabled);
    set_checked(hwnd, 216, cfg.general.run_on_startup);
}

/// Reads Tab 0 control states and writes them into `AppConfig`.
///
/// # Safety
/// Caller must pass a valid dialog window handle where Tab 0 controls exist.
pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig) {
    if is_checked(hwnd, 101) {
        cfg.tray.style = TrayStyle::Floating;
    } else if is_checked(hwnd, 102) {
        cfg.tray.style = TrayStyle::Capsule;
    } else if is_checked(hwnd, 103) {
        cfg.tray.style = TrayStyle::Logo;
    }

    if is_checked(hwnd, 104) {
        cfg.tray.color_mode = TrayColorMode::Dynamic;
    } else if is_checked(hwnd, 105) {
        cfg.tray.color_mode = TrayColorMode::Monochrome;
    }

    if is_checked(hwnd, 201) {
        cfg.tray.click_action = TrayClickAction::OpenDashboard;
    } else if is_checked(hwnd, 202) {
        cfg.tray.click_action = TrayClickAction::QuickOptimize;
    }

    cfg.general.start_minimized = is_checked(hwnd, 203);
    cfg.general.global_hotkey_enabled = is_checked(hwnd, 204);
    cfg.updates.check_on_startup = is_checked(hwnd, 214);
    let hud_enabled = is_checked(hwnd, 215);
    cfg.general.gaming_hud_enabled = hud_enabled;
    if hud_enabled {
        crate::ui::hud::show_hud();
    } else {
        crate::ui::hud::hide_hud();
    }
    let run_startup = is_checked(hwnd, 216);
    cfg.general.run_on_startup = run_startup;
    let _ = crate::core::process::register_autostart(run_startup);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_system_ctrls_unique_and_count() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for &id in CTRLS {
            assert!(seen.insert(id), "Duplicate ID {} in CTRLS", id);
        }
        assert_eq!(CTRLS.len(), 12);
    }
}
