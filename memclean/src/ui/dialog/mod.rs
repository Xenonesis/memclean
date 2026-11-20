pub mod common;
pub mod tab_kernel;
pub mod tab_shortcuts;
pub mod tab_system;
pub mod tab_triggers;

use crate::config::AppConfig;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreatePen, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, InvalidateRect, LineTo, MoveToEx,
    SelectObject, SetBkColor, SetBkMode, SetTextColor, HDC, PAINTSTRUCT, DT_CENTER, DT_NOPREFIX,
    DT_RIGHT, DT_SINGLELINE, DT_VCENTER, PS_SOLID, SRCCOPY, TRANSPARENT,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
};
const WM_MOUSELEAVE: u32 = 0x02A3;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetDlgCtrlID,
    GetDlgItem, GetMessageW, GetSystemMetrics, IsDialogMessageW, IsWindow, KillTimer, LoadCursorW,
    LoadIconW, MessageBoxW, RegisterClassW, SendMessageW, SetTimer, ShowWindow, BS_OWNERDRAW,
    IDC_ARROW, IDI_APPLICATION, IDYES, MB_ICONQUESTION, MB_YESNO, MSG, SM_CXSCREEN, SM_CYSCREEN,
    SW_HIDE, SW_SHOW, WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG, WM_CTLCOLOREDIT,
    WM_CTLCOLORSTATIC, WM_DESTROY, WM_DRAWITEM, WM_ERASEBKGND, WM_MOUSEMOVE, WM_PAINT, WM_SETFONT,
    WM_SETICON, WM_TIMER, WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use self::common::{
    bg_brush, border_pen, card_brush, draw_action_btn, draw_rounded_card, draw_segmented_btn,
    draw_toggle_row, draw_tray_preview_card, font_badge, font_bold, font_desc,
    font_stats_big, font_subheader, font_title, font_ui, free_resources, get_live_cpu_usage,
    init_resources, is_checked, set_checked, surface_brush, COLOR_ACCENT, COLOR_BORDER,
    COLOR_CARD, COLOR_DANGER, COLOR_SUCCESS, COLOR_SURFACE, COLOR_SURFACE_HOVER,
    COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY, COLOR_TEXT_SECONDARY, COLOR_WARNING, ODS_SELECTED,
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
static mut HOVERED_BTN: i32 = 0;
const MONITOR_TIMER_ID: usize = 9999;

unsafe fn switch_tab(hwnd: HWND, new_tab: usize) {
    CURRENT_TAB = new_tab;

    // Tab 0: General
    for &id in tab_system::CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 0 { SW_SHOW } else { SW_HIDE });
        }
    }
    // Tab 1: Automation
    for &id in tab_triggers::CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 1 { SW_SHOW } else { SW_HIDE });
        }
    }
    // Tab 2: Kernel & Gaming
    for &id in tab_kernel::CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 2 { SW_SHOW } else { SW_HIDE });
        }
    }
    // Tab 3: Shortcuts
    for &id in tab_shortcuts::CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 3 { SW_SHOW } else { SW_HIDE });
        }
    }

    // Invalidate tab buttons (601..=604) to redraw active states
    for id in 601..=604 {
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
            hIcon: LoadIconW(std::ptr::null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wnd_class);

        let win_w = 840;
        let win_h = 724;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - win_w) / 2;
        let pos_y = (screen_h - win_h) / 2;

        let hwnd = CreateWindowExW(
            0,
            class_name,
            windows_sys::w!("MemClean – Settings & Optimization Policy"),
            (WS_OVERLAPPEDWINDOW
                & !(windows_sys::Win32::UI::WindowsAndMessaging::WS_MAXIMIZEBOX
                    | windows_sys::Win32::UI::WindowsAndMessaging::WS_THICKFRAME))
                | WS_VISIBLE,
            pos_x,
            pos_y,
            win_w,
            win_h,
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

        // Top Navigation Tab Buttons (BS_OWNERDRAW) - 4 Tabs
        let create_tab_btn = |text: *const u16, cx: i32, cy: i32, cw: i32, id: i32| -> HWND {
            let btn = CreateWindowExW(
                0,
                windows_sys::w!("BUTTON"),
                text,
                WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                cx,
                cy,
                cw,
                38,
                hwnd,
                id as _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            let f_ui = font_ui();
            if !btn.is_null() && !f_ui.is_null() {
                SendMessageW(btn, WM_SETFONT, f_ui as WPARAM, 1);
            }
            btn
        };

        // 4 Modern segmented navigation tabs
        create_tab_btn(windows_sys::w!("⚙ General"), 20, 122, 194, 601);
        create_tab_btn(windows_sys::w!("⚡ Automation"), 218, 122, 194, 602);
        create_tab_btn(windows_sys::w!("🎮 Kernel & Gaming"), 416, 122, 194, 603);
        create_tab_btn(windows_sys::w!("⌨ Shortcuts"), 614, 122, 194, 604);

        // Instantiate controls for each tab page component
        tab_system::create(hwnd);
        tab_triggers::create(hwnd);
        tab_kernel::create(hwnd);
        tab_shortcuts::create(hwnd);

        // Load configuration into controls
        let cfg = AppConfig::load();
        tab_system::load(hwnd, &cfg);
        tab_triggers::load(hwnd, &cfg);
        tab_kernel::load(hwnd, &cfg);
        tab_shortcuts::load(hwnd, &cfg);

        // Switch to Tab 0 as initial view
        switch_tab(hwnd, 0);

        // Footer Action Buttons
        // ID 303: Restore Defaults
        CreateWindowExW(
            0,
            windows_sys::w!("BUTTON"),
            windows_sys::w!("↺ Restore Defaults"),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            20,
            636,
            140,
            36,
            hwnd,
            303 as _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        // ID 302: Cancel
        CreateWindowExW(
            0,
            windows_sys::w!("BUTTON"),
            windows_sys::w!("Cancel"),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            536,
            636,
            110,
            36,
            hwnd,
            302 as _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        // ID 301: Save Changes
        CreateWindowExW(
            0,
            windows_sys::w!("BUTTON"),
            windows_sys::w!("Save Changes"),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            654,
            636,
            154,
            36,
            hwnd,
            301 as _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        // Start live 1-second monitoring timer
        SetTimer(hwnd, MONITOR_TIMER_ID, 1000, None);

        ShowWindow(hwnd, SW_SHOW);

        let mut msg: MSG = std::mem::zeroed();
        while IsWindow(hwnd) != 0 && GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &msg) == 0 {
                DispatchMessageW(&msg);
            }
        }
    }
}

unsafe fn paint_monitoring_section(hdc: HDC) {
    let card_rc = RECT {
        left: 20,
        top: 56,
        right: 808,
        bottom: 110,
    };
    draw_rounded_card(hdc, &card_rc, 10, card_brush(), border_pen());

    // 4 Column dividers inside monitoring card
    let col_w = (card_rc.right - card_rc.left) / 4;

    let div_pen = CreatePen(PS_SOLID, 1, 0x002B211A);
    let old_pen = SelectObject(hdc, div_pen);
    for i in 1..4 {
        let div_x = card_rc.left + i * col_w;
        MoveToEx(hdc, div_x, card_rc.top + 8, std::ptr::null_mut());
        LineTo(hdc, div_x, card_rc.bottom - 8);
    }
    SelectObject(hdc, old_pen);
    DeleteObject(div_pen);

    // Query real system metrics
    let ext_stats = crate::core::memory::get_extended_memory_stats();
    let base_stats = crate::core::memory::get_memory_stats();
    let cpu_percent = get_live_cpu_usage();
    let (hf, risk) = crate::core::telemetry::get_instant_fault_telemetry();

    let (used_mb, total_mb, used_pct, free_mb, standby_mb) = if let Ok(s) = ext_stats {
        (
            s.used_phys_mb,
            s.total_phys_mb,
            s.used_percent,
            s.free_phys_mb,
            s.standby_total_mb,
        )
    } else if let Ok(s) = base_stats {
        (
            s.used_phys_mb,
            s.total_phys_mb,
            s.used_percent,
            s.avail_phys_mb,
            0,
        )
    } else {
        (0, 0, 0, 0, 0)
    };

    SetBkMode(hdc, TRANSPARENT as i32);

    // --- Col 0: RAM IN USE ---
    let col0_x = card_rc.left + 14;
    SelectObject(hdc, font_badge());
    SetTextColor(hdc, COLOR_TEXT_MUTED);
    let mut lbl_rc0 = RECT {
        left: col0_x,
        top: card_rc.top + 8,
        right: col0_x + col_w - 20,
        bottom: card_rc.top + 20,
    };
    DrawTextW(
        hdc,
        windows_sys::w!("RAM IN USE"),
        -1,
        &mut lbl_rc0,
        DT_SINGLELINE,
    );

    SelectObject(hdc, font_stats_big());
    SetTextColor(hdc, COLOR_TEXT_PRIMARY);
    let used_gb = (used_mb as f64) / 1024.0;
    let total_gb = (total_mb as f64) / 1024.0;
    let ram_str = format!("{:.1} GB / {:.1} GB ({}%)", used_gb, total_gb, used_pct);
    let wide_ram: Vec<u16> = ram_str.encode_utf16().chain(std::iter::once(0)).collect();
    let mut val_rc0 = RECT {
        left: col0_x,
        top: card_rc.top + 22,
        right: col0_x + col_w - 20,
        bottom: card_rc.top + 38,
    };
    DrawTextW(
        hdc,
        wide_ram.as_ptr(),
        -1,
        &mut val_rc0,
        DT_SINGLELINE | DT_VCENTER,
    );

    // Mini RAM progress bar
    let bar_w = col_w - 30;
    let bar_h = 4;
    let bar_x = col0_x;
    let bar_y = card_rc.top + 40;
    let bar_bg = RECT {
        left: bar_x,
        top: bar_y,
        right: bar_x + bar_w,
        bottom: bar_y + bar_h,
    };
    let bg_br = CreateSolidBrush(COLOR_SURFACE);
    FillRect(hdc, &bar_bg, bg_br);
    DeleteObject(bg_br);

    let fill_w = ((bar_w as u32 * used_pct.min(100)) / 100) as i32;
    let bar_fill = RECT {
        left: bar_x,
        top: bar_y,
        right: bar_x + fill_w,
        bottom: bar_y + bar_h,
    };
    let bar_color = if used_pct > 85 {
        COLOR_DANGER
    } else if used_pct > 75 {
        COLOR_WARNING
    } else {
        COLOR_ACCENT
    };
    let fill_br = CreateSolidBrush(bar_color);
    FillRect(hdc, &bar_fill, fill_br);
    DeleteObject(fill_br);

    // --- Col 1: AVAILABLE RAM ---
    let col1_x = card_rc.left + col_w + 14;
    SelectObject(hdc, font_badge());
    SetTextColor(hdc, COLOR_TEXT_MUTED);
    let mut lbl_rc1 = RECT {
        left: col1_x,
        top: card_rc.top + 8,
        right: col1_x + col_w - 20,
        bottom: card_rc.top + 20,
    };
    DrawTextW(
        hdc,
        windows_sys::w!("AVAILABLE MEMORY"),
        -1,
        &mut lbl_rc1,
        DT_SINGLELINE,
    );

    SelectObject(hdc, font_stats_big());
    SetTextColor(hdc, COLOR_TEXT_PRIMARY);
    let avail_gb = ((total_mb.saturating_sub(used_mb)) as f64) / 1024.0;
    let avail_str = format!("{:.1} GB Free", avail_gb);
    let wide_avail: Vec<u16> = avail_str.encode_utf16().chain(std::iter::once(0)).collect();
    let mut val_rc1 = RECT {
        left: col1_x,
        top: card_rc.top + 22,
        right: col1_x + col_w - 20,
        bottom: card_rc.top + 38,
    };
    DrawTextW(
        hdc,
        wide_avail.as_ptr(),
        -1,
        &mut val_rc1,
        DT_SINGLELINE | DT_VCENTER,
    );

    SelectObject(hdc, font_desc());
    SetTextColor(hdc, COLOR_TEXT_SECONDARY);
    let sb_str = format!("Standby: {} MB · Free: {} MB", standby_mb, free_mb);
    let wide_sb: Vec<u16> = sb_str.encode_utf16().chain(std::iter::once(0)).collect();
    let mut sub_rc1 = RECT {
        left: col1_x,
        top: card_rc.top + 38,
        right: col1_x + col_w - 20,
        bottom: card_rc.top + 48,
    };
    DrawTextW(hdc, wide_sb.as_ptr(), -1, &mut sub_rc1, DT_SINGLELINE);

    // --- Col 2: CPU LOAD ---
    let col2_x = card_rc.left + 2 * col_w + 14;
    SelectObject(hdc, font_badge());
    SetTextColor(hdc, COLOR_TEXT_MUTED);
    let mut lbl_rc2 = RECT {
        left: col2_x,
        top: card_rc.top + 8,
        right: col2_x + col_w - 20,
        bottom: card_rc.top + 20,
    };
    DrawTextW(
        hdc,
        windows_sys::w!("CPU UTILIZATION"),
        -1,
        &mut lbl_rc2,
        DT_SINGLELINE,
    );

    SelectObject(hdc, font_stats_big());
    SetTextColor(hdc, COLOR_TEXT_PRIMARY);
    let cpu_str = format!("{}% System Load", cpu_percent);
    let wide_cpu: Vec<u16> = cpu_str.encode_utf16().chain(std::iter::once(0)).collect();
    let mut val_rc2 = RECT {
        left: col2_x,
        top: card_rc.top + 22,
        right: col2_x + col_w - 20,
        bottom: card_rc.top + 38,
    };
    DrawTextW(
        hdc,
        wide_cpu.as_ptr(),
        -1,
        &mut val_rc2,
        DT_SINGLELINE | DT_VCENTER,
    );

    SelectObject(hdc, font_desc());
    SetTextColor(hdc, COLOR_TEXT_SECONDARY);
    let mut sub_rc2 = RECT {
        left: col2_x,
        top: card_rc.top + 38,
        right: col2_x + col_w - 20,
        bottom: card_rc.top + 48,
    };
    DrawTextW(
        hdc,
        windows_sys::w!("Zero-Stutter Active"),
        -1,
        &mut sub_rc2,
        DT_SINGLELINE,
    );

    // --- Col 3: OPTIMIZATION HEALTH ---
    let col3_x = card_rc.left + 3 * col_w + 14;
    SelectObject(hdc, font_badge());
    SetTextColor(hdc, COLOR_TEXT_MUTED);
    let mut lbl_rc3 = RECT {
        left: col3_x,
        top: card_rc.top + 8,
        right: col3_x + col_w - 20,
        bottom: card_rc.top + 20,
    };
    DrawTextW(
        hdc,
        windows_sys::w!("SYSTEM HEALTH"),
        -1,
        &mut lbl_rc3,
        DT_SINGLELINE,
    );

    SelectObject(hdc, font_stats_big());
    let (health_tag, health_color) = match risk {
        crate::core::telemetry::StutterRiskLevel::Smooth => ("● Smooth (0 Thrash)", COLOR_SUCCESS),
        crate::core::telemetry::StutterRiskLevel::Moderate => {
            ("● Minor Faults", COLOR_WARNING)
        }
        crate::core::telemetry::StutterRiskLevel::Elevated => {
            ("● Stutter Risk", COLOR_WARNING)
        }
        crate::core::telemetry::StutterRiskLevel::Severe => {
            ("● High Thrashing", COLOR_DANGER)
        }
    };
    SetTextColor(hdc, health_color);
    let wide_health: Vec<u16> = health_tag
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut val_rc3 = RECT {
        left: col3_x,
        top: card_rc.top + 22,
        right: col3_x + col_w - 20,
        bottom: card_rc.top + 38,
    };
    DrawTextW(
        hdc,
        wide_health.as_ptr(),
        -1,
        &mut val_rc3,
        DT_SINGLELINE | DT_VCENTER,
    );

    SelectObject(hdc, font_desc());
    SetTextColor(hdc, COLOR_TEXT_SECONDARY);
    let fault_str = format!("Disk Faults: {}/s", hf);
    let wide_fault: Vec<u16> = fault_str.encode_utf16().chain(std::iter::once(0)).collect();
    let mut sub_rc3 = RECT {
        left: col3_x,
        top: card_rc.top + 38,
        right: col3_x + col_w - 20,
        bottom: card_rc.top + 48,
    };
    DrawTextW(hdc, wide_fault.as_ptr(), -1, &mut sub_rc3, DT_SINGLELINE);
}

unsafe extern "system" fn dialog_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => 1,
        WM_TIMER => {
            if wparam == MONITOR_TIMER_ID {
                // Refresh monitoring header
                let mon_rc = RECT {
                    left: 20,
                    top: 56,
                    right: 808,
                    bottom: 110,
                };
                InvalidateRect(hwnd, &mon_rc, 0);
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);

            // Double buffering
            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bm = CreateCompatibleBitmap(
                hdc,
                client_rc.right - client_rc.left,
                client_rc.bottom - client_rc.top,
            );
            let old_bm = SelectObject(mem_dc, mem_bm);

            // Background Fill #0B0D12
            FillRect(mem_dc, &client_rc, bg_brush());

            // Header Brand Area
            SetBkMode(mem_dc, TRANSPARENT as i32);
            SelectObject(mem_dc, font_title());
            SetTextColor(mem_dc, COLOR_TEXT_PRIMARY);
            let mut title_rc = RECT {
                left: 20,
                top: 14,
                right: 350,
                bottom: 36,
            };
            DrawTextW(
                mem_dc,
                windows_sys::w!("MemClean"),
                -1,
                &mut title_rc,
                DT_SINGLELINE | DT_VCENTER,
            );

            SelectObject(mem_dc, font_desc());
            SetTextColor(mem_dc, COLOR_TEXT_SECONDARY);
            let mut sub_rc = RECT {
                left: 120,
                top: 18,
                right: 450,
                bottom: 36,
            };
            DrawTextW(
                mem_dc,
                windows_sys::w!("Settings & Optimization Policy"),
                -1,
                &mut sub_rc,
                DT_SINGLELINE | DT_VCENTER,
            );

            // Admin Status Badge & Version Tag
            let is_admin = crate::core::privileges::is_elevated();
            let admin_str = if is_admin {
                windows_sys::w!("● Administrator Mode")
            } else {
                windows_sys::w!("● Standard User")
            };
            SelectObject(mem_dc, font_badge());
            SetTextColor(
                mem_dc,
                if is_admin {
                    COLOR_SUCCESS
                } else {
                    COLOR_WARNING
                },
            );
            let mut badge_rc = RECT {
                left: 560,
                top: 16,
                right: 730,
                bottom: 34,
            };
            DrawTextW(
                mem_dc,
                admin_str,
                -1,
                &mut badge_rc,
                DT_RIGHT | DT_SINGLELINE | DT_VCENTER,
            );

            SetTextColor(mem_dc, COLOR_TEXT_MUTED);
            let mut ver_rc = RECT {
                left: 740,
                top: 16,
                right: 808,
                bottom: 34,
            };
            DrawTextW(
                mem_dc,
                windows_sys::w!("v1.1.3"),
                -1,
                &mut ver_rc,
                DT_RIGHT | DT_SINGLELINE | DT_VCENTER,
            );

            // Compact Live System Monitoring Section
            paint_monitoring_section(mem_dc);

            // Two main column cards (X: 20..402, X: 418..800, Y: 172..616)
            let card_left = RECT {
                left: 20,
                top: 172,
                right: 402,
                bottom: 616,
            };
            let card_right = RECT {
                left: 418,
                top: 172,
                right: 800,
                bottom: 616,
            };

            draw_rounded_card(mem_dc, &card_left, 12, card_brush(), border_pen());
            draw_rounded_card(mem_dc, &card_right, 12, card_brush(), border_pen());

            // Card Section Headers & Dividers based on CURRENT_TAB
            SelectObject(mem_dc, font_subheader());
            SetTextColor(mem_dc, COLOR_TEXT_PRIMARY);

            match CURRENT_TAB {
                0 => {
                    // TAB 0: GENERAL
                    let mut h1 = RECT {
                        left: 34,
                        top: 186,
                        right: 388,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("TRAY ICON STYLE & PREVIEWS"),
                        -1,
                        &mut h1,
                        DT_SINGLELINE,
                    );

                    let mut h2 = RECT {
                        left: 34,
                        top: 324,
                        right: 388,
                        bottom: 342,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("COLOR PALETTE MODE"),
                        -1,
                        &mut h2,
                        DT_SINGLELINE,
                    );

                    let mut h3 = RECT {
                        left: 34,
                        top: 398,
                        right: 388,
                        bottom: 416,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("TRAY CLICK ACTION"),
                        -1,
                        &mut h3,
                        DT_SINGLELINE,
                    );

                    let mut hr1 = RECT {
                        left: 432,
                        top: 186,
                        right: 786,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("SYSTEM BEHAVIOR & STARTUP"),
                        -1,
                        &mut hr1,
                        DT_SINGLELINE,
                    );

                    let mut hr2 = RECT {
                        left: 432,
                        top: 482,
                        right: 786,
                        bottom: 500,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("STORAGE HARDWARE PRESET"),
                        -1,
                        &mut hr2,
                        DT_SINGLELINE,
                    );
                }
                1 => {
                    // TAB 1: AUTOMATION
                    let mut h1 = RECT {
                        left: 34,
                        top: 186,
                        right: 388,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("RAM AUTO-OPTIMIZATION THRESHOLD"),
                        -1,
                        &mut h1,
                        DT_SINGLELINE,
                    );

                    let mut h2 = RECT {
                        left: 34,
                        top: 298,
                        right: 388,
                        bottom: 316,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("PERIODIC AUTO-CLEAN TIMER"),
                        -1,
                        &mut h2,
                        DT_SINGLELINE,
                    );

                    let mut hr1 = RECT {
                        left: 432,
                        top: 186,
                        right: 786,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("INTELLIGENT STANDBY LIST CLEANER (ISLC)"),
                        -1,
                        &mut hr1,
                        DT_SINGLELINE,
                    );

                    let mut hr2 = RECT {
                        left: 432,
                        top: 326,
                        right: 786,
                        bottom: 344,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("USER INACTIVITY & AFK TRIGGER"),
                        -1,
                        &mut hr2,
                        DT_SINGLELINE,
                    );
                }
                2 => {
                    // TAB 2: KERNEL & GAMING
                    let mut h1 = RECT {
                        left: 34,
                        top: 186,
                        right: 388,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("KERNEL TIMER RESOLUTION LOCK"),
                        -1,
                        &mut h1,
                        DT_SINGLELINE,
                    );

                    let (cur_res, _) = crate::core::ntdll::query_timer_resolution()
                        .map(|(_, _, cur)| ((cur as f64) / 10000.0, true))
                        .unwrap_or((15.625, false));
                    let res_label = format!("Live System Timer: {:.3} ms", cur_res);
                    let wide_res: Vec<u16> =
                        res_label.encode_utf16().chain(std::iter::once(0)).collect();
                    SelectObject(mem_dc, font_desc());
                    SetTextColor(mem_dc, COLOR_SUCCESS);
                    let mut res_rc = RECT {
                        left: 230,
                        top: 186,
                        right: 388,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        wide_res.as_ptr(),
                        -1,
                        &mut res_rc,
                        DT_RIGHT | DT_SINGLELINE,
                    );

                    SelectObject(mem_dc, font_subheader());
                    SetTextColor(mem_dc, COLOR_TEXT_PRIMARY);

                    let mut hr1 = RECT {
                        left: 432,
                        top: 186,
                        right: 786,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("ADVANCED HARDWARE OPTIMIZATIONS"),
                        -1,
                        &mut hr1,
                        DT_SINGLELINE,
                    );

                    let mut hr2 = RECT {
                        left: 432,
                        top: 348,
                        right: 786,
                        bottom: 366,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("COMPETITIVE GAMING NETWORK STACK"),
                        -1,
                        &mut hr2,
                        DT_SINGLELINE,
                    );
                }
                _ => {
                    // TAB 3: SHORTCUTS
                    let mut h1 = RECT {
                        left: 34,
                        top: 186,
                        right: 388,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("GLOBAL APPLICATION SHORTCUTS"),
                        -1,
                        &mut h1,
                        DT_SINGLELINE,
                    );

                    let mut hr1 = RECT {
                        left: 432,
                        top: 186,
                        right: 786,
                        bottom: 204,
                    };
                    DrawTextW(
                        mem_dc,
                        windows_sys::w!("KEYBOARD NAVIGATION & CHEATSHEET"),
                        -1,
                        &mut hr1,
                        DT_SINGLELINE,
                    );
                }
            }

            // Footer Version Info
            SelectObject(mem_dc, font_desc());
            SetTextColor(mem_dc, COLOR_TEXT_MUTED);
            let mut foot_rc = RECT {
                left: 176,
                top: 644,
                right: 520,
                bottom: 664,
            };
            DrawTextW(
                mem_dc,
                windows_sys::w!("MemClean v1.1.3 · Windows 11 Kernel Engine · High-Performance Mode"),
                -1,
                &mut foot_rc,
                DT_SINGLELINE | DT_VCENTER,
            );

            // Blit to screen DC
            BitBlt(
                hdc,
                0,
                0,
                client_rc.right - client_rc.left,
                client_rc.bottom - client_rc.top,
                mem_dc,
                0,
                0,
                SRCCOPY,
            );

            SelectObject(mem_dc, old_bm);
            DeleteObject(mem_bm);
            DeleteDC(mem_dc);

            EndPaint(hwnd, &mut ps);
            0
        }
        WM_CTLCOLORDLG => bg_brush() as LRESULT,
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, COLOR_TEXT_PRIMARY);
            card_brush() as LRESULT
        }
        WM_CTLCOLOREDIT => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, COLOR_TEXT_PRIMARY);
            SetBkColor(hdc, COLOR_SURFACE);
            surface_brush() as LRESULT
        }
        WM_CTLCOLORBTN => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, COLOR_TEXT_PRIMARY);
            card_brush() as LRESULT
        }
        WM_DRAWITEM => {
            let dis = &*(lparam as *const DRAWITEMSTRUCT);
            let is_selected = (dis.itemState & ODS_SELECTED) != 0;
            let is_hover = dis.CtlID as i32 == HOVERED_BTN;

            // Navigation Tabs (601..=604)
            if dis.CtlID >= 601 && dis.CtlID <= 604 {
                let tab_idx = (dis.CtlID - 601) as usize;
                let is_active = tab_idx == CURRENT_TAB;
                let bg_color = if is_active {
                    COLOR_SURFACE
                } else if is_hover {
                    COLOR_SURFACE_HOVER
                } else {
                    COLOR_CARD
                };
                let border_color = if is_active {
                    COLOR_ACCENT
                } else if is_hover {
                    COLOR_BORDER
                } else {
                    COLOR_BORDER
                };
                let text_color = if is_active {
                    COLOR_TEXT_PRIMARY
                } else {
                    COLOR_TEXT_SECONDARY
                };

                let brush = CreateSolidBrush(bg_color);
                let pen = CreatePen(PS_SOLID, 1, border_color);
                draw_rounded_card(dis.hDC, &dis.rcItem, 8, brush, pen);
                DeleteObject(pen);
                DeleteObject(brush);

                // Active bottom pill indicator
                if is_active {
                    let ind_rc = RECT {
                        left: dis.rcItem.left + 24,
                        top: dis.rcItem.bottom - 3,
                        right: dis.rcItem.right - 24,
                        bottom: dis.rcItem.bottom,
                    };
                    let ind_brush = CreateSolidBrush(COLOR_ACCENT);
                    FillRect(dis.hDC, &ind_rc, ind_brush);
                    DeleteObject(ind_brush);
                }

                SetBkMode(dis.hDC, TRANSPARENT as i32);
                SetTextColor(dis.hDC, text_color);
                let old_font =
                    SelectObject(dis.hDC, if is_active { font_bold() } else { font_ui() });

                let title = match tab_idx {
                    0 => windows_sys::w!("⚙ General"),
                    1 => windows_sys::w!("⚡ Automation"),
                    2 => windows_sys::w!("🎮 Kernel & Gaming"),
                    _ => windows_sys::w!("⌨ Shortcuts"),
                };
                let mut rc = dis.rcItem;
                DrawTextW(
                    dis.hDC,
                    title,
                    -1,
                    &mut rc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
                SelectObject(dis.hDC, old_font);
                return 1;
            }

            // Footer Action Buttons
            if dis.CtlID == 301 {
                draw_action_btn(
                    dis.hDC,
                    &dis.rcItem,
                    "💾 Save Changes",
                    true,
                    false,
                    is_hover,
                    is_selected,
                );
                return 1;
            } else if dis.CtlID == 302 {
                draw_action_btn(
                    dis.hDC,
                    &dis.rcItem,
                    "Cancel",
                    false,
                    false,
                    is_hover,
                    is_selected,
                );
                return 1;
            } else if dis.CtlID == 303 {
                draw_action_btn(
                    dis.hDC,
                    &dis.rcItem,
                    "↺ Restore Defaults",
                    false,
                    true,
                    is_hover,
                    is_selected,
                );
                return 1;
            }

            // Tray Style Preview Cards (101, 102, 103)
            if dis.CtlID >= 101 && dis.CtlID <= 103 {
                let style_idx = (dis.CtlID - 101) as usize;
                let active = is_checked(hwnd, dis.CtlID as i32);
                let title = match style_idx {
                    0 => "Floating Number",
                    1 => "Capsule Badge",
                    _ => "Monogram Logo",
                };
                draw_tray_preview_card(dis.hDC, &dis.rcItem, style_idx, title, active, is_hover);
                return 1;
            }

            // Color Palette Segmented Buttons (104, 105)
            if dis.CtlID == 104 || dis.CtlID == 105 {
                let active = is_checked(hwnd, dis.CtlID as i32);
                let text = if dis.CtlID == 104 {
                    "● Dynamic Color"
                } else {
                    "○ Monochrome"
                };
                draw_segmented_btn(dis.hDC, &dis.rcItem, text, active, is_hover);
                return 1;
            }

            // Tray Click Action Segmented Buttons (201, 202)
            if dis.CtlID == 201 || dis.CtlID == 202 {
                let active = is_checked(hwnd, dis.CtlID as i32);
                let text = if dis.CtlID == 201 {
                    "📊 Open Dashboard"
                } else {
                    "⚡ Quick Optimize"
                };
                draw_segmented_btn(dis.hDC, &dis.rcItem, text, active, is_hover);
                return 1;
            }

            // Timer Resolution Segmented Buttons (240, 241, 242)
            if dis.CtlID >= 240 && dis.CtlID <= 242 {
                let active = is_checked(hwnd, dis.CtlID as i32);
                let text = match dis.CtlID {
                    240 => "Default (15.6 ms - Windows Standard)",
                    241 => "1.0 ms Multimedia Sync (Audio/Video)",
                    _ => "0.5 ms Competitive Gaming (500 µs Low Latency)",
                };
                draw_segmented_btn(dis.hDC, &dis.rcItem, text, active, is_hover);
                return 1;
            }

            // Action Buttons
            if dis.CtlID == 250 && CURRENT_TAB == 0 {
                draw_action_btn(
                    dis.hDC,
                    &dis.rcItem,
                    "⚡ Apply Best Defaults for Your Device",
                    false,
                    false,
                    is_hover,
                    is_selected,
                );
                return 1;
            } else if dis.CtlID == 261 {
                draw_action_btn(
                    dis.hDC,
                    &dis.rcItem,
                    "↺ Restore Defaults",
                    false,
                    true,
                    is_hover,
                    is_selected,
                );
                return 1;
            } else if dis.CtlID == 403 {
                draw_action_btn(
                    dis.hDC,
                    &dis.rcItem,
                    "⚡ Test Quick Optimization Now",
                    false,
                    false,
                    is_hover,
                    is_selected,
                );
                return 1;
            }

            // Toggle Switches
            let (title, desc) = match dis.CtlID {
                216 => (
                    "Run at Windows Startup",
                    "Launch MemClean automatically when signing in",
                ),
                203 => (
                    "Start Minimized",
                    "Launch silently into system notification tray",
                ),
                214 => (
                    "Check for Updates",
                    "Automatically check for new releases on startup",
                ),
                204 => (
                    "Global Quick Clean Hotkey",
                    "Press Ctrl + Alt + M to clean RAM from any application",
                ),
                215 => (
                    "Gaming HUD Overlay",
                    "Press Ctrl + Shift + O to display real-time FPS & RAM overlay",
                ),
                209 => (
                    "Periodic Auto-Clean Timer",
                    "Regularly optimizes physical memory at configured interval",
                ),
                220 => (
                    "ISLC Standby List Cleaner",
                    "Purges standby cache when standby is high and free RAM low",
                ),
                230 => (
                    "Auto-Clean on Inactivity",
                    "Performs gentle working-set trim when system is idle",
                ),
                243 => (
                    "Unpark 100% CPU Cores",
                    "Eliminates core parking wake-up stutter for burst gaming load",
                ),
                250 if CURRENT_TAB == 2 => (
                    "Kernel Page Combining",
                    "Merges duplicate memory pages to recover 200MB - 1GB RAM",
                ),
                270 => (
                    "Trim GPU / VRAM Allocations",
                    "Discards orphaned video memory buffers on clean (WDDM 2.1+)",
                ),
                275 => (
                    "Hybrid CPU Scheduling",
                    "Locks gaming threads to P-Cores and bloatware to E-Cores",
                ),
                260 => (
                    "Competitive Gaming TCP Stack",
                    "Disables Nagle's algorithm and network throttling index",
                ),
                401 => (
                    "Global Quick Clean Hotkey",
                    "Ctrl + Alt + M: Sub-millisecond direct kernel clean",
                ),
                402 => (
                    "Gaming HUD Micro-Overlay",
                    "Ctrl + Shift + O: In-game lightweight metrics HUD",
                ),
                _ => ("", ""),
            };

            if !title.is_empty() {
                let on = is_checked(hwnd, dis.CtlID as i32);
                draw_toggle_row(dis.hDC, &dis.rcItem, title, desc, on, is_hover);
                return 1;
            }

            0
        }
        WM_MOUSEMOVE => {
            let pt = POINT {
                x: (lparam & 0xFFFF) as i16 as i32,
                y: ((lparam >> 16) & 0xFFFF) as i16 as i32,
            };
            let child =
                windows_sys::Win32::UI::WindowsAndMessaging::ChildWindowFromPoint(hwnd, pt);
            let raw = if !child.is_null() {
                GetDlgCtrlID(child)
            } else {
                0
            };
            let new_hover = raw;
            if new_hover != HOVERED_BTN {
                let old = HOVERED_BTN;
                HOVERED_BTN = new_hover;
                if old != 0 {
                    let btn = GetDlgItem(hwnd, old as i32);
                    if !btn.is_null() {
                        InvalidateRect(btn, std::ptr::null(), 1);
                    }
                }
                if new_hover != 0 {
                    let btn = GetDlgItem(hwnd, new_hover as i32);
                    if !btn.is_null() {
                        InvalidateRect(btn, std::ptr::null(), 1);
                    }
                }
            }
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            TrackMouseEvent(&mut tme);
            0
        }
        WM_MOUSELEAVE => {
            let old = HOVERED_BTN;
            HOVERED_BTN = 0;
            if old != 0 {
                let btn = GetDlgItem(hwnd, old as i32);
                if !btn.is_null() {
                    InvalidateRect(btn, std::ptr::null(), 1);
                }
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as u32;

            // Navigation tabs switching
            if id >= 601 && id <= 604 {
                switch_tab(hwnd, (id - 601) as usize);
                return 0;
            }

            // Restore Defaults Confirmation Dialog (ID 303)
            if id == 303 {
                let confirm = MessageBoxW(
                    hwnd,
                    windows_sys::w!("Are you sure you want to restore all optimization settings to factory defaults?\n\nThis will reset auto-clean triggers, kernel policies, and tray styling."),
                    windows_sys::w!("MemClean – Restore Defaults"),
                    MB_YESNO | MB_ICONQUESTION,
                );
                if confirm == IDYES {
                    let cfg = AppConfig::default();
                    let _ = cfg.save();
                    tab_system::load(hwnd, &cfg);
                    tab_triggers::load(hwnd, &cfg);
                    tab_kernel::load(hwnd, &cfg);
                    tab_shortcuts::load(hwnd, &cfg);
                    InvalidateRect(hwnd, std::ptr::null(), 1);
                }
                return 0;
            }

            // Cancel Button (ID 302)
            if id == 302 {
                DestroyWindow(hwnd);
                return 0;
            }

            // Best for Device Button (ID 250 in Tab 0)
            if id == 250 && CURRENT_TAB == 0 {
                let mut cfg = AppConfig::load();
                let is_ssd = matches!(
                    crate::core::disk::detect_system_drive_type(),
                    crate::core::disk::DriveType::Ssd
                );
                cfg.apply_device_defaults(is_ssd);
                let _ = cfg.save();
                tab_system::load(hwnd, &cfg);
                tab_triggers::load(hwnd, &cfg);
                InvalidateRect(hwnd, std::ptr::null(), 1);
                return 0;
            }

            // Delegations to tabs
            let mut cfg = AppConfig::load();
            if tab_kernel::handle_command(hwnd, id as i32, &mut cfg) {
                InvalidateRect(hwnd, std::ptr::null(), 1);
                return 0;
            }
            if tab_shortcuts::handle_command(hwnd, id as i32, &mut cfg) {
                return 0;
            }

            // Tray Style Radio Group (101, 102, 103)
            if id >= 101 && id <= 103 {
                for i in 101..=103 {
                    set_checked(hwnd, i, i == id as i32);
                }
                return 0;
            }

            // Color Palette Radio Group (104, 105)
            if id == 104 || id == 105 {
                set_checked(hwnd, 104, id == 104);
                set_checked(hwnd, 105, id == 105);
                return 0;
            }

            // Tray Click Action Radio Group (201, 202)
            if id == 201 || id == 202 {
                set_checked(hwnd, 201, id == 201);
                set_checked(hwnd, 202, id == 202);
                return 0;
            }

            // Timer Resolution Radio Group (240, 241, 242)
            if id >= 240 && id <= 242 {
                for i in 240..=242 {
                    set_checked(hwnd, i, i == id as i32);
                }
                return 0;
            }

            // Toggle Switches
            let toggle_ids = [
                203, 204, 214, 215, 216, 209, 220, 230, 243, 270, 275, 260, 401, 402,
            ];
            if toggle_ids.contains(&(id as i32)) || (id == 250 && CURRENT_TAB == 2) {
                let cur = is_checked(hwnd, id as i32);
                set_checked(hwnd, id as i32, !cur);

                // Mirror hotkey state between Tab 0 and Tab 3
                if id == 204 || id == 401 {
                    set_checked(hwnd, 204, !cur);
                    set_checked(hwnd, 401, !cur);
                }
                if id == 215 || id == 402 {
                    set_checked(hwnd, 215, !cur);
                    set_checked(hwnd, 402, !cur);
                }

                let ctrl = GetDlgItem(hwnd, id as i32);
                if !ctrl.is_null() {
                    InvalidateRect(ctrl, std::ptr::null(), 1);
                }
                return 0;
            }

            // Save Changes (ID 301)
            if id == 301 {
                tab_system::save(hwnd, &mut cfg);
                tab_triggers::save(hwnd, &mut cfg);
                tab_kernel::save(hwnd, &mut cfg);
                tab_shortcuts::save(hwnd, &mut cfg);

                let _ = cfg.save();
                DestroyWindow(hwnd);
                return 0;
            }

            0
        }
        WM_DESTROY => {
            KillTimer(hwnd, MONITOR_TIMER_ID);
            free_resources();
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
