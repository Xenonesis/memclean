use crate::config::AppConfig;
use crate::core::memory;
use crate::engine::cleaner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject, SelectObject,
    SetBkMode, SetTextColor, TextOutW, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    FW_BOLD, TRANSPARENT,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, SetTimer, TrackPopupMenu, ICONINFO, MSG, TPM_BOTTOMALIGN,
    TPM_RIGHTALIGN, WM_COMMAND, WM_DESTROY, WM_RBUTTONUP, WM_TIMER, WM_USER, WNDCLASSW,
};

const WM_TRAYICON: u32 = WM_USER + 1;
const TIMER_ID: usize = 1001;

fn create_percent_icon(percent: u32) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
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
        let hbm_mask = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);

        let old_bm = SelectObject(hdc, hbm_color);

        // Color based on memory usage: Green < 70%, Yellow 70-85%, Red > 85%
        let text_color = if percent < 70 {
            0x002E7D32 // Green
        } else if percent < 85 {
            0x00F57C00 // Orange
        } else {
            0x00C62828 // Red
        };

        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, text_color);

        let font = CreateFontW(
            12, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 0, 0, 0, 0, 0,
            windows_sys::w!("Arial"),
        );
        let old_font = SelectObject(hdc, font);

        let text = format!("{:02}", percent.min(99));
        let wide: Vec<u16> = text.encode_utf16().collect();
        TextOutW(hdc, 0, 1, wide.as_ptr(), wide.len() as i32);

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

pub fn run_tray_application() {
    unsafe {
        let class_name = windows_sys::w!("MemCleanTrayClass");
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(tray_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name,
            windows_sys::w!("MemClean"),
            0,
            0, 0, 0, 0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

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
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = hicon;

        let tip = format!("MemClean: {}% RAM Used\0", stats.used_percent);
        let tip_wide: Vec<u16> = tip.encode_utf16().collect();
        let copy_len = tip_wide.len().min(nid.szTip.len());
        nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

        Shell_NotifyIconW(NIM_ADD, &nid);

        // Update timer every 2 seconds
        SetTimer(hwnd, TIMER_ID, 2000, None);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            DispatchMessageW(&msg);
        }

        Shell_NotifyIconW(NIM_DELETE, &nid);
        DestroyIcon(hicon);
    }
}

unsafe extern "system" fn tray_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            if lparam as u32 == WM_RBUTTONUP {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                SetForegroundWindow(hwnd);
                show_tray_menu(hwnd, pt.x, pt.y);
            }
            0
        }
        WM_TIMER => {
            if let Ok(stats) = memory::get_memory_stats() {
                let config = AppConfig::load();
                if config.triggers.auto_threshold_enabled && stats.used_percent >= config.triggers.threshold_percent {
                    cleaner::perform_clean(&config, false);
                }

                let new_icon = create_percent_icon(stats.used_percent);
                let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = 1;
                nid.uFlags = NIF_ICON | NIF_TIP;
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
                101 => { // Quick Clean
                    let config = AppConfig::load();
                    cleaner::perform_clean(&config, false);
                }
                102 => { // Purge Standby
                    let _ = crate::core::ntdll::purge_standby_list();
                }
                103 => { // Empty Working Sets
                    let _ = crate::core::ntdll::empty_system_working_sets();
                }
                104 => { // Settings
                    crate::ui::dialog::show_settings_dialog(hwnd);
                }
                105 => { // Exit
                    DestroyWindow(hwnd);
                }
                _ => {}
            }
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
    AppendMenuW(menu, MF_STRING, 101, windows_sys::w!("Quick Optimize"));
    AppendMenuW(menu, MF_STRING, 102, windows_sys::w!("Purge Standby List"));
    AppendMenuW(menu, MF_STRING, 103, windows_sys::w!("Empty Working Sets"));
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, 104, windows_sys::w!("Settings..."));
    AppendMenuW(menu, MF_STRING, 105, windows_sys::w!("Exit"));

    TrackPopupMenu(menu, TPM_RIGHTALIGN | TPM_BOTTOMALIGN, x, y, 0, hwnd, std::ptr::null());
    DestroyMenu(menu);
}
