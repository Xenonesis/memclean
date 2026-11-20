use crate::config::{AppConfig, TrayClickAction, TrayColorMode, TrayStyle};
use crate::core::disk::{detect_system_drive_type, DriveType};
use windows_sys::Win32::Foundation::{HWND, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, SendMessageW, BS_OWNERDRAW, WM_SETFONT, WS_CHILD, WS_VISIBLE,
};

use super::common::{create_chk, create_radio, is_checked, set_checked};

pub const CTRLS: &[i32] = &[
    101, 102, 103, 104, 105, 201, 202, 204, 215, 216, 203, 214, 250, 251,
];

const BTN_BEST_DEVICE: i32 = 250;
const LBL_DRIVE_TYPE: i32 = 251;

/// Creates all controls for Tab 0 (General & Tray).
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog window handle on the UI thread.
pub unsafe fn create(hwnd: HWND) {
    // --- TAB 0: TRAY & SYSTEM (LEFT COLUMN: Tray Styling & Previews) ---
    // 3 Visual Preview Cards for Tray Icon Style
    let create_preview_card = |text: *const u16, x: i32, y: i32, w: i32, h: i32, id: i32| -> HWND {
        let btn = CreateWindowExW(
            0,
            windows_sys::w!("BUTTON"),
            text,
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            x,
            y,
            w,
            h,
            hwnd,
            id as _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        let f_ui = super::common::font_ui();
        if !btn.is_null() && !f_ui.is_null() {
            SendMessageW(btn, WM_SETFONT, f_ui as WPARAM, 1);
        }
        btn
    };

    create_preview_card(
        windows_sys::w!("Floating Transparent"),
        34,
        220,
        110,
        88,
        101,
    );
    create_preview_card(
        windows_sys::w!("Rounded Capsule"),
        154,
        220,
        110,
        88,
        102,
    );
    create_preview_card(
        windows_sys::w!("Monogram Logo"),
        274,
        220,
        110,
        88,
        103,
    );

    // Color Palette Mode Segmented Control
    create_radio(
        hwnd,
        windows_sys::w!("Dynamic Color"),
        34,
        348,
        172,
        104,
    );
    create_radio(
        hwnd,
        windows_sys::w!("Monochrome White"),
        214,
        348,
        172,
        105,
    );

    // Tray Click Action Segmented Control
    create_radio(
        hwnd,
        windows_sys::w!("Open Dashboard"),
        34,
        422,
        172,
        201,
    );
    create_radio(
        hwnd,
        windows_sys::w!("Quick Optimize"),
        214,
        422,
        172,
        202,
    );

    // --- RIGHT COLUMN: System Startup & Behavior Toggles ---
    create_chk(
        hwnd,
        windows_sys::w!("Launch MemClean on Windows Startup"),
        432,
        220,
        354,
        216,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Start Minimized to System Tray"),
        432,
        270,
        354,
        203,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Automatically Check for Updates"),
        432,
        320,
        354,
        214,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Enable Global Hotkey (Ctrl + Alt + M)"),
        432,
        370,
        354,
        204,
    );
    create_chk(
        hwnd,
        windows_sys::w!("Enable Gaming HUD Overlay (Ctrl + Shift + O)"),
        432,
        420,
        354,
        215,
    );

    // Drive Optimization Status Label & Best-For-Device Button
    let dt = detect_system_drive_type();
    let dt_label = match dt {
        DriveType::Ssd => windows_sys::w!("Detected Storage: Solid State Drive (SSD)  ·  Zero-Wear Policy"),
        DriveType::Hdd => windows_sys::w!("Detected Storage: Hard Disk Drive (HDD)  ·  High-Cache Flush Policy"),
        DriveType::Unknown => windows_sys::w!("Detected Storage: System Storage  ·  Balanced Safe Defaults"),
    };

    let lbl_drive = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        dt_label,
        WS_CHILD | WS_VISIBLE,
        432,
        508,
        354,
        18,
        hwnd,
        LBL_DRIVE_TYPE as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    let f_desc = super::common::font_desc();
    if !lbl_drive.is_null() && !f_desc.is_null() {
        SendMessageW(lbl_drive, WM_SETFONT, f_desc as WPARAM, 1);
    }

    let btn_device = CreateWindowExW(
        0,
        windows_sys::w!("BUTTON"),
        windows_sys::w!("⚡ Apply Best Defaults for Your Device"),
        WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
        432,
        534,
        354,
        36,
        hwnd,
        BTN_BEST_DEVICE as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    let f_bold = super::common::font_bold();
    if !btn_device.is_null() && !f_bold.is_null() {
        SendMessageW(btn_device, WM_SETFONT, f_bold as WPARAM, 1);
    }
}

/// Populates Tab 0 controls from application configuration.
///
/// # Safety
/// Caller must pass a valid dialog window handle where Tab 0 controls exist.
pub unsafe fn load(hwnd: HWND, cfg: &AppConfig) {
    match cfg.tray.style {
        TrayStyle::Floating => {
            set_checked(hwnd, 101, true);
            set_checked(hwnd, 102, false);
            set_checked(hwnd, 103, false);
        }
        TrayStyle::Capsule => {
            set_checked(hwnd, 101, false);
            set_checked(hwnd, 102, true);
            set_checked(hwnd, 103, false);
        }
        TrayStyle::Logo => {
            set_checked(hwnd, 101, false);
            set_checked(hwnd, 102, false);
            set_checked(hwnd, 103, true);
        }
    }
    match cfg.tray.color_mode {
        TrayColorMode::Dynamic => {
            set_checked(hwnd, 104, true);
            set_checked(hwnd, 105, false);
        }
        TrayColorMode::Monochrome => {
            set_checked(hwnd, 104, false);
            set_checked(hwnd, 105, true);
        }
    }
    match cfg.tray.click_action {
        TrayClickAction::OpenDashboard => {
            set_checked(hwnd, 201, true);
            set_checked(hwnd, 202, false);
        }
        TrayClickAction::QuickOptimize => {
            set_checked(hwnd, 201, false);
            set_checked(hwnd, 202, true);
        }
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
        assert_eq!(CTRLS.len(), 14);
    }
}
