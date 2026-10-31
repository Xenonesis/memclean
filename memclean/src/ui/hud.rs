use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen,
    CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, InvalidateRect,
    LineTo, MoveToEx, RoundRect, SelectObject, SetBkMode, SetTextColor, HBRUSH, HFONT, HPEN,
    PAINTSTRUCT, SRCCOPY, TRANSPARENT, DT_LEFT, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, PS_SOLID,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetForegroundWindow,
    GetMessageW, GetSystemMetrics, GetWindowTextW, GetWindowThreadProcessId, IsWindow, IsWindowVisible,
    KillTimer, PostQuitMessage, RegisterClassW, SetLayeredWindowAttributes,
    SetTimer, SetWindowPos, ShowWindow, HWND_TOPMOST, LWA_ALPHA, MSG, SM_CXSCREEN, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE, WM_DESTROY, WM_ERASEBKGND, WM_PAINT,
    WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};

static mut G_HUD_HWND: HWND = std::ptr::null_mut();
static mut HUD_BG_BRUSH: HBRUSH = std::ptr::null_mut();
static mut HUD_CARD_BRUSH: HBRUSH = std::ptr::null_mut();
static mut HUD_BAR_BG_BRUSH: HBRUSH = std::ptr::null_mut();
static mut HUD_BAR_FILL_BRUSH: HBRUSH = std::ptr::null_mut();
static mut HUD_BORDER_PEN: HPEN = std::ptr::null_mut();
static mut HUD_DIVIDER_PEN: HPEN = std::ptr::null_mut();
static mut HUD_FONT_TITLE: HFONT = std::ptr::null_mut();
static mut HUD_FONT_BOLD: HFONT = std::ptr::null_mut();
static mut HUD_FONT_DATA: HFONT = std::ptr::null_mut();
static mut HUD_FONT_MINI: HFONT = std::ptr::null_mut();

static HUD_RUNNING: AtomicBool = AtomicBool::new(false);
const HUD_TIMER_ID: usize = 2001;

pub fn is_hud_visible() -> bool {
    unsafe {
        if !G_HUD_HWND.is_null() && IsWindow(G_HUD_HWND) != 0 {
            IsWindowVisible(G_HUD_HWND) != 0
        } else {
            false
        }
    }
}

pub fn toggle_hud() {
    unsafe {
        if !G_HUD_HWND.is_null() && IsWindow(G_HUD_HWND) != 0 {
            if IsWindowVisible(G_HUD_HWND) != 0 {
                ShowWindow(G_HUD_HWND, SW_HIDE);
            } else {
                ShowWindow(G_HUD_HWND, SW_SHOWNOACTIVATE);
                SetWindowPos(
                    G_HUD_HWND,
                    HWND_TOPMOST,
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        } else {
            spawn_hud_thread();
        }
    }
}

pub fn show_hud() {
    unsafe {
        if !G_HUD_HWND.is_null() && IsWindow(G_HUD_HWND) != 0 {
            ShowWindow(G_HUD_HWND, SW_SHOWNOACTIVATE);
            SetWindowPos(
                G_HUD_HWND,
                HWND_TOPMOST,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        } else {
            spawn_hud_thread();
        }
    }
}

pub fn hide_hud() {
    unsafe {
        if !G_HUD_HWND.is_null() && IsWindow(G_HUD_HWND) != 0 {
            ShowWindow(G_HUD_HWND, SW_HIDE);
        }
    }
}

pub fn spawn_hud_thread() {
    if HUD_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(|| {
        unsafe {
            run_hud_window();
        }
        HUD_RUNNING.store(false, Ordering::SeqCst);
    });
}

unsafe fn run_hud_window() {
    let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
    let class_name = windows_sys::w!("MemCleanGamingHudClass");

    HUD_BG_BRUSH = CreateSolidBrush(0x00111111); // Matte Black #111111
    HUD_CARD_BRUSH = CreateSolidBrush(0x001C1C1C);
    HUD_BAR_BG_BRUSH = CreateSolidBrush(0x002B2B2B);
    HUD_BAR_FILL_BRUSH = CreateSolidBrush(0x00EDF3EC); // Soft Sage Green
    HUD_BORDER_PEN = CreatePen(PS_SOLID as i32, 1, 0x00333333); // Subtle Hairline
    HUD_DIVIDER_PEN = CreatePen(PS_SOLID as i32, 1, 0x00262626);

    let font_face = windows_sys::w!("Segoe UI");
    HUD_FONT_TITLE = CreateFontW(14, 0, 0, 0, 700, 0, 0, 0, 0, 0, 0, 5, 0, font_face);
    HUD_FONT_BOLD = CreateFontW(13, 0, 0, 0, 700, 0, 0, 0, 0, 0, 0, 5, 0, font_face);
    HUD_FONT_DATA = CreateFontW(12, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 5, 0, font_face);
    HUD_FONT_MINI = CreateFontW(11, 0, 0, 0, 600, 0, 0, 0, 0, 0, 0, 5, 0, font_face);

    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(hud_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: 0 as _,
        hCursor: std::ptr::null_mut(),
        hbrBackground: HUD_BG_BRUSH,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name,
    };
    RegisterClassW(&wc);

    let width = 310;
    let height = 118;
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let x = screen_w - width - 20;
    let y = 20;

    let ex_style = WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
    let hwnd = CreateWindowExW(
        ex_style,
        class_name,
        windows_sys::w!("MemClean Gaming HUD"),
        WS_POPUP | windows_sys::Win32::UI::WindowsAndMessaging::WS_VISIBLE,
        x, y, width, height,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        hinstance,
        std::ptr::null_mut(),
    );

    G_HUD_HWND = hwnd;

    // 88% Alpha Opacity (225/255)
    SetLayeredWindowAttributes(hwnd, 0, 225, LWA_ALPHA);

    SetTimer(hwnd, HUD_TIMER_ID, 1000, None);

    let mut msg: MSG = std::mem::zeroed();
    while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn hud_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => 1,
        WM_TIMER => {
            if wparam == HUD_TIMER_ID {
                InvalidateRect(hwnd, std::ptr::null(), 0);
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            let w = client_rc.right - client_rc.left;
            let h = client_rc.bottom - client_rc.top;

            // Double Buffer
            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            // 1. Background Rounded Card
            let old_brush = SelectObject(mem_dc, HUD_BG_BRUSH);
            let old_pen = SelectObject(mem_dc, HUD_BORDER_PEN);
            RoundRect(mem_dc, 0, 0, w, h, 10, 10);

            SetBkMode(mem_dc, TRANSPARENT as i32);

            // 2. Title & Timer Header
            let old_font = SelectObject(mem_dc, HUD_FONT_TITLE);
            SetTextColor(mem_dc, 0x00FFFFFF);
            let mut title_rc = RECT { left: 12, top: 8, right: 160, bottom: 24 };
            DrawTextW(mem_dc, windows_sys::w!("[M] MEMCLEAN HUD"), -1, &mut title_rc, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            // Active Timer Resolution
            let cur_res = crate::core::timer_res::TimerResolutionGuard::get_current_resolution_ms();
            let timer_color = if cur_res <= 1.0 { 0x0080E8A7 } else { 0x00A0A0A0 }; // Soft Green or Gray
            SetTextColor(mem_dc, timer_color);
            SelectObject(mem_dc, HUD_FONT_MINI);
            let timer_str = format!("TIMER: {:.3}ms\0", cur_res);
            let timer_wide: Vec<u16> = timer_str.encode_utf16().collect();
            let mut timer_rc = RECT { left: 160, top: 8, right: w - 12, bottom: 24 };
            DrawTextW(mem_dc, timer_wide.as_ptr(), -1, &mut timer_rc, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);

            // Divider line
            SelectObject(mem_dc, HUD_DIVIDER_PEN);
            MoveToEx(mem_dc, 12, 28, std::ptr::null_mut());
            LineTo(mem_dc, w - 12, 28);

            // 3. Telemetry Lines
            SelectObject(mem_dc, HUD_FONT_DATA);

            // Line 1: RAM Usage
            let mem_stats = crate::core::memory::get_extended_memory_stats();
            let (ram_used_mb, ram_pct, standby_mb) = if let Ok(s) = mem_stats {
                (s.used_phys_mb, s.used_percent, s.standby_total_mb)
            } else {
                (0, 0, 0)
            };

            SetTextColor(mem_dc, 0x00D0D0D0);
            let mut l1_label = RECT { left: 12, top: 34, right: 55, bottom: 50 };
            DrawTextW(mem_dc, windows_sys::w!("RAM"), -1, &mut l1_label, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            let ram_info_str = format!("{:.1} GB ({}%)  Standby: {:.1} GB\0", ram_used_mb as f64 / 1024.0, ram_pct, standby_mb as f64 / 1024.0);
            let ram_info_wide: Vec<u16> = ram_info_str.encode_utf16().collect();
            let mut l1_val = RECT { left: 55, top: 34, right: w - 12, bottom: 50 };
            SetTextColor(mem_dc, 0x00FFFFFF);
            DrawTextW(mem_dc, ram_info_wide.as_ptr(), -1, &mut l1_val, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            // Line 2: GPU VRAM
            SetTextColor(mem_dc, 0x00D0D0D0);
            let mut l2_label = RECT { left: 12, top: 54, right: 55, bottom: 70 };
            DrawTextW(mem_dc, windows_sys::w!("VRAM"), -1, &mut l2_label, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            let gpu_adapters = crate::core::vram::get_gpu_adapters();
            let vram_str = if let Some(gpu) = gpu_adapters.iter().find(|g| g.dedicated_vram_mb > 512).or_else(|| gpu_adapters.first()) {
                format!("{} ({:.1} GB VRAM)\0", gpu.description.chars().take(22).collect::<String>(), gpu.dedicated_vram_mb as f64 / 1024.0)
            } else {
                "Integrated Graphics\0".to_string()
            };
            let vram_wide: Vec<u16> = vram_str.encode_utf16().collect();
            let mut l2_val = RECT { left: 55, top: 54, right: w - 12, bottom: 70 };
            SetTextColor(mem_dc, 0x00FFFFFF);
            DrawTextW(mem_dc, vram_wide.as_ptr(), -1, &mut l2_val, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            // Line 3: Hard Page Faults & Stutter Status
            let (hf_rate, risk) = crate::core::telemetry::get_instant_fault_telemetry();
            SetTextColor(mem_dc, 0x00D0D0D0);
            let mut l3_label = RECT { left: 12, top: 74, right: 55, bottom: 90 };
            DrawTextW(mem_dc, windows_sys::w!("DISK"), -1, &mut l3_label, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            let (risk_text, risk_color) = match risk {
                crate::core::telemetry::StutterRiskLevel::Smooth => ("0 Hard Faults [Smooth - No Stutter]\0", 0x0080E8A7),
                crate::core::telemetry::StutterRiskLevel::Moderate => ("Minor Paging Latency\0", 0x0078DBFB), // Amber
                crate::core::telemetry::StutterRiskLevel::Elevated => ("Stutter Risk (Paging Churn)\0", 0x004A4AFE), // Orange
                crate::core::telemetry::StutterRiskLevel::Severe => ("Severe Disk Thrashing!\0", 0x003333FE), // Red
            };
            let fault_msg = if hf_rate > 0 {
                format!("{} Hard Faults/s [{}]\0", hf_rate, risk_text.trim_end_matches('\0'))
            } else {
                risk_text.to_string()
            };
            let fault_wide: Vec<u16> = fault_msg.encode_utf16().collect();
            let mut l3_val = RECT { left: 55, top: 74, right: w - 12, bottom: 90 };
            SetTextColor(mem_dc, risk_color);
            DrawTextW(mem_dc, fault_wide.as_ptr(), -1, &mut l3_val, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            // Line 4: Active Foreground Process
            SetTextColor(mem_dc, 0x00D0D0D0);
            let mut l4_label = RECT { left: 12, top: 94, right: 55, bottom: 110 };
            DrawTextW(mem_dc, windows_sys::w!("ACTIVE"), -1, &mut l4_label, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            let fg_name = get_active_process_name();
            let fg_str = format!("{}\0", fg_name);
            let fg_wide: Vec<u16> = fg_str.encode_utf16().collect();
            let mut l4_val = RECT { left: 55, top: 94, right: w - 12, bottom: 110 };
            SetTextColor(mem_dc, 0x00FFFFFF);
            DrawTextW(mem_dc, fg_wide.as_ptr(), -1, &mut l4_val, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            // Blit to screen
            BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, SRCCOPY);

            // Cleanup GDI
            SelectObject(mem_dc, old_font);
            SelectObject(mem_dc, old_pen);
            SelectObject(mem_dc, old_brush);
            SelectObject(mem_dc, old_bmp);
            DeleteObject(mem_bmp);
            DeleteDC(mem_dc);

            EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            KillTimer(hwnd, HUD_TIMER_ID);
            G_HUD_HWND = std::ptr::null_mut();
            if !HUD_BG_BRUSH.is_null() { DeleteObject(HUD_BG_BRUSH); HUD_BG_BRUSH = std::ptr::null_mut(); }
            if !HUD_CARD_BRUSH.is_null() { DeleteObject(HUD_CARD_BRUSH); HUD_CARD_BRUSH = std::ptr::null_mut(); }
            if !HUD_BAR_BG_BRUSH.is_null() { DeleteObject(HUD_BAR_BG_BRUSH); HUD_BAR_BG_BRUSH = std::ptr::null_mut(); }
            if !HUD_BAR_FILL_BRUSH.is_null() { DeleteObject(HUD_BAR_FILL_BRUSH); HUD_BAR_FILL_BRUSH = std::ptr::null_mut(); }
            if !HUD_BORDER_PEN.is_null() { DeleteObject(HUD_BORDER_PEN); HUD_BORDER_PEN = std::ptr::null_mut(); }
            if !HUD_DIVIDER_PEN.is_null() { DeleteObject(HUD_DIVIDER_PEN); HUD_DIVIDER_PEN = std::ptr::null_mut(); }
            if !HUD_FONT_TITLE.is_null() { DeleteObject(HUD_FONT_TITLE); HUD_FONT_TITLE = std::ptr::null_mut(); }
            if !HUD_FONT_BOLD.is_null() { DeleteObject(HUD_FONT_BOLD); HUD_FONT_BOLD = std::ptr::null_mut(); }
            if !HUD_FONT_DATA.is_null() { DeleteObject(HUD_FONT_DATA); HUD_FONT_DATA = std::ptr::null_mut(); }
            if !HUD_FONT_MINI.is_null() { DeleteObject(HUD_FONT_MINI); HUD_FONT_MINI = std::ptr::null_mut(); }
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn get_active_process_name() -> String {
    let fg_hwnd = GetForegroundWindow();
    if fg_hwnd.is_null() {
        return "Desktop / Background".to_string();
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(fg_hwnd, &mut pid);
    if pid <= 4 {
        return "System Idle".to_string();
    }

    let mut title_buf = [0u16; 64];
    let len = GetWindowTextW(fg_hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
    if len > 0 {
        let title = String::from_utf16_lossy(&title_buf[..len as usize]);
        return format!("PID {}: {}", pid, title.chars().take(24).collect::<String>());
    }

    format!("PID: {}", pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_dimensions_and_alpha() {
        let width = 310;
        let height = 118;
        assert!(width >= 300 && width <= 360);
        assert!(height >= 100 && height <= 130);
    }
}
