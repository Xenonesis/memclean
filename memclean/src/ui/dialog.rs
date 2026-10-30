use crate::config::{AppConfig, TrayClickAction, TrayColorMode, TrayStyle};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, PAINTSTRUCT,
    CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, LineTo, MoveToEx,
    RoundRect, SelectObject, SetBkMode, SetTextColor, PS_SOLID, TRANSPARENT, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetDlgItem, GetMessageW,
    GetWindowTextW, IsDialogMessageW, IsWindow, LoadIconW, RegisterClassW, SendMessageW, SetWindowTextW, ShowWindow,
    BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_OWNERDRAW, ES_CENTER, ES_NUMBER, MSG, SW_SHOW,
    WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DESTROY,
    WM_DRAWITEM, WM_ERASEBKGND, WM_PAINT, WM_SETFONT, WM_SETICON, WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
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
const ODS_SELECTED: u32 = 0x0001;

static mut D_BG_BRUSH: windows_sys::Win32::Graphics::Gdi::HBRUSH = std::ptr::null_mut();
static mut D_CARD_BRUSH: windows_sys::Win32::Graphics::Gdi::HBRUSH = std::ptr::null_mut();
static mut D_BORDER_PEN: windows_sys::Win32::Graphics::Gdi::HPEN = std::ptr::null_mut();
static mut D_FONT_UI: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();
static mut D_FONT_BOLD: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();
static mut D_FONT_SUBHEADER: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();

const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;

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

pub fn show_settings_dialog(_parent: HWND) {
    unsafe {
        let class_name = windows_sys::w!("MemCleanSettingsClass");
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
        let mut app_icon = LoadIconW(hinstance, 1 as _);
        if app_icon.is_null() {
            app_icon = crate::ui::tray::load_embedded_icon(32);
        }
        D_BG_BRUSH = CreateSolidBrush(0x00FAFBFB); // Warm Bone #FBFBFA
        D_CARD_BRUSH = CreateSolidBrush(0x00FFFFFF); // Pure White #FFFFFF
        D_BORDER_PEN = CreatePen(PS_SOLID as i32, 1, 0x00EAEAEA); // Hairline #EAEAEA

        D_FONT_UI = CreateFontW(
            -13, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        D_FONT_BOLD = CreateFontW(
            -13, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        D_FONT_SUBHEADER = CreateFontW(
            -11, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(dialog_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: app_icon,
            hCursor: std::ptr::null_mut(),
            hbrBackground: D_BG_BRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wc);

        let width = 380;
        let height = 830;
        let screen_w = windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows_sys::Win32::UI::WindowsAndMessaging::SM_CXSCREEN);
        let screen_h = windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows_sys::Win32::UI::WindowsAndMessaging::SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let hwnd = CreateWindowExW(
            0,
            class_name,
            windows_sys::w!("MemClean Settings"),
            WS_OVERLAPPEDWINDOW & !(0x00010000 | 0x00040000), // Disable maximize & resize
            x, y, width, height,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        let small_icon = crate::ui::tray::load_embedded_icon(16);
        let big_icon = crate::ui::tray::load_embedded_icon(32);
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

        let create_radio = |text: *const u16, cy: i32, id: i32| -> HWND {
            let ctrl = CreateWindowExW(
                0, windows_sys::w!("BUTTON"), text,
                WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON as u32,
                24, cy, 316, 20, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut(),
            );
            if !ctrl.is_null() && !D_FONT_UI.is_null() {
                SendMessageW(ctrl, WM_SETFONT, D_FONT_UI as WPARAM, 1);
            }
            ctrl
        };

        let create_chk = |text: *const u16, cy: i32, id: i32| -> HWND {
            let ctrl = CreateWindowExW(
                0, windows_sys::w!("BUTTON"), text,
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
                24, cy, 316, 20, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut(),
            );
            if !ctrl.is_null() && !D_FONT_UI.is_null() {
                SendMessageW(ctrl, WM_SETFONT, D_FONT_UI as WPARAM, 1);
            }
            ctrl
        };

        // Card 1: Tray Icon Style
        create_radio(windows_sys::w!("Floating Transparent Number (Minimalist)"), 40, 101);
        create_radio(windows_sys::w!("Rounded Capsule Badge (Dark Matte)"), 64, 102);
        create_radio(windows_sys::w!("Monogram Logo Icon (Concept B)"), 88, 103);

        // Card 1: Color Mode
        create_radio(windows_sys::w!("Dynamic Color (Green / Amber / Red)"), 132, 104);
        create_radio(windows_sys::w!("Monochrome White (Distraction-Free)"), 156, 105);

        // Card 2: Tray Behavior & Startup
        create_radio(windows_sys::w!("Click Opens / Restores Dashboard"), 226, 201);
        create_radio(windows_sys::w!("Click Optimizes Memory Instantly"), 248, 202);
        create_chk(windows_sys::w!("Enable Global Hotkey (Ctrl + Alt + M)"), 284, 204);
        create_chk(windows_sys::w!("Start Minimized to System Tray on Login"), 306, 203);
        create_chk(windows_sys::w!("Automatically check for updates on startup"), 328, 214);

        // Card 2: RAM Threshold Controls
        let lbl_thresh = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Auto-optimize when RAM exceeds:"),
            WS_CHILD | WS_VISIBLE,
            24, 355, 226, 20, hwnd, 206 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_thresh.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_thresh, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_thresh = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("80"),
            WS_CHILD | WS_VISIBLE | windows_sys::Win32::UI::WindowsAndMessaging::WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            254, 352, 44, 22, hwnd, 205 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_thresh.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_thresh, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_pct = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("%"),
            WS_CHILD | WS_VISIBLE,
            304, 355, 20, 20, hwnd, 207 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_pct.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_pct, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_note = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("(Range: 10% - 95%, Default: 80%)"),
            WS_CHILD | WS_VISIBLE,
            24, 380, 316, 16, hwnd, 208 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_note.is_null() && !D_FONT_SUBHEADER.is_null() {
            SendMessageW(lbl_note, WM_SETFONT, D_FONT_SUBHEADER as WPARAM, 1);
        }
        // Card 2: Periodic Timer Controls
        create_chk(windows_sys::w!("Enable periodic auto-clean timer"), 420, 209);

        let lbl_timer = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Optimize interval:"),
            WS_CHILD | WS_VISIBLE,
            24, 448, 136, 20, hwnd, 210 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_timer.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_timer, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_timer = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("15"),
            WS_CHILD | WS_VISIBLE | windows_sys::Win32::UI::WindowsAndMessaging::WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            164, 445, 50, 22, hwnd, 211 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_timer.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_timer, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_secs = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("seconds"),
            WS_CHILD | WS_VISIBLE,
            220, 448, 70, 20, hwnd, 212 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_secs.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_secs, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_note2 = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("(0 = Continuous / No limit, Range: 1 - 300s)"),
            WS_CHILD | WS_VISIBLE,
            24, 472, 316, 16, hwnd, 213 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_note2.is_null() && !D_FONT_SUBHEADER.is_null() {
            SendMessageW(lbl_note2, WM_SETFONT, D_FONT_SUBHEADER as WPARAM, 1);
        }
        // Card 2: ISLC Dual-Condition Standby Controls
        create_chk(windows_sys::w!("Enable ISLC Standby Trigger (Gaming)"), 496, 220);

        let lbl_islc_s = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Standby >="),
            WS_CHILD | WS_VISIBLE,
            24, 520, 75, 20, hwnd, 223 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_islc_s.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_islc_s, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_islc_standby = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("1024"),
            WS_CHILD | WS_VISIBLE | windows_sys::Win32::UI::WindowsAndMessaging::WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            102, 518, 50, 22, hwnd, 221 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_islc_standby.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_islc_standby, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_islc_s_mb = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("MB"),
            WS_CHILD | WS_VISIBLE,
            156, 520, 24, 20, hwnd, 224 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_islc_s_mb.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_islc_s_mb, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_islc_f = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Free <="),
            WS_CHILD | WS_VISIBLE,
            190, 520, 55, 20, hwnd, 225 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_islc_f.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_islc_f, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_islc_free = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("1024"),
            WS_CHILD | WS_VISIBLE | windows_sys::Win32::UI::WindowsAndMessaging::WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            248, 518, 50, 22, hwnd, 222 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_islc_free.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_islc_free, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_islc_f_mb = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("MB"),
            WS_CHILD | WS_VISIBLE,
            302, 520, 24, 20, hwnd, 226 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_islc_f_mb.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_islc_f_mb, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        // Card 2: Inactivity / AFK Idle Clean
        create_chk(windows_sys::w!("Auto-clean on user inactivity / AFK"), 548, 230);

        let lbl_idle = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Idle timeout:"),
            WS_CHILD | WS_VISIBLE,
            24, 572, 85, 20, hwnd, 232 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_idle.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_idle, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_idle = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("300"),
            WS_CHILD | WS_VISIBLE | windows_sys::Win32::UI::WindowsAndMessaging::WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            112, 570, 50, 22, hwnd, 231 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_idle.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_idle, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_idle_sec = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("seconds (Default: 300s)"),
            WS_CHILD | WS_VISIBLE,
            168, 572, 160, 20, hwnd, 233 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_idle_sec.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_idle_sec, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        // Card 3: Scheduler Timer Resolution & Kernel Tuning
        create_radio(windows_sys::w!("Default Timer Resolution (15.6 ms)"), 634, 240);
        create_radio(windows_sys::w!("1.0 ms Multimedia Latency (1,000 µs)"), 656, 241);
        create_radio(windows_sys::w!("0.5 ms Competitive Gaming (500 µs)"), 678, 242);
        create_chk(windows_sys::w!("Enable Kernel Page Combining (de-dup)"), 704, 250);

        // Load current config into controls
        let cfg = AppConfig::load();
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
        let thresh_str = format!("{}\0", cfg.triggers.threshold_percent);
        let thresh_wide: Vec<u16> = thresh_str.encode_utf16().collect();
        SetWindowTextW(edit_thresh, thresh_wide.as_ptr());

        set_checked(hwnd, 209, cfg.triggers.auto_timer_enabled);
        let timer_str = format!("{}\0", cfg.triggers.timer_seconds);
        let timer_wide: Vec<u16> = timer_str.encode_utf16().collect();
        SetWindowTextW(edit_timer, timer_wide.as_ptr());

        set_checked(hwnd, 220, cfg.triggers.islc_enabled);
        let islc_s_str = format!("{}\0", cfg.triggers.islc_standby_mb);
        let islc_s_wide: Vec<u16> = islc_s_str.encode_utf16().collect();
        SetWindowTextW(edit_islc_standby, islc_s_wide.as_ptr());

        let islc_f_str = format!("{}\0", cfg.triggers.islc_free_mb);
        let islc_f_wide: Vec<u16> = islc_f_str.encode_utf16().collect();
        SetWindowTextW(edit_islc_free, islc_f_wide.as_ptr());

        set_checked(hwnd, 230, cfg.triggers.idle_clean_enabled);
        let idle_str = format!("{}\0", cfg.triggers.idle_seconds);
        let idle_wide: Vec<u16> = idle_str.encode_utf16().collect();
        SetWindowTextW(edit_idle, idle_wide.as_ptr());

        match cfg.tuning.timer_mode_enum() {
            crate::core::timer_res::TimerResolutionMode::HalfMs => set_checked(hwnd, 242, true),
            crate::core::timer_res::TimerResolutionMode::OneMs => set_checked(hwnd, 241, true),
            crate::core::timer_res::TimerResolutionMode::Default => set_checked(hwnd, 240, true),
        }
        set_checked(hwnd, 250, cfg.tuning.combine_pages_on_clean);

        // Flat Minimalist Apply & Close button (BS_OWNERDRAW)
        let btn = CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Apply and Close"),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            12, 744, 340, 36, hwnd, 301 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !btn.is_null() && !D_FONT_BOLD.is_null() {
            SendMessageW(btn, WM_SETFONT, D_FONT_BOLD as WPARAM, 1);
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
            FillRect(hdc, &client_rc, D_BG_BRUSH);

            // Card 1: Tray Appearance (12, 10, 352, 192)
            let old_brush = SelectObject(hdc, D_CARD_BRUSH);
            let old_pen = SelectObject(hdc, D_BORDER_PEN);
            RoundRect(hdc, 12, 10, 352, 192, 10, 10);

            // Card 1 Header: TRAY APPEARANCE
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00747778); // Muted gray #787774
            let old_font = SelectObject(hdc, D_FONT_SUBHEADER);
            let header1 = windows_sys::w!("TRAY ICON STYLE");
            let mut h1_rc = RECT { left: 24, top: 18, right: 340, bottom: 32 };
            DrawTextW(hdc, header1, -1, &mut h1_rc, DT_SINGLELINE);

            // Sub-divider for color mode
            MoveToEx(hdc, 24, 116, std::ptr::null_mut());
            LineTo(hdc, 340, 116);

            let header_color = windows_sys::w!("COLOR MODE");
            let mut hc_rc = RECT { left: 24, top: 120, right: 340, bottom: 132 };
            DrawTextW(hdc, header_color, -1, &mut hc_rc, DT_SINGLELINE);

            // Card 2: Optimization Triggers (12, 202, 352, 608)
            SelectObject(hdc, D_CARD_BRUSH);
            SelectObject(hdc, D_BORDER_PEN);
            RoundRect(hdc, 12, 202, 352, 608, 10, 10);
            // Card 2 Header: TRAY INTERACTION & STARTUP
            SetTextColor(hdc, 0x00747778);
            let header2 = windows_sys::w!("TRAY INTERACTION && STARTUP");
            let mut h2_rc = RECT { left: 24, top: 210, right: 340, bottom: 224 };
            DrawTextW(hdc, header2, -1, &mut h2_rc, DT_SINGLELINE);

            // Sub-divider for Hotkey & Startup
            MoveToEx(hdc, 24, 268, std::ptr::null_mut());
            LineTo(hdc, 340, 268);

            // Sub-divider for Threshold
            MoveToEx(hdc, 24, 324, std::ptr::null_mut());
            LineTo(hdc, 340, 324);

            let header_thresh = windows_sys::w!("RAM AUTO-OPTIMIZATION THRESHOLD");
            let mut ht_rc = RECT { left: 24, top: 328, right: 340, bottom: 342 };
            DrawTextW(hdc, header_thresh, -1, &mut ht_rc, DT_SINGLELINE);

            // Sub-divider for Timer
            MoveToEx(hdc, 24, 396, std::ptr::null_mut());
            LineTo(hdc, 340, 396);

            let header_timer = windows_sys::w!("PERIODIC AUTO-OPTIMIZATION TIMER");
            let mut htm_rc = RECT { left: 24, top: 400, right: 340, bottom: 414 };
            DrawTextW(hdc, header_timer, -1, &mut htm_rc, DT_SINGLELINE);

            // Sub-divider for ISLC
            MoveToEx(hdc, 24, 480, std::ptr::null_mut());
            LineTo(hdc, 340, 480);

            let header_islc = windows_sys::w!("INTELLIGENT STANDBY (ISLC)");
            let mut hislc_rc = RECT { left: 24, top: 484, right: 340, bottom: 498 };
            DrawTextW(hdc, header_islc, -1, &mut hislc_rc, DT_SINGLELINE);

            // Sub-divider for Idle Clean
            MoveToEx(hdc, 24, 538, std::ptr::null_mut());
            LineTo(hdc, 340, 538);

            // Card 3: Scheduler Timer Resolution & Tuning (12, 618, 352, 736)
            SelectObject(hdc, D_CARD_BRUSH);
            SelectObject(hdc, D_BORDER_PEN);
            RoundRect(hdc, 12, 618, 352, 736, 10, 10);
            SetTextColor(hdc, 0x00747778);
            let header3 = windows_sys::w!("SCHEDULER TIMER RESOLUTION && KERNEL TUNING");
            let mut h3_rc = RECT { left: 24, top: 624, right: 340, bottom: 638 };
            DrawTextW(hdc, header3, -1, &mut h3_rc, DT_SINGLELINE);

            SelectObject(hdc, old_brush);
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_font);
            EndPaint(hwnd, &ps);
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            let ctrl_id = windows_sys::Win32::UI::WindowsAndMessaging::GetDlgCtrlID(lparam as HWND);
            if ctrl_id == 208 || ctrl_id == 213 || ctrl_id == 233 {
                SetTextColor(hdc, 0x00747778); // Muted helper text
            } else {
                SetTextColor(hdc, 0x00111111);
            }
            D_CARD_BRUSH as LRESULT // Controls match white card surface
        }
        WM_CTLCOLORBTN => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00111111);
            D_CARD_BRUSH as LRESULT
        }
        WM_CTLCOLORDLG => {
            D_BG_BRUSH as LRESULT
        }
        WM_CTLCOLOREDIT => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00111111);
            D_CARD_BRUSH as LRESULT
        }
        WM_DRAWITEM => {
            let dis = &*(lparam as *const DRAWITEMSTRUCT);
            if dis.CtlID == 301 {
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
                let old_font = SelectObject(dis.hDC, D_FONT_BOLD);
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
            if id == 301 {
                let mut cfg = AppConfig::load();
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

                let mut buf = [0u16; 16];
                let len = GetWindowTextW(GetDlgItem(hwnd, 205), buf.as_mut_ptr(), buf.len() as i32);
                let text = String::from_utf16_lossy(&buf[..len.max(0) as usize]);
                if let Ok(val) = text.trim().parse::<u32>() {
                    cfg.triggers.threshold_percent = crate::config::clamp_threshold_percent(val);
                }
                cfg.triggers.auto_timer_enabled = is_checked(hwnd, 209);
                let mut buf_time = [0u16; 16];
                let len_time = GetWindowTextW(GetDlgItem(hwnd, 211), buf_time.as_mut_ptr(), buf_time.len() as i32);
                let text_time = String::from_utf16_lossy(&buf_time[..len_time.max(0) as usize]);
                if let Ok(val) = text_time.trim().parse::<u32>() {
                    cfg.triggers.timer_seconds = crate::config::clamp_timer_seconds(val);
                }

                cfg.triggers.islc_enabled = is_checked(hwnd, 220);
                let mut buf_s = [0u16; 16];
                let len_s = GetWindowTextW(GetDlgItem(hwnd, 221), buf_s.as_mut_ptr(), buf_s.len() as i32);
                let text_s = String::from_utf16_lossy(&buf_s[..len_s.max(0) as usize]);
                if let Ok(val) = text_s.trim().parse::<u64>() {
                    cfg.triggers.islc_standby_mb = crate::config::clamp_islc_mb(val);
                }

                let mut buf_f = [0u16; 16];
                let len_f = GetWindowTextW(GetDlgItem(hwnd, 222), buf_f.as_mut_ptr(), buf_f.len() as i32);
                let text_f = String::from_utf16_lossy(&buf_f[..len_f.max(0) as usize]);
                if let Ok(val) = text_f.trim().parse::<u64>() {
                    cfg.triggers.islc_free_mb = crate::config::clamp_islc_mb(val);
                }

                cfg.triggers.idle_clean_enabled = is_checked(hwnd, 230);
                let mut buf_idle = [0u16; 16];
                let len_idle = GetWindowTextW(GetDlgItem(hwnd, 231), buf_idle.as_mut_ptr(), buf_idle.len() as i32);
                let text_idle = String::from_utf16_lossy(&buf_idle[..len_idle.max(0) as usize]);
                if let Ok(val) = text_idle.trim().parse::<u32>() {
                    cfg.triggers.idle_seconds = crate::config::clamp_idle_seconds(val);
                }

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

                cfg.tuning.combine_pages_on_clean = is_checked(hwnd, 250);
                let _ = cfg.save();
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            if !D_BG_BRUSH.is_null() {
                DeleteObject(D_BG_BRUSH);
                D_BG_BRUSH = std::ptr::null_mut();
            }
            if !D_CARD_BRUSH.is_null() {
                DeleteObject(D_CARD_BRUSH);
                D_CARD_BRUSH = std::ptr::null_mut();
            }
            if !D_BORDER_PEN.is_null() {
                DeleteObject(D_BORDER_PEN);
                D_BORDER_PEN = std::ptr::null_mut();
            }
            if !D_FONT_UI.is_null() {
                DeleteObject(D_FONT_UI);
                D_FONT_UI = std::ptr::null_mut();
            }
            if !D_FONT_BOLD.is_null() {
                DeleteObject(D_FONT_BOLD);
                D_FONT_BOLD = std::ptr::null_mut();
            }
            if !D_FONT_SUBHEADER.is_null() {
                DeleteObject(D_FONT_SUBHEADER);
                D_FONT_SUBHEADER = std::ptr::null_mut();
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
