use crate::config::{AppConfig, TrayClickAction, TrayColorMode, TrayStyle};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, PAINTSTRUCT,
    CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, LineTo, MoveToEx,
    RoundRect, SelectObject, SetBkMode, SetTextColor, PS_SOLID, TRANSPARENT, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetDlgItem, GetMessageW,
    IsDialogMessageW, LoadIconW, RegisterClassW, SendMessageW, ShowWindow,
    BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_OWNERDRAW, MSG, SW_SHOW,
    WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG, WM_CTLCOLORSTATIC, WM_DESTROY,
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
        let app_icon = LoadIconW(hinstance, 1 as _);

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
        let height = 460;
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
        if !app_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 1 as _, app_icon as _);
            SendMessageW(hwnd, WM_SETICON, 0 as _, app_icon as _);
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
        create_radio(windows_sys::w!("Click Opens / Restores Dashboard"), 228, 201);
        create_radio(windows_sys::w!("Click Optimizes Memory Instantly"), 252, 202);
        create_chk(windows_sys::w!("Start Minimized to System Tray on Login"), 294, 203);

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

        // Flat Minimalist Apply & Close button (BS_OWNERDRAW)
        let btn = CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Apply and Close"),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            12, 385, 340, 36, hwnd, 301 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !btn.is_null() && !D_FONT_BOLD.is_null() {
            SendMessageW(btn, WM_SETFONT, D_FONT_BOLD as WPARAM, 1);
        }

        ShowWindow(hwnd, SW_SHOW);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
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

            // Card 2: Tray Behavior & Startup (12, 202, 352, 372)
            SelectObject(hdc, D_CARD_BRUSH);
            SelectObject(hdc, D_BORDER_PEN);
            RoundRect(hdc, 12, 202, 352, 372, 10, 10);

            // Card 2 Header: TRAY INTERACTION & STARTUP
            SetTextColor(hdc, 0x00747778);
            let header2 = windows_sys::w!("TRAY INTERACTION && STARTUP");
            let mut h2_rc = RECT { left: 24, top: 210, right: 340, bottom: 224 };
            DrawTextW(hdc, header2, -1, &mut h2_rc, DT_SINGLELINE);

            // Sub-divider
            MoveToEx(hdc, 24, 280, std::ptr::null_mut());
            LineTo(hdc, 340, 280);

            SelectObject(hdc, old_brush);
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_font);
            EndPaint(hwnd, &ps);
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00111111);
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
