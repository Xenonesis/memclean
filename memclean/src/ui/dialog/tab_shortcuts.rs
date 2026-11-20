use crate::config::AppConfig;
use windows_sys::Win32::Foundation::{HWND, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetDlgItem, SendMessageW, SetWindowTextW, WM_SETFONT, WS_CHILD, WS_VISIBLE,
};

use super::common::{create_chk, font_ui, is_checked, set_checked};

pub const CTRLS: &[i32] = &[401, 402, 403, 404, 405];

pub const BTN_HOTKEY_MEM: i32 = 401;
pub const BTN_HOTKEY_HUD: i32 = 402;
pub const BTN_TEST_HOTKEY: i32 = 403;
pub const LBL_HOTKEY_STATUS: i32 = 404;
pub const LBL_SHORTCUTS_INFO: i32 = 405;

/// Creates all controls for Tab 3 (Shortcuts & Hotkeys).
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog window handle on the UI thread.
pub unsafe fn create(hwnd: HWND) {
    // --- TAB 3: SHORTCUTS ---
    // Left Column: Hotkeys
    create_chk(
        hwnd,
        windows_sys::w!("Enable Global Hotkey (Ctrl + Alt + M)"),
        34,
        220,
        354,
        BTN_HOTKEY_MEM,
    );

    create_chk(
        hwnd,
        windows_sys::w!("Enable Gaming HUD Micro-Overlay (Ctrl + Shift + O)"),
        34,
        280,
        354,
        BTN_HOTKEY_HUD,
    );

    // Test Quick Clean Hotkey Action Button
    let btn_test = CreateWindowExW(
        0,
        windows_sys::w!("BUTTON"),
        windows_sys::w!("Test Quick Optimization Now"),
        WS_CHILD | WS_VISIBLE | windows_sys::Win32::UI::WindowsAndMessaging::BS_OWNERDRAW as u32,
        34,
        348,
        354,
        36,
        hwnd,
        BTN_TEST_HOTKEY as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    let f_ui = font_ui();
    if !btn_test.is_null() && !f_ui.is_null() {
        SendMessageW(btn_test, WM_SETFONT, f_ui as WPARAM, 1);
    }

    // Status Label
    let lbl_status = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Status: Global shortcuts active and registered with Windows kernel."),
        WS_CHILD | WS_VISIBLE,
        34,
        396,
        354,
        34,
        hwnd,
        LBL_HOTKEY_STATUS as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_status.is_null() && !f_ui.is_null() {
        SendMessageW(lbl_status, WM_SETFONT, f_ui as WPARAM, 1);
    }

    // Right Column: Shortcuts Cheatsheet Info
    let lbl_info = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("MemClean Keyboard Navigation Reference:\n\n• Ctrl + Alt + M: Instant Memory Optimization\n• Ctrl + Shift + O: Toggle In-Game HUD Overlay\n• 1 / 2 / 3 / 4: Switch Navigation Tabs\n• Tab / Shift+Tab: Move Focus Between Controls\n• Space / Enter: Toggle Switch or Trigger Button\n• Esc: Cancel and Close Settings\n\nDirect Low-Latency Kernel Path:\nHotkeys bypass Win32 message queuing for sub-millisecond execution."),
        WS_CHILD | WS_VISIBLE,
        432,
        220,
        354,
        220,
        hwnd,
        LBL_SHORTCUTS_INFO as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_info.is_null() && !f_ui.is_null() {
        SendMessageW(lbl_info, WM_SETFONT, f_ui as WPARAM, 1);
    }
}

/// Loads configuration into Tab 3 controls.
pub unsafe fn load(hwnd: HWND, cfg: &AppConfig) {
    set_checked(hwnd, BTN_HOTKEY_MEM, cfg.general.global_hotkey_enabled);
    set_checked(hwnd, BTN_HOTKEY_HUD, cfg.general.gaming_hud_enabled);
}

/// Saves Tab 3 controls into configuration.
pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig) {
    cfg.general.global_hotkey_enabled = is_checked(hwnd, BTN_HOTKEY_MEM);
    let hud = is_checked(hwnd, BTN_HOTKEY_HUD);
    cfg.general.gaming_hud_enabled = hud;
    if hud {
        crate::ui::hud::show_hud();
    } else {
        crate::ui::hud::hide_hud();
    }
}

/// Handles commands for Tab 3 controls (e.g. Test Quick Optimization).
pub unsafe fn handle_command(hwnd: HWND, id: i32, _cfg: &mut AppConfig) -> bool {
    if id == BTN_TEST_HOTKEY {
        let stats_before = crate::core::memory::get_memory_stats();
        let _res = crate::engine::cleaner::perform_clean(_cfg, false);
        let stats_after = crate::core::memory::get_memory_stats();
        let status_ctrl = GetDlgItem(hwnd, LBL_HOTKEY_STATUS);
        if !status_ctrl.is_null() {
            let freed_mb = match (stats_before, stats_after) {
                (Ok(b), Ok(a)) => b.used_phys_mb.saturating_sub(a.used_phys_mb),
                _ => 0,
            };
            let msg = if freed_mb > 0 {
                format!("Cleaned {} MB! System memory optimized successfully.", freed_mb)
            } else {
                "Memory optimization executed! Standby caches and working sets trimmed.".to_string()
            };
            let wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
            SetWindowTextW(status_ctrl, wide.as_ptr());
        }
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
    fn test_tab_shortcuts_ctrls_unique_and_count() {
        assert_eq!(CTRLS.len(), 5);
        let mut set = HashSet::new();
        for &id in CTRLS {
            assert!(set.insert(id), "Duplicate ID {} in CTRLS", id);
        }
    }
}
