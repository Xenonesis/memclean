use crate::config::AppConfig;
use crate::core::memory;
use crate::engine::cleaner;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, FillRect, GetStockObject, SelectObject, SetBkMode,
    SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, FW_BOLD,
    TRANSPARENT, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO,
    NIM_ADD, NIM_DELETE, NIM_MODIFY,
};
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow,
    DispatchMessageW, FindWindowW, GetCursorPos, GetDlgItem, GetMessageW,
    GetSystemMetrics, IsWindowVisible, PostQuitMessage, RegisterClassW, SendMessageW,
    SetForegroundWindow, SetTimer, SetWindowTextW, ShowWindow, TrackPopupMenu,
    BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_GROUPBOX, BS_PUSHBUTTON,
    ICONINFO, MSG, SC_MINIMIZE,
    SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_RESTORE, SW_SHOW, TPM_BOTTOMALIGN, TPM_RIGHTALIGN,
    WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP,
    WM_SETFONT, WM_SYSCOMMAND, WM_TIMER, WM_USER, WNDCLASSW, WS_CAPTION, WS_CHILD,
    WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
};

const WM_TRAYICON: u32 = WM_USER + 1;
const TIMER_ID: usize = 1001;

pub fn create_percent_icon(percent: u32) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
    unsafe {
        let hdc = CreateCompatibleDC(std::ptr::null_mut());

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = 16;
        bmi.bmiHeader.biHeight = 16;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbm_color = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);

        // 1-bit monochrome mask (16x16 bits = 16 rows of 2 bytes = 32 bytes of 0 -> 0 is opaque)
        let mask_bytes = [0u8; 32];
        let hbm_mask = CreateBitmap(16, 16, 1, 1, mask_bytes.as_ptr() as _);

        let old_bm = SelectObject(hdc, hbm_color);

        // High-contrast badge background color (BGR format):
        let bg_color = if percent < 70 {
            0x00327D2E // Green (RGB: 46, 125, 50)
        } else if percent < 85 {
            0x000051E6 // Orange (RGB: 230, 81, 0)
        } else {
            0x002828C6 // Red (RGB: 198, 40, 40)
        };

        let brush = CreateSolidBrush(bg_color);
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 16,
            bottom: 16,
        };
        FillRect(hdc, &rect, brush);
        DeleteObject(brush);

        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, 0x00FFFFFF); // Pure white text

        let font = CreateFontW(
            -11, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 0, 0, 0, 0, 0,
            windows_sys::w!("Arial"),
        );
        let old_font = SelectObject(hdc, font);

        let text = format!("{:02}", percent.min(99));
        let wide: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(hdc, wide.as_ptr(), wide.len() as i32, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        SelectObject(hdc, old_font);
        DeleteObject(font);
        SelectObject(hdc, old_bm);
        DeleteDC(hdc);

        let mut icon_info = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let icon = CreateIconIndirect(&mut icon_info);
        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);

        icon
    }
}

const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;
const DEFAULT_GUI_FONT: i32 = 17;

unsafe fn set_text(hwnd: HWND, id: i32, text: &str) {
    let ctrl = GetDlgItem(hwnd, id);
    if !ctrl.is_null() {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        SetWindowTextW(ctrl, wide.as_ptr());
    }
}

unsafe fn set_checked(hwnd: HWND, id: i32, checked: bool) {
    let ctrl = GetDlgItem(hwnd, id);
    if !ctrl.is_null() {
        let state = if checked { BST_CHECKED } else { BST_UNCHECKED };
        SendMessageW(ctrl, BM_SETCHECK, state as WPARAM, 0);
    }
}

unsafe fn is_checked(hwnd: HWND, id: i32) -> bool {
    let ctrl = GetDlgItem(hwnd, id);
    if !ctrl.is_null() {
        SendMessageW(ctrl, BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT
    } else {
        false
    }
}

unsafe fn update_stats_display(hwnd: HWND) {
    if let Ok(stats) = memory::get_memory_stats() {
        let used_str = format!("RAM Used: {}% ({} MB / {} MB)", stats.used_percent, stats.used_phys_mb, stats.total_phys_mb);
        let avail_str = format!("Free / Available: {} MB", stats.avail_phys_mb);
        set_text(hwnd, 101, &used_str);
        set_text(hwnd, 102, &avail_str);
    }
}

unsafe fn load_config_to_controls(hwnd: HWND) {
    let config = AppConfig::load();
    set_checked(hwnd, 301, config.policy.purge_standby);
    set_checked(hwnd, 302, config.policy.purge_low_priority_standby);
    set_checked(hwnd, 303, config.policy.flush_modified);
    set_checked(hwnd, 304, config.policy.trim_file_cache);
    set_checked(hwnd, 305, config.policy.trim_working_sets);
    set_checked(hwnd, 306, config.policy.game_guard_enabled);
    set_checked(hwnd, 307, config.triggers.auto_threshold_enabled);
}

unsafe fn save_config_from_controls(hwnd: HWND) {
    let mut config = AppConfig::load();
    config.policy.purge_standby = is_checked(hwnd, 301);
    config.policy.purge_low_priority_standby = is_checked(hwnd, 302);
    config.policy.flush_modified = is_checked(hwnd, 303);
    config.policy.trim_file_cache = is_checked(hwnd, 304);
    config.policy.trim_working_sets = is_checked(hwnd, 305);
    config.policy.game_guard_enabled = is_checked(hwnd, 306);
    config.triggers.auto_threshold_enabled = is_checked(hwnd, 307);
    let _ = config.save();
    set_text(hwnd, 103, "Status: Settings saved successfully.");
}

pub fn run_tray_application() {
    run_application(false);
}

pub fn run_application(start_minimized: bool) {
    unsafe {
        let class_name = windows_sys::w!("MemCleanMainClass");
        let window_title = windows_sys::w!("MemClean - Memory Optimizer");

        // Check if another instance is already running
        let existing = FindWindowW(class_name, window_title);
        if !existing.is_null() {
            ShowWindow(existing, SW_RESTORE);
            SetForegroundWindow(existing);
            return;
        }

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(main_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: 16 as _, // (COLOR_BTNFACE + 1)
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wc);

        let width = 420;
        let height = 530;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let hwnd = CreateWindowExW(
            0,
            class_name,
            window_title,
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            x, y, width, height,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if hwnd.is_null() {
            return;
        }

        let hfont = GetStockObject(DEFAULT_GUI_FONT);

        let create_ctrl = |class: *const u16, text: *const u16, style: u32, cx: i32, cy: i32, cw: i32, ch: i32, id: i32| -> HWND {
            let ctrl = CreateWindowExW(
                0, class, text,
                WS_CHILD | WS_VISIBLE | style,
                cx, cy, cw, ch, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut()
            );
            if !ctrl.is_null() && !hfont.is_null() {
                SendMessageW(ctrl, WM_SETFONT, hfont as WPARAM, 1);
            }
            ctrl
        };

        // Memory statistics group box & labels
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!(" Physical RAM Statistics "), BS_GROUPBOX as u32, 16, 10, 372, 115, 0);
        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!("RAM Used: Calculating..."), 0, 32, 34, 340, 22, 101);
        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!("Free / Available: Calculating..."), 0, 32, 60, 340, 22, 102);
        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!("Status: Ready"), 0, 32, 88, 340, 24, 103);

        // Action buttons
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Optimize Memory Now"), BS_PUSHBUTTON as u32, 16, 136, 185, 38, 201);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Purge Standby"), BS_PUSHBUTTON as u32, 208, 136, 180, 38, 202);

        // Options group box & checkboxes
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!(" Optimization Options "), BS_GROUPBOX as u32, 16, 186, 372, 226, 0);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Purge Standby Page List"), BS_AUTOCHECKBOX as u32, 32, 210, 340, 22, 301);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Low-Priority Standby Purge"), BS_AUTOCHECKBOX as u32, 32, 236, 340, 22, 302);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Flush Modified Page List"), BS_AUTOCHECKBOX as u32, 32, 262, 340, 22, 303);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim System File Cache"), BS_AUTOCHECKBOX as u32, 32, 288, 340, 22, 304);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim Background Working Sets"), BS_AUTOCHECKBOX as u32, 32, 314, 340, 22, 305);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Enable Game & Foreground Shield"), BS_AUTOCHECKBOX as u32, 32, 340, 340, 22, 306);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Auto-optimize when RAM exceeds 80%"), BS_AUTOCHECKBOX as u32, 32, 366, 340, 22, 307);

        // Footer action buttons
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Save Settings"), BS_PUSHBUTTON as u32, 16, 424, 114, 32, 401);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Minimize to Tray"), BS_PUSHBUTTON as u32, 140, 424, 126, 32, 402);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Exit"), BS_PUSHBUTTON as u32, 276, 424, 112, 32, 403);

        load_config_to_controls(hwnd);
        update_stats_display(hwnd);

        // Setup Tray Icon
        let stats = memory::get_memory_stats().unwrap_or(memory::MemoryStats {
            total_phys_mb: 0,
            avail_phys_mb: 0,
            used_phys_mb: 0,
            used_percent: 50,
        });

        let hicon = create_percent_icon(stats.used_percent);
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_INFO;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = hicon;
        nid.dwInfoFlags = NIIF_INFO;

        let tip = format!("MemClean: {}% RAM Used\0", stats.used_percent);
        let tip_wide: Vec<u16> = tip.encode_utf16().collect();
        let copy_len = tip_wide.len().min(nid.szTip.len());
        nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

        let title = "MemClean Active\0";
        let title_wide: Vec<u16> = title.encode_utf16().collect();
        let copy_title_len = title_wide.len().min(nid.szInfoTitle.len());
        nid.szInfoTitle[..copy_title_len].copy_from_slice(&title_wide[..copy_title_len]);

        let info = "MemClean is running. Click icon to show or right-click to optimize.\0";
        let info_wide: Vec<u16> = info.encode_utf16().collect();
        let copy_info_len = info_wide.len().min(nid.szInfo.len());
        nid.szInfo[..copy_info_len].copy_from_slice(&info_wide[..copy_info_len]);

        Shell_NotifyIconW(NIM_ADD, &nid);

        if !start_minimized {
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        }

        // 1-second update timer
        SetTimer(hwnd, TIMER_ID, 1000, None);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            DispatchMessageW(&msg);
        }

        Shell_NotifyIconW(NIM_DELETE, &nid);
        DestroyIcon(hicon);
    }
}

unsafe extern "system" fn main_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = lparam as u32;
            if event == WM_RBUTTONUP {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                SetForegroundWindow(hwnd);
                show_tray_menu(hwnd, pt.x, pt.y);
            } else if event == WM_LBUTTONDBLCLK {
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
            } else if event == WM_LBUTTONUP {
                if IsWindowVisible(hwnd) != 0 {
                    ShowWindow(hwnd, SW_HIDE);
                } else {
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                }
            }
            0
        }
        WM_TIMER => {
            if let Ok(stats) = memory::get_memory_stats() {
                let config = AppConfig::load();
                if config.triggers.auto_threshold_enabled && stats.used_percent >= config.triggers.threshold_percent {
                    cleaner::perform_clean(&config, false);
                    set_text(hwnd, 103, "Status: Auto-cleaned (exceeded threshold).");
                }

                update_stats_display(hwnd);

                let new_icon = create_percent_icon(stats.used_percent);
                let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
                nid.uCallbackMessage = WM_TRAYICON;
                nid.hIcon = new_icon;

                let tip = format!("MemClean: {}% RAM Used ({} MB Free)\0", stats.used_percent, stats.avail_phys_mb);
                let tip_wide: Vec<u16> = tip.encode_utf16().collect();
                let copy_len = tip_wide.len().min(nid.szTip.len());
                nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

                Shell_NotifyIconW(NIM_MODIFY, &nid);
                DestroyIcon(new_icon);
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as u32;
            match id {
                201 | 502 => { // Optimize Memory Now
                    let before = memory::get_memory_stats().map(|s| s.avail_phys_mb).unwrap_or(0);
                    let config = AppConfig::load();
                    let _ = cleaner::perform_clean(&config, false);
                    let after = memory::get_memory_stats().map(|s| s.avail_phys_mb).unwrap_or(0);
                    let freed = if after > before { after - before } else { 0 };
                    set_text(hwnd, 103, &format!("Status: Optimized! Freed {} MB RAM.", freed));
                    update_stats_display(hwnd);
                }
                202 | 503 => { // Purge Standby
                    let _ = crate::core::ntdll::purge_standby_list();
                    set_text(hwnd, 103, "Status: Standby list purged.");
                    update_stats_display(hwnd);
                }
                401 => { // Save Settings
                    save_config_from_controls(hwnd);
                }
                402 => { // Minimize to Tray
                    ShowWindow(hwnd, SW_HIDE);
                }
                403 | 505 => { // Exit
                    DestroyWindow(hwnd);
                }
                501 => { // Open MemClean
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                }
                504 => { // Empty Working Sets
                    let _ = crate::core::ntdll::empty_system_working_sets();
                    set_text(hwnd, 103, "Status: Working sets emptied.");
                    update_stats_display(hwnd);
                }
                _ => {}
            }
            0
        }
        WM_SYSCOMMAND => {
            let cmd = (wparam & 0xFFF0) as u32;
            if cmd == SC_MINIMIZE {
                ShowWindow(hwnd, SW_HIDE);
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_CLOSE => {
            // Minimize to tray on close
            ShowWindow(hwnd, SW_HIDE);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_tray_menu(hwnd: HWND, x: i32, y: i32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, MF_SEPARATOR, MF_STRING,
    };

    let menu = CreatePopupMenu();
    AppendMenuW(menu, MF_STRING, 501, windows_sys::w!("Open MemClean"));
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, 502, windows_sys::w!("Quick Optimize"));
    AppendMenuW(menu, MF_STRING, 503, windows_sys::w!("Purge Standby List"));
    AppendMenuW(menu, MF_STRING, 504, windows_sys::w!("Empty Working Sets"));
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, 505, windows_sys::w!("Exit"));

    TrackPopupMenu(menu, TPM_RIGHTALIGN | TPM_BOTTOMALIGN, x, y, 0, hwnd, std::ptr::null());
    DestroyMenu(menu);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_icon_not_null() {
        let icon = create_percent_icon(42);
        assert!(!icon.is_null());
    }
}
