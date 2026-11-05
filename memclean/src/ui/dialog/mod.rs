pub mod common;
pub mod tab_kernel;
pub mod tab_system;
pub mod tab_triggers;

use crate::config::AppConfig;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    InvalidateRect, LineTo, MoveToEx, RoundRect, SelectObject, SetBkMode, SetTextColor,
    PAINTSTRUCT, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, PS_SOLID, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetDlgCtrlID,
    GetDlgItem, GetMessageW, IsDialogMessageW, IsWindow, LoadIconW, RegisterClassW, SendMessageW,
    ShowWindow, BS_OWNERDRAW, MSG, SW_HIDE, SW_SHOW, WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG,
    WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DESTROY, WM_DRAWITEM, WM_ERASEBKGND, WM_PAINT,
    WM_SETFONT, WM_SETICON, WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use self::common::{
    bg_brush, border_pen, card_brush, font_bold, font_subheader, font_ui,
    free_resources, init_resources, ODS_SELECTED,
};

#[repr(C)]
#[allow(non_snake_case, dead_code)]
struct DRAWITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemAction: u32,
    pub itemState: u32,
    pub hwndItem: HWND,
    pub hDC: windows_sys::Win32::Graphics::Gdi::HDC,
    pub rcItem: RECT,
    pub itemData: usize,
}

static mut CURRENT_TAB: usize = 0;

unsafe fn switch_tab(hwnd: HWND, new_tab: usize) {
    CURRENT_TAB = new_tab;
    for &id in tab_system::CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 0 { SW_SHOW } else { SW_HIDE });
        }
    }
    for &id in tab_triggers::CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 1 { SW_SHOW } else { SW_HIDE });
        }
    }
    for &id in tab_kernel::CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 2 { SW_SHOW } else { SW_HIDE });
        }
    }

    // Invalidate tab buttons to redraw active states
    for id in 601..=603 {
        let btn = GetDlgItem(hwnd, id);
        if !btn.is_null() {
            InvalidateRect(btn, std::ptr::null(), 1);
        }
    }

    InvalidateRect(hwnd, std::ptr::null(), 1);
}

/// Displays the MemClean Settings modal dialog window.
pub fn show_settings_dialog(_parent: HWND) {
    unsafe {
        let class_name = windows_sys::w!("MemCleanSettingsDialog");
        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(dialog_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: LoadIconW(std::ptr::null_mut(), windows_sys::Win32::UI::WindowsAndMessaging::IDI_APPLICATION),
            hCursor: windows_sys::Win32::UI::WindowsAndMessaging::LoadCursorW(
                std::ptr::null_mut(),
                windows_sys::Win32::UI::WindowsAndMessaging::IDC_ARROW,
            ),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            0,
            class_name,
            windows_sys::w!("MemClean - Settings & Optimization Policy"),
            (WS_OVERLAPPEDWINDOW & !(windows_sys::Win32::UI::WindowsAndMessaging::WS_MAXIMIZEBOX | windows_sys::Win32::UI::WindowsAndMessaging::WS_THICKFRAME)) | WS_VISIBLE,
            windows_sys::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT,
            windows_sys::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT,
            500,
            500,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if hwnd.is_null() {
            return;
        }

        let big_icon = crate::ui::tray::load_embedded_icon(32);
        let sm_icon = crate::ui::tray::load_embedded_icon(16);
        if !big_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 1, big_icon as LPARAM);
        }
        if !sm_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 0, sm_icon as LPARAM);
        }

        init_resources();

        // Top Navigation Tab Buttons (BS_OWNERDRAW)
        let create_tab_btn = |text: *const u16, cx: i32, cy: i32, cw: i32, id: i32| -> HWND {
            let btn = CreateWindowExW(
                0,
                windows_sys::w!("BUTTON"),
                text,
                WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                cx, cy, cw, 28, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut(),
            );
            let f_ui = font_ui();
            if !btn.is_null() && !f_ui.is_null() {
                SendMessageW(btn, WM_SETFONT, f_ui as WPARAM, 1);
            }
            btn
        };

        create_tab_btn(windows_sys::w!("Tray & System"), 16, 14, 145, 601);
        create_tab_btn(windows_sys::w!("Auto-Triggers"), 169, 14, 145, 602);
        create_tab_btn(windows_sys::w!("Kernel & Gaming"), 322, 14, 146, 603);

        // Instantiate controls for each tab page component
        tab_system::create(hwnd);
        tab_triggers::create(hwnd);
        tab_kernel::create(hwnd);

        // Load configuration into controls
        let cfg = AppConfig::load();
        tab_system::load(hwnd, &cfg);
        tab_triggers::load(hwnd, &cfg);
        tab_kernel::load(hwnd, &cfg);

        // Switch to Tab 0 as initial view
        switch_tab(hwnd, 0);

        // Flat Minimalist Apply & Close button (BS_OWNERDRAW)
        let btn = CreateWindowExW(
            0,
            windows_sys::w!("BUTTON"),
            windows_sys::w!("Apply and Close"),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            16, 420, 452, 36, hwnd, 301 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        let f_bold = font_bold();
        if !btn.is_null() && !f_bold.is_null() {
            SendMessageW(btn, WM_SETFONT, f_bold as WPARAM, 1);
        }
        ShowWindow(hwnd, SW_SHOW);

        let mut msg: MSG = std::mem::zeroed();
        while IsWindow(hwnd) != 0 && GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &msg) == 0 {
                DispatchMessageW(&msg);
            }
        }
    }
}

unsafe extern "system" fn dialog_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            FillRect(hdc, &client_rc, bg_brush());

            SetBkMode(hdc, TRANSPARENT as i32);
            let old_font = SelectObject(hdc, font_subheader());
            let old_brush = SelectObject(hdc, card_brush());
            let old_pen = SelectObject(hdc, border_pen());

            // Draw Cards based on CURRENT_TAB
            match CURRENT_TAB {
                0 => {
                    // Card 1: Tray Icon Appearance (16, 54, 468, 226)
                    RoundRect(hdc, 16, 54, 468, 226, 8, 8);
                    SetTextColor(hdc, 0x00747778);
                    let mut h1_rc = RECT { left: 30, top: 64, right: 454, bottom: 78 };
                    DrawTextW(hdc, windows_sys::w!("TRAY ICON STYLE"), -1, &mut h1_rc, DT_SINGLELINE);

                    MoveToEx(hdc, 30, 156, std::ptr::null_mut());
                    LineTo(hdc, 454, 156);

                    let mut hc_rc = RECT { left: 30, top: 162, right: 454, bottom: 176 };
                    DrawTextW(hdc, windows_sys::w!("COLOR PALETTE"), -1, &mut hc_rc, DT_SINGLELINE);

                    // Card 2: System Behavior & Startup (16, 238, 468, 404)
                    RoundRect(hdc, 16, 238, 468, 404, 8, 8);
                    let mut hb_rc = RECT { left: 30, top: 248, right: 454, bottom: 262 };
                    DrawTextW(hdc, windows_sys::w!("SYSTEM BEHAVIOR && STARTUP"), -1, &mut hb_rc, DT_SINGLELINE);
                }
                1 => {
                    // Card 1: RAM & Periodic Timer (16, 54, 468, 226)
                    RoundRect(hdc, 16, 54, 468, 226, 8, 8);
                    SetTextColor(hdc, 0x00747778);
                    let mut ht_rc = RECT { left: 30, top: 64, right: 454, bottom: 78 };
                    DrawTextW(hdc, windows_sys::w!("RAM AUTO-OPTIMIZATION THRESHOLD"), -1, &mut ht_rc, DT_SINGLELINE);

                    MoveToEx(hdc, 30, 136, std::ptr::null_mut());
                    LineTo(hdc, 454, 136);

                    let mut htm_rc = RECT { left: 30, top: 144, right: 454, bottom: 158 };
                    DrawTextW(hdc, windows_sys::w!("PERIODIC AUTO-CLEAN TIMER"), -1, &mut htm_rc, DT_SINGLELINE);

                    // Card 2: ISLC & Idle Triggers (16, 238, 468, 404)
                    RoundRect(hdc, 16, 238, 468, 404, 8, 8);
                    let mut hislc_rc = RECT { left: 30, top: 248, right: 454, bottom: 262 };
                    DrawTextW(hdc, windows_sys::w!("INTELLIGENT STANDBY LIST CLEANER (ISLC)"), -1, &mut hislc_rc, DT_SINGLELINE);

                    MoveToEx(hdc, 30, 326, std::ptr::null_mut());
                    LineTo(hdc, 454, 326);

                    let mut hidle_rc = RECT { left: 30, top: 334, right: 454, bottom: 348 };
                    DrawTextW(hdc, windows_sys::w!("USER INACTIVITY && AFK TRIGGER"), -1, &mut hidle_rc, DT_SINGLELINE);
                }
                _ => {
                    // Card 1: Windows Scheduler Timer Resolution (16, 54, 468, 226)
                    RoundRect(hdc, 16, 54, 468, 226, 8, 8);
                    SetTextColor(hdc, 0x00747778);
                    let mut hres_rc = RECT { left: 30, top: 64, right: 454, bottom: 78 };
                    DrawTextW(hdc, windows_sys::w!("SCHEDULER TIMER RESOLUTION (SUB-MILLISECOND PRECISION)"), -1, &mut hres_rc, DT_SINGLELINE);

                    // Card 2: Memory De-duplication & Network (16, 238, 468, 412)
                    RoundRect(hdc, 16, 238, 468, 412, 8, 8);
                    let mut hpage_rc = RECT { left: 30, top: 246, right: 454, bottom: 258 };
                    DrawTextW(hdc, windows_sys::w!("MEMORY DE-DUPLICATION (NT KERNEL)"), -1, &mut hpage_rc, DT_SINGLELINE);

                    MoveToEx(hdc, 30, 328, std::ptr::null_mut());
                    LineTo(hdc, 454, 328);

                    let mut hnet_rc = RECT { left: 30, top: 334, right: 454, bottom: 346 };
                    DrawTextW(hdc, windows_sys::w!("COMPETITIVE NETWORK STACK (LOW PING)"), -1, &mut hnet_rc, DT_SINGLELINE);
                }
            }

            SelectObject(hdc, old_brush);
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_font);
            EndPaint(hwnd, &ps);
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            let ctrl_id = GetDlgCtrlID(lparam as HWND);
            if ctrl_id == 208 || ctrl_id == 213 || ctrl_id == 233 || ctrl_id == 251 || ctrl_id == 252 || ctrl_id == 262 {
                SetTextColor(hdc, 0x00747778); // Muted helper text
            } else {
                SetTextColor(hdc, 0x00111111);
            }
            card_brush() as LRESULT // Controls match white card surface
        }
        WM_CTLCOLORBTN => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00111111);
            card_brush() as LRESULT
        }
        WM_CTLCOLORDLG => {
            bg_brush() as LRESULT
        }
        WM_CTLCOLOREDIT => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00111111);
            card_brush() as LRESULT
        }
        WM_DRAWITEM => {
            let dis = &*(lparam as *const DRAWITEMSTRUCT);
            if dis.CtlID >= 601 && dis.CtlID <= 603 {
                let tab_idx = (dis.CtlID - 601) as usize;
                let is_active = tab_idx == CURRENT_TAB;
                let bg_color = if is_active { 0x00111111 } else { 0x00FFFFFF };
                let border_color = if is_active { 0x00111111 } else { 0x00EAEAEA };
                let text_color = if is_active { 0x00FFFFFF } else { 0x00787774 };

                let brush = CreateSolidBrush(bg_color);
                let pen = CreatePen(PS_SOLID as i32, 1, border_color);
                let old_brush = SelectObject(dis.hDC, brush);
                let old_pen = SelectObject(dis.hDC, pen);
                RoundRect(dis.hDC, dis.rcItem.left, dis.rcItem.top, dis.rcItem.right, dis.rcItem.bottom, 6, 6);
                SelectObject(dis.hDC, old_pen);
                SelectObject(dis.hDC, old_brush);
                DeleteObject(pen);
                DeleteObject(brush);

                SetBkMode(dis.hDC, TRANSPARENT as i32);
                SetTextColor(dis.hDC, text_color);
                let old_font = SelectObject(dis.hDC, if is_active { font_bold() } else { font_ui() });

                let title = match tab_idx {
                    0 => windows_sys::w!("Tray & System"),
                    1 => windows_sys::w!("Auto-Triggers"),
                    _ => windows_sys::w!("Kernel & Gaming"),
                };
                let mut rc = dis.rcItem;
                DrawTextW(dis.hDC, title, -1, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                SelectObject(dis.hDC, old_font);
                1
            } else if dis.CtlID == 301 {
                let is_selected = (dis.itemState & ODS_SELECTED) != 0;
                let bg_color = if is_selected { 0x002E2E2E } else { 0x00111111 };
                let brush = CreateSolidBrush(bg_color);
                let pen = CreatePen(PS_SOLID as i32, 1, bg_color);
                let old_brush = SelectObject(dis.hDC, brush);
                let old_pen = SelectObject(dis.hDC, pen);
                RoundRect(dis.hDC, dis.rcItem.left, dis.rcItem.top, dis.rcItem.right, dis.rcItem.bottom, 6, 6);
                SelectObject(dis.hDC, old_pen);
                SelectObject(dis.hDC, old_brush);
                DeleteObject(pen);
                DeleteObject(brush);

                SetBkMode(dis.hDC, TRANSPARENT as i32);
                SetTextColor(dis.hDC, 0x00FFFFFF);
                let old_font = SelectObject(dis.hDC, font_bold());
                let text = windows_sys::w!("Apply and Close");
                let mut rc = dis.rcItem;
                DrawTextW(dis.hDC, text, -1, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                SelectObject(dis.hDC, old_font);
                1
            } else {
                0
            }
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as u32;
            if id >= 601 && id <= 603 {
                switch_tab(hwnd, (id - 601) as usize);
                return 0;
            }

            let mut cfg = AppConfig::load();

            // Delegate to tab_kernel for command actions like Restore Network Defaults (ID 261)
            if tab_kernel::handle_command(hwnd, id as i32, &mut cfg) {
                return 0;
            }

            if id == 301 {
                tab_system::save(hwnd, &mut cfg);
                tab_triggers::save(hwnd, &mut cfg);
                tab_kernel::save(hwnd, &mut cfg);

                let _ = cfg.save();
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            free_resources();
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_control_ids_are_disjoint() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for &id in tab_system::CTRLS {
            assert!(seen.insert(id), "Duplicate ID {} in tab_system::CTRLS", id);
        }
        for &id in tab_triggers::CTRLS {
            assert!(seen.insert(id), "Duplicate ID {} in tab_triggers::CTRLS", id);
        }
        for &id in tab_kernel::CTRLS {
            assert!(seen.insert(id), "Duplicate ID {} in tab_kernel::CTRLS", id);
        }

        // Tab buttons and action button must not collide with content IDs
        for id in [301, 601, 602, 603] {
            assert!(!seen.contains(&id), "Button ID {} collides with control ID", id);
        }
    }

    #[test]
    fn test_all_expected_controls_mapped() {
        let expected = [
            101, 102, 103, 104, 105, 201, 202, 204, 215, 216, 203, 214, // Tab 0
            205, 206, 207, 208, 209, 210, 211, 212, 213, // Tab 1 threshold & timer
            220, 221, 222, 223, 224, 225, 226, 230, 231, 232, 233, // Tab 1 islc & idle
            240, 241, 242, 243, 250, 270, 275, 251, 260, 261, 262, // Tab 2 kernel, de-dup, vram, cpu & network
        ];
        for &id in &expected {
            let in_0 = tab_system::CTRLS.contains(&id);
            let in_1 = tab_triggers::CTRLS.contains(&id);
            let in_2 = tab_kernel::CTRLS.contains(&id);
            assert!(in_0 || in_1 || in_2, "Expected control ID {} not mapped in any tab", id);
        }
    }
}
