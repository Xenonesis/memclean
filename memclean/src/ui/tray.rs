use crate::config::{AppConfig, TrayClickAction, TrayColorMode, TrayStyle};
use crate::core::memory;
use crate::core::process::get_top_memory_processes;
use crate::engine::cleaner;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT,
    CreateBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW, CreatePen, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, FillRect, GetStockObject, LineTo, MoveToEx, RoundRect, SelectObject, SetBkMode,
    SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, FW_BOLD,
    NULL_PEN, PS_SOLID, TRANSPARENT, DT_CENTER, DT_RIGHT, DT_SINGLELINE, DT_VCENTER,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO,
    NIM_ADD, NIM_DELETE, NIM_MODIFY,
};
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconFromResourceEx, CreateIconIndirect, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow,
    DispatchMessageW, DrawIconEx, FindWindowW, GetClientRect, GetCursorPos, GetDlgItem, GetMessageW,
    IsWindowVisible, LoadIconW, PostQuitMessage, RegisterClassW, SendMessageW,
    SetForegroundWindow, SetTimer, SetWindowTextW, ShowWindow, SystemParametersInfoW, TrackPopupMenu,
    BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_OWNERDRAW,
    DI_NORMAL, GetDlgCtrlID, ICONINFO, MSG, SC_MINIMIZE, SPI_GETWORKAREA,
    SW_HIDE, SW_RESTORE, SW_SHOW, TPM_BOTTOMALIGN, TPM_RIGHTALIGN,
    WM_CLOSE, WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG, WM_CTLCOLORSTATIC, WM_DESTROY,
    WM_DRAWITEM, WM_ERASEBKGND, WM_HOTKEY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_PAINT, WM_RBUTTONUP,
    WM_SETFONT, WM_SETICON, WM_SYSCOMMAND, WM_TIMER, WM_USER, WNDCLASSW, WS_CAPTION, WS_CHILD,
    WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT};
use windows_sys::Win32::UI::WindowsAndMessaging::HICON;

const EMBEDDED_ICO: &[u8] = include_bytes!("../../memclean.ico");

pub fn load_embedded_icon(desired_size: u32) -> HICON {
    unsafe {
        if EMBEDDED_ICO.len() < 6 {
            return std::ptr::null_mut();
        }
        let count = u16::from_le_bytes([EMBEDDED_ICO[4], EMBEDDED_ICO[5]]) as usize;
        let mut best_offset = 0usize;
        let mut best_size = 0usize;
        let mut best_dim = 0u32;
        let mut best_diff = i32::MAX;

        for i in 0..count {
            let entry_off = 6 + i * 16;
            if entry_off + 16 > EMBEDDED_ICO.len() {
                break;
            }
            let raw_w = EMBEDDED_ICO[entry_off];
            let dim = if raw_w == 0 { 256 } else { raw_w as u32 };
            let size = u32::from_le_bytes([
                EMBEDDED_ICO[entry_off + 8],
                EMBEDDED_ICO[entry_off + 9],
                EMBEDDED_ICO[entry_off + 10],
                EMBEDDED_ICO[entry_off + 11],
            ]) as usize;
            let offset = u32::from_le_bytes([
                EMBEDDED_ICO[entry_off + 12],
                EMBEDDED_ICO[entry_off + 13],
                EMBEDDED_ICO[entry_off + 14],
                EMBEDDED_ICO[entry_off + 15],
            ]) as usize;

            let diff = (dim as i32 - desired_size as i32).abs();
            if diff < best_diff {
                best_diff = diff;
                best_dim = dim;
                best_offset = offset;
                best_size = size;
            }
        }

        if best_size > 0 && best_offset + best_size <= EMBEDDED_ICO.len() {
            let res_bytes = &EMBEDDED_ICO[best_offset..best_offset + best_size];
            let hicon = CreateIconFromResourceEx(
                res_bytes.as_ptr(),
                best_size as u32,
                1,
                0x00030000,
                best_dim as i32,
                best_dim as i32,
                0,
            );
            if !hicon.is_null() {
                return hicon;
            }
        }
        std::ptr::null_mut()
    }
}
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
const ODS_SELECTED: u32 = 0x0001;

static mut G_BG_BRUSH: windows_sys::Win32::Graphics::Gdi::HBRUSH = std::ptr::null_mut();
static mut G_CARD_BRUSH: windows_sys::Win32::Graphics::Gdi::HBRUSH = std::ptr::null_mut();
static mut G_BORDER_PEN: windows_sys::Win32::Graphics::Gdi::HPEN = std::ptr::null_mut();
static mut G_TRACK_BRUSH: windows_sys::Win32::Graphics::Gdi::HBRUSH = std::ptr::null_mut();
static mut G_FONT_UI: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();
static mut G_FONT_BOLD: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();
static mut G_FONT_SUBHEADER: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();
static mut G_FONT_STATS: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();
static mut G_USED_PERCENT: u32 = 0;
const WM_TRAYICON: u32 = WM_USER + 1;
const HOTKEY_ID: i32 = 2001;
const PROCESS_TIMER_ID: usize = 1002;
const TIMER_ID: usize = 1001;

pub fn create_percent_icon(percent: u32) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
    let cfg = get_config();
    create_percent_icon_with_config(percent, &cfg.tray)
}

pub fn create_percent_icon_with_config(
    percent: u32,
    tray_cfg: &crate::config::TrayConfig,
) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
    unsafe {
        if tray_cfg.style == TrayStyle::Logo {
            let logo_icon = load_embedded_icon(16);
            if !logo_icon.is_null() {
                return logo_icon;
            }
            let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
            let app_icon = LoadIconW(hinstance, 1 as _);
            if !app_icon.is_null() {
                return app_icon;
            }
        }

        let hdc = CreateCompatibleDC(std::ptr::null_mut());

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = 16;
        bmi.bmiHeader.biHeight = -16; // Top-down DIB so row 0 is top
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbm_color = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);

        let old_bm = SelectObject(hdc, hbm_color);

        // Text color in BGR format
        let text_color = match tray_cfg.color_mode {
            TrayColorMode::Monochrome => 0x00FFFFFF, // Crisp White
            TrayColorMode::Dynamic => {
                if percent < 70 {
                    0x005EC522 // Mint/Forest Green #22C55E (BGR: 0x5E, 0xC5, 0x22)
                } else if percent < 85 {
                    0x000B9EF5 // Warm Amber #F59E0B (BGR: 0x0B, 0x9E, 0xF5)
                } else {
                    0x004444EF // Carmine Coral #EF4444 (BGR: 0x44, 0x44, 0xEF)
                }
            }
        };

        if tray_cfg.style == TrayStyle::Capsule {
            // Draw a rounded capsule container
            let capsule_brush = CreateSolidBrush(0x00241E18); // Dark Slate #181E24
            let capsule_pen = CreatePen(PS_SOLID as i32, 1, 0x0041342A); // Border #2A3441
            let old_brush = SelectObject(hdc, capsule_brush);
            let old_pen = SelectObject(hdc, capsule_pen);
            RoundRect(hdc, 0, 1, 16, 15, 6, 6);
            SelectObject(hdc, old_brush);
            SelectObject(hdc, old_pen);
            DeleteObject(capsule_brush);
            DeleteObject(capsule_pen);
        }

        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, text_color);

        let font = CreateFontW(
            -11, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        let old_font = SelectObject(hdc, font);

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 16,
            bottom: 16,
        };
        let text = format!("{:02}", percent.min(99));
        let wide: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(hdc, wide.as_ptr(), wide.len() as i32, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        SelectObject(hdc, old_font);
        DeleteObject(font);

        // Build 1-bit transparency mask (1 = transparent, 0 = opaque)
        let mut mask_bytes = [0xFFu8; 32];
        if !bits.is_null() {
            let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, 256);
            for y in 0..16 {
                for x in 0..16 {
                    let idx = y * 16 + x;
                    if (pixels[idx] & 0x00FFFFFF) != 0 {
                        let byte_idx = y * 2 + (x / 8);
                        let bit_idx = 7 - (x % 8);
                        mask_bytes[byte_idx] &= !(1 << bit_idx);
                        pixels[idx] |= 0xFF000000; // Alpha 255
                    }
                }
            }
        }

        let hbm_mask = CreateBitmap(16, 16, 1, 1, mask_bytes.as_ptr() as _);

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

pub fn calculate_memory_bar_segments(
    total_mb: u64,
    used_mb: u64,
    standby_mb: u64,
    modified_mb: u64,
    bar_width: i32,
) -> (i32, i32, i32, i32) {
    if total_mb == 0 || bar_width <= 0 {
        return (0, 0, 0, bar_width.max(0));
    }
    let in_use_w = (((used_mb as u128) * (bar_width as u128)) / (total_mb as u128)) as i32;
    let standby_w = (((standby_mb as u128) * (bar_width as u128)) / (total_mb as u128)) as i32;
    let mod_w = (((modified_mb as u128) * (bar_width as u128)) / (total_mb as u128)) as i32;
    let allocated = in_use_w + standby_w + mod_w;
    let remaining_w = (bar_width - allocated).max(0);
    (in_use_w, standby_w, mod_w, remaining_w)
}

pub fn format_tray_tooltip(percent: u32, standby_mb: u64, timer_res_ms: f32) -> String {
    format!("MemClean: {}% | Standby: {}MB | Timer: {:.1}ms", percent, standby_mb, timer_res_ms)
}

unsafe fn update_stats_display(hwnd: HWND) {
    if let Ok(stats) = memory::get_extended_memory_stats() {
        if let Ok(mut lock) = G_EXT_STATS.lock() {
            *lock = Some(stats);
        }
        G_USED_PERCENT = stats.used_percent;
        let used_str = format!("RAM Used: {} MB ({}%)", stats.used_phys_mb, stats.used_percent);
        let legend_str = format!(
            "Standby: {} MB | Free: {} MB | Mod: {} MB",
            stats.standby_total_mb, stats.free_phys_mb, stats.modified_mb
        );
        set_text(hwnd, 101, &used_str);
        set_text(hwnd, 102, &legend_str);
        InvalidateRect(hwnd, std::ptr::null(), 0);
    } else if let Ok(stats) = memory::get_memory_stats() {
        G_USED_PERCENT = stats.used_percent;
        let used_str = format!("RAM Used: {} MB ({}%)", stats.used_phys_mb, stats.used_percent);
        let avail_str = format!("Free / Available: {} MB", stats.avail_phys_mb);
        set_text(hwnd, 101, &used_str);
        set_text(hwnd, 102, &avail_str);
        InvalidateRect(hwnd, std::ptr::null(), 0);
    }
}

unsafe fn update_top_processes(hwnd: HWND) {
    let top = get_top_memory_processes(5);
    if let Ok(mut cached) = TOP_PROCESSES.lock() {
        *cached = top.clone();
    }

    for index in 0..5 {
        let label_id = 105 + index as i32;
        let trim_btn_id = 601 + index as i32;
        let eco_btn_id = 701 + index as i32;
        if let Some(process) = top.get(index) {
            let name = process.name.chars().take(22).collect::<String>();
            set_text(hwnd, label_id, &format!("{}  {} MB", name, process.working_set_mb));
            ShowWindow(GetDlgItem(hwnd, trim_btn_id), SW_SHOW);
            ShowWindow(GetDlgItem(hwnd, eco_btn_id), SW_SHOW);
        } else {
            set_text(hwnd, label_id, "");
            ShowWindow(GetDlgItem(hwnd, trim_btn_id), SW_HIDE);
            ShowWindow(GetDlgItem(hwnd, eco_btn_id), SW_HIDE);
        }
    }
}
unsafe fn sync_global_hotkey(hwnd: HWND, enabled: bool) -> bool {
    UnregisterHotKey(hwnd, HOTKEY_ID);
    !enabled || RegisterHotKey(hwnd, HOTKEY_ID, MOD_ALT | MOD_CONTROL | MOD_NOREPEAT, 0x4D) != 0
}

use std::sync::Mutex;
use std::time::Instant;

static CACHED_CONFIG: Mutex<Option<AppConfig>> = Mutex::new(None);
static LAST_AUTO_CLEAN: Mutex<Option<Instant>> = Mutex::new(None);
static LAST_PERIODIC_CLEAN: Mutex<Option<Instant>> = Mutex::new(None);
static TOP_PROCESSES: Mutex<Vec<crate::core::process::ProcessMemUsage>> = Mutex::new(Vec::new());
static G_EXT_STATS: Mutex<Option<crate::core::memory::ExtendedMemoryStats>> = Mutex::new(None);
static G_TIMER_GUARD: Mutex<Option<crate::core::timer_res::TimerResolutionGuard>> = Mutex::new(None);
static G_IDLE_CLEANED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
fn get_config() -> AppConfig {
    let mut lock = CACHED_CONFIG.lock().unwrap();
    if lock.is_none() {
        *lock = Some(AppConfig::load());
    }
    lock.clone().unwrap()
}

fn update_config<F: FnOnce(&mut AppConfig)>(f: F) {
    let mut lock = CACHED_CONFIG.lock().unwrap();
    if lock.is_none() {
        *lock = Some(AppConfig::load());
    }
    if let Some(cfg) = lock.as_mut() {
        f(cfg);
        let _ = cfg.save();
    }
}

unsafe fn load_config_to_controls(hwnd: HWND) {
    let config = get_config();
    set_checked(hwnd, 301, config.policy.purge_standby);
    set_checked(hwnd, 302, config.policy.purge_low_priority_standby);
    set_checked(hwnd, 303, config.policy.flush_modified);
    set_checked(hwnd, 304, config.policy.trim_file_cache);
    set_checked(hwnd, 305, config.policy.trim_working_sets);
    set_checked(hwnd, 306, config.policy.game_guard_enabled);
    set_checked(hwnd, 307, config.triggers.auto_threshold_enabled);
    let thresh_label = format!("Auto-optimize when RAM exceeds {}%\0", config.triggers.threshold_percent);
    let thresh_wide: Vec<u16> = thresh_label.encode_utf16().collect();
    let ctrl307 = GetDlgItem(hwnd, 307);
    SetWindowTextW(ctrl307, thresh_wide.as_ptr());
    windows_sys::Win32::Graphics::Gdi::InvalidateRect(ctrl307, std::ptr::null(), 1);
}
unsafe fn save_config_from_controls(hwnd: HWND) {
    let purge_standby = is_checked(hwnd, 301);
    let purge_low_standby = is_checked(hwnd, 302);
    let flush_mod = is_checked(hwnd, 303);
    let trim_cache = is_checked(hwnd, 304);
    let trim_ws = is_checked(hwnd, 305);
    let game_guard = is_checked(hwnd, 306);
    let auto_thresh = is_checked(hwnd, 307);

    update_config(|cfg| {
        cfg.policy.purge_standby = purge_standby;
        cfg.policy.purge_low_priority_standby = purge_low_standby;
        cfg.policy.flush_modified = flush_mod;
        cfg.policy.trim_file_cache = trim_cache;
        cfg.policy.trim_working_sets = trim_ws;
        cfg.policy.game_guard_enabled = game_guard;
        cfg.triggers.auto_threshold_enabled = auto_thresh;
        cfg.policy.preset = crate::config::Preset::Custom;
    });
    set_text(hwnd, 103, "Status: Settings saved successfully.");
}

unsafe fn show_tray_notification(hwnd: HWND, title: &str, message: &str) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_INFO;
    nid.dwInfoFlags = NIIF_INFO;

    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let msg_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let copy_title_len = title_wide.len().min(nid.szInfoTitle.len());
    nid.szInfoTitle[..copy_title_len].copy_from_slice(&title_wide[..copy_title_len]);
    let copy_msg_len = msg_wide.len().min(nid.szInfo.len());
    nid.szInfo[..copy_msg_len].copy_from_slice(&msg_wide[..copy_msg_len]);

    Shell_NotifyIconW(NIM_MODIFY, &nid);
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
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
        let mut app_icon = LoadIconW(hinstance, 1 as _);
        if app_icon.is_null() {
            app_icon = load_embedded_icon(32);
        }

        G_BG_BRUSH = CreateSolidBrush(0x00FAFBFB); // Warm Bone #FBFBFA
        G_CARD_BRUSH = CreateSolidBrush(0x00FFFFFF); // Pure White #FFFFFF
        G_BORDER_PEN = CreatePen(PS_SOLID as i32, 1, 0x00EAEAEA); // Hairline #EAEAEA
        G_TRACK_BRUSH = CreateSolidBrush(0x00EEF0F0); // Subtle Gray #F0F0EE

        G_FONT_UI = CreateFontW(
            -13, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        G_FONT_BOLD = CreateFontW(
            -13, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        G_FONT_SUBHEADER = CreateFontW(
            -11, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        G_FONT_STATS = CreateFontW(
            -15, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(main_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: app_icon,
            hCursor: std::ptr::null_mut(),
            hbrBackground: G_BG_BRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wc);

        let width = 400;
        let height = 560;

        let mut work_area: RECT = std::mem::zeroed();
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            &mut work_area as *mut _ as _,
            0,
        );
        let work_w = work_area.right - work_area.left;
        let x = work_area.left + (work_w - width) / 2;
        let y = work_area.top + 40;
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

        let small_icon = load_embedded_icon(16);
        let big_icon = load_embedded_icon(32);
        if !small_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 0 as _, small_icon as _);
        } else if !app_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 0 as _, app_icon as _);
        }
        if !big_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 1 as _, big_icon as _);
        } else if !app_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 1 as _, app_icon as _);
        }

        let create_ctrl = |class: *const u16, text: *const u16, style: u32, cx: i32, cy: i32, cw: i32, ch: i32, id: i32, font: windows_sys::Win32::Graphics::Gdi::HFONT| -> HWND {
            let ctrl = CreateWindowExW(
                0, class, text,
                WS_CHILD | WS_VISIBLE | style,
                cx, cy, cw, ch, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut()
            );
            if !ctrl.is_null() && !font.is_null() {
                SendMessageW(ctrl, WM_SETFONT, font as WPARAM, 1);
            }
            ctrl
        };

        // Card 1: Telemetry stats labels
        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!("RAM Used: Calculating..."), 0, 22, 48, 340, 20, 101, G_FONT_STATS);
        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!("Free / Available: Calculating..."), 0, 22, 68, 340, 18, 102, G_FONT_UI);
        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!("Status: Ready"), 0, 22, 88, 340, 18, 103, G_FONT_UI);

        // Action buttons (Flat Minimalist Owner-Drawn Ribbon)
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Optimize All"), BS_OWNERDRAW as u32, 12, 134, 136, 32, 201, G_FONT_BOLD);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Purge Standby"), BS_OWNERDRAW as u32, 154, 134, 110, 32, 202, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Combine Pages"), BS_OWNERDRAW as u32, 270, 134, 102, 32, 203, G_FONT_UI);

        // Top 5 RAM Consumers card rows with per-process Trim and Eco actions
        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!("TOP 5 RAM CONSUMERS"), 0, 22, 178, 200, 14, 104, G_FONT_SUBHEADER);
        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!("Loading..."), 0, 22, 202, 240, 16, 105, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim"), BS_OWNERDRAW as u32, 268, 201, 44, 18, 601, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Eco"), BS_OWNERDRAW as u32, 318, 201, 44, 18, 701, G_FONT_UI);

        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!(""), 0, 22, 216, 240, 16, 106, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim"), BS_OWNERDRAW as u32, 268, 215, 44, 18, 602, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Eco"), BS_OWNERDRAW as u32, 318, 215, 44, 18, 702, G_FONT_UI);

        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!(""), 0, 22, 230, 240, 16, 107, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim"), BS_OWNERDRAW as u32, 268, 229, 44, 18, 603, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Eco"), BS_OWNERDRAW as u32, 318, 229, 44, 18, 703, G_FONT_UI);

        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!(""), 0, 22, 244, 240, 16, 108, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim"), BS_OWNERDRAW as u32, 268, 243, 44, 18, 604, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Eco"), BS_OWNERDRAW as u32, 318, 243, 44, 18, 704, G_FONT_UI);

        create_ctrl(windows_sys::w!("STATIC"), windows_sys::w!(""), 0, 22, 258, 240, 16, 109, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim"), BS_OWNERDRAW as u32, 268, 257, 44, 18, 605, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Eco"), BS_OWNERDRAW as u32, 318, 257, 44, 18, 705, G_FONT_UI);
        // Card 2: Optimization Pipeline Checkboxes
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Purge Standby Page List"), BS_AUTOCHECKBOX as u32, 22, 322, 340, 20, 301, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Low-Priority Standby Purge"), BS_AUTOCHECKBOX as u32, 22, 343, 340, 20, 302, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Flush Modified Page List"), BS_AUTOCHECKBOX as u32, 22, 364, 340, 20, 303, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim System File Cache"), BS_AUTOCHECKBOX as u32, 22, 385, 340, 20, 304, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Trim Background Working Sets"), BS_AUTOCHECKBOX as u32, 22, 406, 340, 20, 305, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Enable Game && Foreground Shield"), BS_AUTOCHECKBOX as u32, 22, 427, 340, 20, 306, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Auto-optimize when RAM exceeds 80%"), BS_AUTOCHECKBOX as u32, 22, 448, 340, 20, 307, G_FONT_UI);

        // Footer action buttons
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Settings..."), BS_OWNERDRAW as u32, 12, 484, 114, 30, 401, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Minimize to Tray"), BS_OWNERDRAW as u32, 132, 484, 126, 30, 402, G_FONT_UI);
        create_ctrl(windows_sys::w!("BUTTON"), windows_sys::w!("Exit"), BS_OWNERDRAW as u32, 264, 484, 108, 30, 403, G_FONT_UI);
        load_config_to_controls(hwnd);
        update_stats_display(hwnd);
        update_top_processes(hwnd);

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

        SetTimer(hwnd, TIMER_ID, 1000, None);
        SetTimer(hwnd, PROCESS_TIMER_ID, 5000, None);

        // Register global hotkey Ctrl+Alt+M if enabled.
        if !sync_global_hotkey(hwnd, get_config().general.global_hotkey_enabled) {
            set_text(hwnd, 103, "Status: Ctrl+Alt+M is already in use.");
        }

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
                let cfg = get_config();
                if cfg.tray.click_action == TrayClickAction::QuickOptimize {
                    let before = memory::get_memory_stats().map(|s| s.avail_phys_mb).unwrap_or(0);
                    let _ = cleaner::perform_clean(&cfg, false);
                    let after = memory::get_memory_stats().map(|s| s.avail_phys_mb).unwrap_or(0);
                    let freed = if after > before { after - before } else { 0 };
                    show_tray_notification(hwnd, "MemClean Optimization", &format!("Freed {} MB physical RAM.", freed));
                    update_stats_display(hwnd);
                } else {
                    if IsWindowVisible(hwnd) != 0 {
                        ShowWindow(hwnd, SW_HIDE);
                    } else {
                        ShowWindow(hwnd, SW_RESTORE);
                        SetForegroundWindow(hwnd);
                    }
                }
            }
            0
        }
        WM_HOTKEY => {
            if wparam as i32 == HOTKEY_ID {
                let cfg = get_config();
                let before = memory::get_memory_stats().map(|s| s.avail_phys_mb).unwrap_or(0);
                let _ = cleaner::perform_clean(&cfg, false);
                let after = memory::get_memory_stats().map(|s| s.avail_phys_mb).unwrap_or(0);
                let freed = if after > before { after - before } else { 0 };
                let status = format!("Status: Hotkey optimized; freed {} MB.", freed);
                set_text(hwnd, 103, &status);
                show_tray_notification(hwnd, "MemClean Hotkey", &format!("Freed {} MB RAM.", freed));
                update_stats_display(hwnd);
            }
            0
        }
        WM_TIMER => {
            if wparam as usize == PROCESS_TIMER_ID {
                update_top_processes(hwnd);
                return 0;
            }
            let config = get_config();
            if let Ok(stats) = memory::get_memory_stats() {
                let now = Instant::now();

                // 1. Threshold-based auto-clean (debounced to 60s cooldown to prevent continuous 1Hz thrashing)
                let can_auto_clean = {
                    let lock = LAST_AUTO_CLEAN.lock().unwrap();
                    lock.map_or(true, |last| now.duration_since(last).as_secs() >= 60)
                };

                if config.triggers.auto_threshold_enabled
                    && stats.used_percent >= config.triggers.threshold_percent
                    && can_auto_clean
                {
                    cleaner::perform_clean(&config, false);
                    if let Ok(mut lock) = LAST_AUTO_CLEAN.lock() {
                        *lock = Some(now);
                    }
                    set_text(hwnd, 103, "Status: Auto-cleaned (exceeded threshold).");
                }

                // 2. Scheduled timer-based auto-clean (continuous or periodic interval)
                if config.triggers.auto_timer_enabled {
                    let sec = config.triggers.timer_seconds;
                    let should_timer_clean = if sec == 0 {
                        // 0 = Continuous mode: runs every 1-second tick without time limit
                        true
                    } else {
                        let lock = LAST_PERIODIC_CLEAN.lock().unwrap();
                        lock.map_or(true, |last| now.duration_since(last).as_secs() >= (sec as u64))
                    };

                    if should_timer_clean {
                        cleaner::perform_clean(&config, false);
                        if let Ok(mut lock) = LAST_PERIODIC_CLEAN.lock() {
                            *lock = Some(now);
                        }
                        if sec == 0 {
                            set_text(hwnd, 103, "Status: Auto-cleaned (continuous mode).");
                        } else {
                            let msg = format!("Status: Auto-cleaned (timer: {}s).\0", sec);
                            let wide: Vec<u16> = msg.encode_utf16().collect();
                            SetWindowTextW(GetDlgItem(hwnd, 103), wide.as_ptr());
                        }
                    }
                }
                let ext_stats = G_EXT_STATS.lock().ok().and_then(|lock| *lock);
                // 3. ISLC dual condition check
                if config.triggers.islc_enabled {
                    if let Some(ext) = ext_stats {
                        if crate::engine::islc::should_islc_purge(&ext, config.triggers.islc_standby_mb, config.triggers.islc_free_mb) {
                            cleaner::purge_standby_only();
                            set_text(hwnd, 103, "Status: ISLC auto-purged standby list.");
                        }
                    }
                }

                // 4. Inactivity / AFK Idle clean check
                if config.triggers.idle_clean_enabled {
                    let idle_sec = crate::engine::idle::get_user_idle_seconds();
                    if idle_sec < 5 {
                        G_IDLE_CLEANED.store(false, std::sync::atomic::Ordering::Relaxed);
                    } else if crate::engine::idle::should_idle_clean(config.triggers.idle_seconds, G_IDLE_CLEANED.load(std::sync::atomic::Ordering::Relaxed)) {
                        let _ = cleaner::perform_clean(&config, false);
                        G_IDLE_CLEANED.store(true, std::sync::atomic::Ordering::Relaxed);
                        set_text(hwnd, 103, "Status: Auto-cleaned while idle.");
                    }
                }

                // 5. Sync timer resolution guard
                let timer_mode = config.tuning.timer_mode_enum();
                if let Ok(mut guard_lock) = G_TIMER_GUARD.lock() {
                    if guard_lock.is_none() {
                        let mut guard = crate::core::timer_res::TimerResolutionGuard::new();
                        let _ = guard.set_mode(timer_mode);
                        *guard_lock = Some(guard);
                    } else if let Some(guard) = guard_lock.as_mut() {
                        if guard.current_mode() != timer_mode {
                            let _ = guard.set_mode(timer_mode);
                        }
                    }
                }

                update_stats_display(hwnd);

                let new_icon = create_percent_icon(stats.used_percent);
                let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
                nid.uCallbackMessage = WM_TRAYICON;
                nid.hIcon = new_icon;

                let cur_res = crate::core::timer_res::TimerResolutionGuard::get_current_resolution_ms();
                let standby_mb = ext_stats.map_or(0, |s| s.standby_total_mb);
                let tip = format!("{}\0", format_tray_tooltip(stats.used_percent, standby_mb, cur_res));
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
                    if id == 201 {
                        save_config_from_controls(hwnd);
                    }
                    let before = memory::get_memory_stats().map(|s| s.avail_phys_mb).unwrap_or(0);
                    let config = get_config();
                    let _ = cleaner::perform_clean(&config, false);
                    let after = memory::get_memory_stats().map(|s| s.avail_phys_mb).unwrap_or(0);
                    let freed = if after > before { after - before } else { 0 };
                    set_text(hwnd, 103, &format!("Status: Optimized! Freed {} MB RAM.", freed));
                    update_stats_display(hwnd);

                    if id == 502 {
                        show_tray_notification(hwnd, "MemClean Optimization", &format!("Freed {} MB physical RAM.", freed));
                    }
                }
                202 | 503 => { // Purge Standby
                    let _ = crate::core::ntdll::purge_standby_list();
                    set_text(hwnd, 103, "Status: Standby list purged.");
                    update_stats_display(hwnd);
                }
                203 | 507 => { // Combine Pages
                    match cleaner::combine_pages_only() {
                        Ok(c) => set_text(hwnd, 103, &format!("Status: Combined {} pages ({:.1} MB saved).", c.pages_combined, c.mb_saved)),
                        Err(e) => set_text(hwnd, 103, &format!("Status: Combine failed: {}", e)),
                    }
                    update_stats_display(hwnd);
                }
                508 => { // Toggle Timer Resolution
                    let cur_res = crate::core::timer_res::TimerResolutionGuard::get_current_resolution_ms();
                    let target_mode = if cur_res <= 0.6 {
                        crate::core::timer_res::TimerResolutionMode::Default
                    } else {
                        crate::core::timer_res::TimerResolutionMode::HalfMs
                    };
                    if let Ok(mut guard_lock) = G_TIMER_GUARD.lock() {
                        if let Some(guard) = guard_lock.as_mut() {
                            let _ = guard.set_mode(target_mode);
                        }
                    }
                    let updated_res = crate::core::timer_res::TimerResolutionGuard::get_current_resolution_ms();
                    set_text(hwnd, 103, &format!("Status: Timer resolution set to {:.3} ms.", updated_res));
                    InvalidateRect(hwnd, std::ptr::null(), 0);
                }
                401 | 506 => { // Settings dialog
                    save_config_from_controls(hwnd);
                    crate::ui::dialog::show_settings_dialog(hwnd);
                    {
                        let mut lock = CACHED_CONFIG.lock().unwrap();
                        *lock = Some(AppConfig::load());
                    }
                    load_config_to_controls(hwnd);
                    let hotkey_enabled = get_config().general.global_hotkey_enabled;
                    if !sync_global_hotkey(hwnd, hotkey_enabled) {
                        set_text(hwnd, 103, "Status: Ctrl+Alt+M is already in use.");
                    } else {
                        set_text(hwnd, 103, "Status: Settings saved successfully.");
                    }
                    update_stats_display(hwnd);
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
                601..=605 => {
                    let index = (id - 601) as usize;
                    let pid = TOP_PROCESSES.lock().ok()
                        .and_then(|processes| processes.get(index).map(|process| process.pid));
                    if let Some(pid) = pid {
                        match crate::core::process::trim_process_working_set(pid) {
                            Ok(()) => set_text(hwnd, 103, "Status: Process working set trimmed."),
                            Err(_) => set_text(hwnd, 103, "Status: Process could not be trimmed."),
                        }
                        update_top_processes(hwnd);
                    }
                }
                701..=705 => {
                    let index = (id - 701) as usize;
                    let proc = TOP_PROCESSES.lock().ok()
                        .and_then(|processes| processes.get(index).cloned());
                    if let Some(p) = proc {
                        match crate::core::process::set_process_eco_qos(p.pid, true) {
                            Ok(()) => set_text(hwnd, 103, &format!("Status: Enabled EcoQoS on {} (PID {}).", p.name, p.pid)),
                            Err(e) => set_text(hwnd, 103, &format!("Status: EcoQoS failed for {}: {}", p.name, e)),
                        }
                    }
                }
                _ => {}
            }
            0
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            FillRect(hdc, &client_rc, G_BG_BRUSH);

            // Card 1: Telemetry (12, 10, 372, 126)
            let old_brush = SelectObject(hdc, G_CARD_BRUSH);
            let old_pen = SelectObject(hdc, G_BORDER_PEN);
            RoundRect(hdc, 12, 10, 372, 126, 10, 10);

            // Card 1 Header
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00747778); // Muted gray #787774
            let old_font = SelectObject(hdc, G_FONT_SUBHEADER);
            let logo_icon = load_embedded_icon(24);
            if !logo_icon.is_null() {
                DrawIconEx(hdc, 22, 14, logo_icon, 20, 20, 0, std::ptr::null_mut(), DI_NORMAL);
            }
            let header1 = windows_sys::w!("MEMCLEAN — PHYSICAL RAM TELEMETRY");
            let mut h1_rc = RECT { left: 47, top: 17, right: 240, bottom: 32 };
            DrawTextW(hdc, header1, -1, &mut h1_rc, DT_SINGLELINE);

            let cur_res = crate::core::timer_res::TimerResolutionGuard::get_current_resolution_ms();
            let timer_str = format!("Timer: {:.3} ms\0", cur_res);
            let timer_wide: Vec<u16> = timer_str.encode_utf16().collect();
            let mut timer_rc = RECT { left: 240, top: 17, right: 362, bottom: 32 };
            DrawTextW(hdc, timer_wide.as_ptr(), -1, &mut timer_rc, DT_RIGHT | DT_SINGLELINE);

            // Progress bar track at (22, 36, 362, 42)
            let null_pen = GetStockObject(NULL_PEN as i32);
            SelectObject(hdc, null_pen);
            SelectObject(hdc, G_TRACK_BRUSH);
            RoundRect(hdc, 22, 36, 362, 42, 4, 4);

            // Segmented progress bar fill
            let ext_stats = G_EXT_STATS.lock().ok().and_then(|lock| *lock);
            if let Some(ext) = ext_stats {
                let (in_use_w, standby_w, mod_w, _rem_w) = calculate_memory_bar_segments(
                    ext.total_phys_mb,
                    ext.used_phys_mb,
                    ext.standby_total_mb,
                    ext.modified_mb,
                    340,
                );
                let mut cur_x = 22;
                if in_use_w > 0 {
                    let b = CreateSolidBrush(0x00111111);
                    SelectObject(hdc, b);
                    RoundRect(hdc, cur_x, 36, (cur_x + in_use_w).min(362), 42, 4, 4);
                    DeleteObject(b);
                    cur_x += in_use_w;
                }
                if standby_w > 0 && cur_x < 362 {
                    let b = CreateSolidBrush(0x00B59A7A); // Slate Blue
                    SelectObject(hdc, b);
                    RoundRect(hdc, cur_x, 36, (cur_x + standby_w).min(362), 42, 4, 4);
                    DeleteObject(b);
                    cur_x += standby_w;
                }
                if mod_w > 0 && cur_x < 362 {
                    let b = CreateSolidBrush(0x000677D9); // Amber
                    SelectObject(hdc, b);
                    RoundRect(hdc, cur_x, 36, (cur_x + mod_w).min(362), 42, 4, 4);
                    DeleteObject(b);
                }
            } else {
                let pct = G_USED_PERCENT.min(100);
                if pct > 0 {
                    let bar_brush = CreateSolidBrush(0x00111111);
                    SelectObject(hdc, bar_brush);
                    let track_w = 362 - 22;
                    let fill_w = ((track_w as u32 * pct) / 100) as i32;
                    RoundRect(hdc, 22, 36, 22 + fill_w.max(6), 42, 4, 4);
                    DeleteObject(bar_brush);
                }
            }

            // Top consumers card
            SelectObject(hdc, G_CARD_BRUSH);
            SelectObject(hdc, G_BORDER_PEN);
            RoundRect(hdc, 12, 170, 372, 280, 10, 10);
            SetBkMode(hdc, TRANSPARENT as i32);
            SelectObject(hdc, G_FONT_SUBHEADER);
            SetTextColor(hdc, 0x00747778);
            let top_header = windows_sys::w!("TOP 5 RAM CONSUMERS");
            let mut top_header_rc = RECT { left: 22, top: 178, right: 362, bottom: 192 };
            DrawTextW(hdc, top_header, -1, &mut top_header_rc, DT_SINGLELINE);
            SelectObject(hdc, G_BORDER_PEN);
            MoveToEx(hdc, 22, 196, std::ptr::null_mut());
            LineTo(hdc, 362, 196);

            // Optimization pipeline card
            SelectObject(hdc, G_CARD_BRUSH);
            SelectObject(hdc, G_BORDER_PEN);
            RoundRect(hdc, 12, 288, 372, 472, 10, 10);
            SelectObject(hdc, G_FONT_SUBHEADER);
            SetTextColor(hdc, 0x00747778);
            let header2 = windows_sys::w!("OPTIMIZATION PIPELINE");
            let mut h2_rc = RECT { left: 22, top: 296, right: 362, bottom: 310 };
            DrawTextW(hdc, header2, -1, &mut h2_rc, DT_SINGLELINE);
            SelectObject(hdc, G_BORDER_PEN);
            MoveToEx(hdc, 22, 316, std::ptr::null_mut());
            LineTo(hdc, 362, 316);
            // Restore DC
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
            if ctrl_id == 101 { // RAM Used
                SetTextColor(hdc, 0x00111111);
            } else if ctrl_id == 102 || ctrl_id == 103 { // Stats secondary
                SetTextColor(hdc, 0x00787774);
            } else {
                SetTextColor(hdc, 0x00111111);
            }
            G_CARD_BRUSH as LRESULT // All statics and checkboxes are inside white cards!
        }
        WM_CTLCOLORBTN => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00111111);
            G_CARD_BRUSH as LRESULT
        }
        WM_CTLCOLORDLG => {
            G_BG_BRUSH as LRESULT
        }
        WM_DRAWITEM => {
            let dis = &*(lparam as *const DRAWITEMSTRUCT);
            if dis.CtlID == 201 {
                // Primary CTA: Solid off-black #111111, white text, 4px rounded rectangle
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
                let old_font = SelectObject(dis.hDC, G_FONT_BOLD);
                let text = windows_sys::w!("Optimize All");
                let mut rc = dis.rcItem;
                DrawTextW(dis.hDC, text, -1, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                SelectObject(dis.hDC, old_font);
                1
            } else if dis.CtlID == 202 {
                // Secondary Button: White background, hairline #EAEAEA border, off-black text
                let is_selected = (dis.itemState & ODS_SELECTED) != 0;
                let bg_color = if is_selected { 0x00EFEFEF } else { 0x00FFFFFF };
                let brush = CreateSolidBrush(bg_color);
                let pen = CreatePen(PS_SOLID as i32, 1, 0x00EAEAEA);
                let old_brush = SelectObject(dis.hDC, brush);
                let old_pen = SelectObject(dis.hDC, pen);
                RoundRect(dis.hDC, dis.rcItem.left, dis.rcItem.top, dis.rcItem.right, dis.rcItem.bottom, 6, 6);
                SelectObject(dis.hDC, old_pen);
                SelectObject(dis.hDC, old_brush);
                DeleteObject(pen);
                DeleteObject(brush);

                SetBkMode(dis.hDC, TRANSPARENT as i32);
                SetTextColor(dis.hDC, 0x00111111);
                let old_font = SelectObject(dis.hDC, G_FONT_UI);
                let text = windows_sys::w!("Purge Standby");
                let mut rc = dis.rcItem;
                DrawTextW(dis.hDC, text, -1, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                SelectObject(dis.hDC, old_font);
                1
            } else if (401..=403).contains(&dis.CtlID) || (601..=605).contains(&dis.CtlID) || (701..=705).contains(&dis.CtlID) || dis.CtlID == 203 {
                // Footer buttons: White background, hairline #EAEAEA border, off-black text
                let is_selected = (dis.itemState & ODS_SELECTED) != 0;
                let bg_color = if is_selected { 0x00EFEFEF } else { 0x00FFFFFF };
                let brush = CreateSolidBrush(bg_color);
                let pen = CreatePen(PS_SOLID as i32, 1, 0x00EAEAEA);
                let old_brush = SelectObject(dis.hDC, brush);
                let old_pen = SelectObject(dis.hDC, pen);
                RoundRect(dis.hDC, dis.rcItem.left, dis.rcItem.top, dis.rcItem.right, dis.rcItem.bottom, 6, 6);
                SelectObject(dis.hDC, old_pen);
                SelectObject(dis.hDC, old_brush);
                DeleteObject(pen);
                DeleteObject(brush);

                SetBkMode(dis.hDC, TRANSPARENT as i32);
                SetTextColor(dis.hDC, 0x00111111);
                let old_font = SelectObject(dis.hDC, G_FONT_UI);
                let text = match dis.CtlID {
                    203 => windows_sys::w!("Combine Pages"),
                    401 => windows_sys::w!("Settings..."),
                    402 => windows_sys::w!("Minimize to Tray"),
                    403 => windows_sys::w!("Exit"),
                    601..=605 => windows_sys::w!("Trim"),
                    701..=705 => windows_sys::w!("Eco"),
                    _ => windows_sys::w!(""),
                };
                let mut rc = dis.rcItem;
                DrawTextW(dis.hDC, text, -1, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                SelectObject(dis.hDC, old_font);
                1
            } else {
                0
            }
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
            UnregisterHotKey(hwnd, HOTKEY_ID);
            if !G_BG_BRUSH.is_null() {
                DeleteObject(G_BG_BRUSH);
                G_BG_BRUSH = std::ptr::null_mut();
            }
            if !G_CARD_BRUSH.is_null() {
                DeleteObject(G_CARD_BRUSH);
                G_CARD_BRUSH = std::ptr::null_mut();
            }
            if !G_BORDER_PEN.is_null() {
                DeleteObject(G_BORDER_PEN);
                G_BORDER_PEN = std::ptr::null_mut();
            }
            if !G_TRACK_BRUSH.is_null() {
                DeleteObject(G_TRACK_BRUSH);
                G_TRACK_BRUSH = std::ptr::null_mut();
            }
            if !G_FONT_UI.is_null() {
                DeleteObject(G_FONT_UI);
                G_FONT_UI = std::ptr::null_mut();
            }
            if !G_FONT_BOLD.is_null() {
                DeleteObject(G_FONT_BOLD);
                G_FONT_BOLD = std::ptr::null_mut();
            }
            if !G_FONT_SUBHEADER.is_null() {
                DeleteObject(G_FONT_SUBHEADER);
                G_FONT_SUBHEADER = std::ptr::null_mut();
            }
            if !G_FONT_STATS.is_null() {
                DeleteObject(G_FONT_STATS);
                G_FONT_STATS = std::ptr::null_mut();
            }
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
    AppendMenuW(menu, MF_STRING, 507, windows_sys::w!("Combine Duplicate Pages"));
    AppendMenuW(menu, MF_STRING, 508, windows_sys::w!("Toggle 0.5ms Timer Resolution"));
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, 506, windows_sys::w!("Settings..."));
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
        use crate::config::TrayConfig;
        let cfg = TrayConfig::default();
        let icon = create_percent_icon_with_config(42, &cfg);
        assert!(!icon.is_null());

        let capsule_cfg = TrayConfig {
            style: TrayStyle::Capsule,
            color_mode: TrayColorMode::Dynamic,
            click_action: TrayClickAction::OpenDashboard,
        };
        let icon_capsule = create_percent_icon_with_config(88, &capsule_cfg);
        assert!(!icon_capsule.is_null());

        let mono_cfg = TrayConfig {
            style: TrayStyle::Floating,
            color_mode: TrayColorMode::Monochrome,
            click_action: TrayClickAction::OpenDashboard,
        };
        let icon_mono = create_percent_icon_with_config(72, &mono_cfg);
        assert!(!icon_mono.is_null());
    }

    #[test]
    fn test_load_embedded_icon_frames() {
        let icon16 = load_embedded_icon(16);
        assert!(!icon16.is_null());
        let icon32 = load_embedded_icon(32);
        assert!(!icon32.is_null());
        let icon48 = load_embedded_icon(48);
        assert!(!icon48.is_null());
        unsafe {
            DestroyIcon(icon16);
            DestroyIcon(icon32);
            DestroyIcon(icon48);
        }
    }

    #[test]
    fn test_calculate_memory_bar_segments() {
        let (in_use, standby, mod_w, rem) = calculate_memory_bar_segments(16384, 8192, 4096, 1024, 340);
        assert_eq!(in_use, 170); // 50%
        assert_eq!(standby, 85);  // 25%
        assert_eq!(mod_w, 21);   // 6.25%
        assert!(rem > 0);
        assert_eq!(in_use + standby + mod_w + rem, 340);
    }

    #[test]
    fn test_format_tray_tooltip() {
        let tip = format_tray_tooltip(57, 4096, 0.5);
        assert_eq!(tip, "MemClean: 57% | Standby: 4096MB | Timer: 0.5ms");
    }
}
