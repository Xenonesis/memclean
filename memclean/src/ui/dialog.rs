use crate::config::{AppConfig, TrayClickAction, TrayColorMode, TrayStyle};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    InvalidateRect, LineTo, MoveToEx, RoundRect, SelectObject, SetBkMode, SetTextColor, PAINTSTRUCT,
    DT_CENTER, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, PS_SOLID, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetDlgCtrlID,
    GetDlgItem, GetMessageW, GetWindowTextW, IsDialogMessageW, IsWindow, LoadIconW, RegisterClassW,
    SendMessageW, SetWindowTextW, ShowWindow, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX,
    BS_AUTORADIOBUTTON, BS_OWNERDRAW, ES_CENTER, ES_NUMBER, MSG, SW_HIDE, SW_SHOW, WM_COMMAND,
    WM_CTLCOLORBTN, WM_CTLCOLORDLG, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DESTROY, WM_DRAWITEM,
    WM_ERASEBKGND, WM_PAINT, WM_SETFONT, WM_SETICON, WNDCLASSW, WS_BORDER, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
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
static mut D_FONT_DESC: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();

static mut CURRENT_TAB: usize = 0;

const TAB_0_CTRLS: &[i32] = &[101, 102, 103, 104, 105, 201, 202, 204, 215, 203, 214];
const TAB_1_CTRLS: &[i32] = &[
    205, 206, 207, 208, 209, 210, 211, 212, 213,
    220, 221, 222, 223, 224, 225, 226, 230, 231, 232, 233,
];
const TAB_2_CTRLS: &[i32] = &[240, 241, 242, 243, 251, 250, 270, 275, 260, 261, 262];

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

unsafe fn switch_tab(hwnd: HWND, new_tab: usize) {
    CURRENT_TAB = new_tab;
    for &id in TAB_0_CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 0 { SW_SHOW } else { SW_HIDE });
        }
    }
    for &id in TAB_1_CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 1 { SW_SHOW } else { SW_HIDE });
        }
    }
    for &id in TAB_2_CTRLS {
        let ctrl = GetDlgItem(hwnd, id);
        if !ctrl.is_null() {
            ShowWindow(ctrl, if new_tab == 2 { SW_SHOW } else { SW_HIDE });
        }
    }
    for id in 601..=603 {
        let btn = GetDlgItem(hwnd, id);
        if !btn.is_null() {
            InvalidateRect(btn, std::ptr::null(), 1);
        }
    }
    InvalidateRect(hwnd, std::ptr::null(), 1);
}

pub fn show_settings_dialog(_parent: HWND) {
    unsafe {
        CURRENT_TAB = 0;
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
        D_FONT_DESC = CreateFontW(
            -12, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0,
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

        let width = 500;
        let height = 495;
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

        let create_tab_btn = |text: *const u16, cx: i32, cw: i32, id: i32| -> HWND {
            let ctrl = CreateWindowExW(
                0, windows_sys::w!("BUTTON"), text,
                WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                cx, 12, cw, 30, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut(),
            );
            if !ctrl.is_null() && !D_FONT_BOLD.is_null() {
                SendMessageW(ctrl, WM_SETFONT, D_FONT_BOLD as WPARAM, 1);
            }
            ctrl
        };

        create_tab_btn(windows_sys::w!("Tray & System"), 16, 145, 601);
        create_tab_btn(windows_sys::w!("Auto-Triggers"), 169, 145, 602);
        create_tab_btn(windows_sys::w!("Kernel & Gaming"), 322, 146, 603);

        let create_radio = |text: *const u16, cx: i32, cy: i32, cw: i32, id: i32| -> HWND {
            let ctrl = CreateWindowExW(
                0, windows_sys::w!("BUTTON"), text,
                WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON as u32,
                cx, cy, cw, 20, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut(),
            );
            if !ctrl.is_null() && !D_FONT_UI.is_null() {
                SendMessageW(ctrl, WM_SETFONT, D_FONT_UI as WPARAM, 1);
            }
            ctrl
        };

        let create_chk = |text: *const u16, cx: i32, cy: i32, cw: i32, id: i32| -> HWND {
            let ctrl = CreateWindowExW(
                0, windows_sys::w!("BUTTON"), text,
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
                cx, cy, cw, 20, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut(),
            );
            if !ctrl.is_null() && !D_FONT_UI.is_null() {
                SendMessageW(ctrl, WM_SETFONT, D_FONT_UI as WPARAM, 1);
            }
            ctrl
        };

        // --- TAB 0: TRAY & SYSTEM ---
        // Card 1: Tray Icon Style
        create_radio(windows_sys::w!("Floating Transparent Number (Minimalist)"), 30, 84, 420, 101);
        create_radio(windows_sys::w!("Rounded Capsule Badge (Dark Matte)"), 30, 108, 420, 102);
        create_radio(windows_sys::w!("Monogram Logo Icon (Concept B)"), 30, 132, 420, 103);

        // Card 1: Color Mode
        create_radio(windows_sys::w!("Dynamic Color (Green / Amber / Red)"), 30, 180, 420, 104);
        create_radio(windows_sys::w!("Monochrome White (Distraction-Free)"), 30, 202, 420, 105);

        // Card 2: Tray Click Action
        create_radio(windows_sys::w!("Click Opens / Restores Dashboard"), 30, 264, 420, 201);
        create_radio(windows_sys::w!("Click Optimizes Memory Instantly"), 30, 286, 420, 202);

        // Card 2: System Behavior & Startup
        create_chk(windows_sys::w!("Enable Global Hotkey (Ctrl + Alt + M)"), 30, 328, 420, 204);
        create_chk(windows_sys::w!("Enable Gaming HUD Micro-Overlay (Ctrl + Shift + O)"), 30, 348, 420, 215);
        create_chk(windows_sys::w!("Start Minimized to System Tray on Login"), 30, 368, 420, 203);
        create_chk(windows_sys::w!("Automatically check for updates on startup"), 30, 388, 420, 214);

        // --- TAB 1: AUTO-TRIGGERS ---
        // Card 1: RAM Threshold Controls
        let lbl_thresh = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Auto-optimize when RAM exceeds:"),
            WS_CHILD | WS_VISIBLE,
            30, 88, 220, 20, hwnd, 206 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_thresh.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_thresh, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_thresh = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("80"),
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            254, 86, 44, 22, hwnd, 205 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_thresh.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_thresh, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_pct = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("%"),
            WS_CHILD | WS_VISIBLE,
            304, 88, 20, 20, hwnd, 207 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_pct.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_pct, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_note = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("(Range: 10% - 95%, Default: 80%)"),
            WS_CHILD | WS_VISIBLE,
            30, 112, 300, 16, hwnd, 208 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_note.is_null() && !D_FONT_SUBHEADER.is_null() {
            SendMessageW(lbl_note, WM_SETFONT, D_FONT_SUBHEADER as WPARAM, 1);
        }

        // Card 1: Periodic Auto-Clean Timer Controls
        create_chk(windows_sys::w!("Enable periodic auto-clean timer"), 30, 164, 420, 209);

        let lbl_timer = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Interval:"),
            WS_CHILD | WS_VISIBLE,
            50, 190, 60, 20, hwnd, 210 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_timer.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_timer, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_timer = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("15"),
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            114, 188, 50, 22, hwnd, 211 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_timer.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_timer, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_secs = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("seconds"),
            WS_CHILD | WS_VISIBLE,
            170, 190, 60, 20, hwnd, 212 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_secs.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_secs, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_note2 = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("(0 = Continuous, Range: 1 - 300s)"),
            WS_CHILD | WS_VISIBLE,
            236, 190, 218, 18, hwnd, 213 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_note2.is_null() && !D_FONT_SUBHEADER.is_null() {
            SendMessageW(lbl_note2, WM_SETFONT, D_FONT_SUBHEADER as WPARAM, 1);
        }

        // Card 2: ISLC Standby Trigger Controls
        create_chk(windows_sys::w!("Enable ISLC Standby Trigger (Gaming)"), 30, 268, 420, 220);

        let lbl_islc_s = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Standby >="),
            WS_CHILD | WS_VISIBLE,
            50, 294, 75, 20, hwnd, 223 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_islc_s.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_islc_s, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_islc_standby = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("1024"),
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            130, 292, 54, 22, hwnd, 221 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_islc_standby.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_islc_standby, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_islc_s_mb = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("MB"),
            WS_CHILD | WS_VISIBLE,
            190, 294, 28, 20, hwnd, 224 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_islc_s_mb.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_islc_s_mb, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_islc_f = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Free <="),
            WS_CHILD | WS_VISIBLE,
            232, 294, 58, 20, hwnd, 225 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_islc_f.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_islc_f, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_islc_free = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("1024"),
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            294, 292, 54, 22, hwnd, 222 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_islc_free.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_islc_free, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_islc_f_mb = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("MB"),
            WS_CHILD | WS_VISIBLE,
            354, 294, 28, 20, hwnd, 226 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_islc_f_mb.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_islc_f_mb, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        // Card 2: User Inactivity / AFK Idle Clean
        create_chk(windows_sys::w!("Auto-clean on user inactivity / AFK"), 30, 352, 420, 230);

        let lbl_idle = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("Idle timeout:"),
            WS_CHILD | WS_VISIBLE,
            50, 376, 85, 20, hwnd, 232 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_idle.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_idle, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let edit_idle = CreateWindowExW(
            0, windows_sys::w!("EDIT"), windows_sys::w!("300"),
            WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
            140, 374, 50, 22, hwnd, 231 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !edit_idle.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(edit_idle, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_idle_sec = CreateWindowExW(
            0, windows_sys::w!("STATIC"), windows_sys::w!("seconds (Default: 300s)"),
            WS_CHILD | WS_VISIBLE,
            196, 376, 200, 20, hwnd, 233 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_idle_sec.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(lbl_idle_sec, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        // --- TAB 2: KERNEL & GAMING ---
        // Card 1: Scheduler Timer Resolution
        create_radio(windows_sys::w!("Default Timer Resolution (15.6 ms - Standard)"), 30, 84, 420, 240);
        create_radio(windows_sys::w!("1.0 ms Multimedia Latency (1,000 µs - Audio / Video Sync)"), 30, 106, 420, 241);
        create_radio(windows_sys::w!("0.5 ms Competitive Gaming (500 µs - Minimum Latency)"), 30, 128, 420, 242);
        create_chk(windows_sys::w!("Unpark 100% CPU Cores (Zero-Latency Burst Performance)"), 30, 152, 420, 243);

        let lbl_res_desc = CreateWindowExW(
            0, windows_sys::w!("STATIC"),
            windows_sys::w!("Locks Windows kernel timer resolution to 0.5ms and unparks all cores for zero-stutter gaming."),
            WS_CHILD | WS_VISIBLE,
            30, 178, 420, 36, hwnd, 251 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_res_desc.is_null() && !D_FONT_DESC.is_null() {
            SendMessageW(lbl_res_desc, WM_SETFONT, D_FONT_DESC as WPARAM, 1);
        }

        // Card 2: Kernel Page Combining, VRAM, CPU & Network Latency
        create_chk(windows_sys::w!("Enable Kernel Page Combining (RAM De-duplication)"), 30, 264, 420, 250);
        create_chk(windows_sys::w!("Enable GPU / VRAM Decommitment on Clean (WDDM 2.1+)"), 30, 285, 420, 270);
        create_chk(windows_sys::w!("Enable Hybrid CPU Scheduling (Games to P-Cores, Bloat to E-Cores)"), 30, 306, 420, 275);

        create_chk(windows_sys::w!("Enable Competitive Gaming TCP Stack (NoDelay && No Throttling)"), 30, 350, 420, 260);

        let btn_net_restore = CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Restore Network Defaults"),
            WS_CHILD | WS_VISIBLE,
            30, 374, 165, 22, hwnd, 261 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !btn_net_restore.is_null() && !D_FONT_UI.is_null() {
            SendMessageW(btn_net_restore, WM_SETFONT, D_FONT_UI as WPARAM, 1);
        }

        let lbl_net_desc = CreateWindowExW(
            0, windows_sys::w!("STATIC"),
            windows_sys::w!("No delayed ACKs, 100% CPU priority."),
            WS_CHILD | WS_VISIBLE,
            205, 377, 245, 18, hwnd, 262 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        if !lbl_net_desc.is_null() && !D_FONT_DESC.is_null() {
            SendMessageW(lbl_net_desc, WM_SETFONT, D_FONT_DESC as WPARAM, 1);
        }

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
        set_checked(hwnd, 215, cfg.general.gaming_hud_enabled);
        set_checked(hwnd, 260, cfg.network.low_latency_enabled);
        set_checked(hwnd, 270, cfg.tuning.vram_trim_on_clean);
        set_checked(hwnd, 243, cfg.tuning.unpark_cores_enabled);
        set_checked(hwnd, 275, cfg.tuning.hybrid_cpu_optimizer_enabled);

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

        // Switch to Tab 0 as initial view
        switch_tab(hwnd, 0);

        // Flat Minimalist Apply & Close button (BS_OWNERDRAW)
        let btn = CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Apply and Close"),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            16, 420, 452, 36, hwnd, 301 as _, std::ptr::null_mut(), std::ptr::null_mut(),
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

            SetBkMode(hdc, TRANSPARENT as i32);
            let old_font = SelectObject(hdc, D_FONT_SUBHEADER);
            let old_brush = SelectObject(hdc, D_CARD_BRUSH);
            let old_pen = SelectObject(hdc, D_BORDER_PEN);

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

                    // Card 2: System & Startup (16, 238, 468, 404)
                    RoundRect(hdc, 16, 238, 468, 404, 8, 8);
                    let mut h2_rc = RECT { left: 30, top: 246, right: 454, bottom: 260 };
                    DrawTextW(hdc, windows_sys::w!("TRAY CLICK ACTION"), -1, &mut h2_rc, DT_SINGLELINE);

                    MoveToEx(hdc, 30, 310, std::ptr::null_mut());
                    LineTo(hdc, 454, 310);

                    let mut hb_rc = RECT { left: 30, top: 316, right: 454, bottom: 330 };
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
                let old_font = SelectObject(dis.hDC, if is_active { D_FONT_BOLD } else { D_FONT_UI });

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
            if id >= 601 && id <= 603 {
                switch_tab(hwnd, (id - 601) as usize);
                return 0;
            }

            if id == 261 {
                match crate::core::network::restore_network_defaults() {
                    Ok(_) => {
                        set_checked(hwnd, 260, false);
                        let mut cfg = AppConfig::load();
                        cfg.network.low_latency_enabled = false;
                        let _ = cfg.save();
                        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                            hwnd,
                            windows_sys::w!("Network defaults successfully restored (throttling reset to 10, responsiveness to 20)."),
                            windows_sys::w!("MemClean Network"),
                            0x00000040, // MB_OK | MB_ICONINFORMATION
                        );
                    }
                    Err(e) => {
                        let err_wide: Vec<u16> = format!("Failed to restore network defaults:\n{}\n\n(Administrator privileges may be required)", e)
                            .encode_utf16()
                            .chain(std::iter::once(0))
                            .collect();
                        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                            hwnd,
                            err_wide.as_ptr(),
                            windows_sys::w!("MemClean Network Error"),
                            0x00000030, // MB_OK | MB_ICONWARNING
                        );
                    }
                }
                return 0;
            }
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
                let hud_enabled = is_checked(hwnd, 215);
                cfg.general.gaming_hud_enabled = hud_enabled;
                if hud_enabled {
                    crate::ui::hud::show_hud();
                } else {
                    crate::ui::hud::hide_hud();
                }

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
                cfg.tuning.vram_trim_on_clean = is_checked(hwnd, 270);
                cfg.tuning.hybrid_cpu_optimizer_enabled = is_checked(hwnd, 275);
                let unpark_checked = is_checked(hwnd, 243);
                if unpark_checked != cfg.tuning.unpark_cores_enabled {
                    cfg.tuning.unpark_cores_enabled = unpark_checked;
                    if unpark_checked {
                        let _ = crate::core::cpu::unpark_all_cores();
                    } else {
                        let _ = crate::core::cpu::restore_core_parking_default();
                    }
                }
                let net_opt_checked = is_checked(hwnd, 260);
                if net_opt_checked != cfg.network.low_latency_enabled {
                    cfg.network.low_latency_enabled = net_opt_checked;
                    if net_opt_checked {
                        let _ = crate::core::network::optimize_network();
                    } else {
                        let _ = crate::core::network::restore_network_defaults();
                    }
                }
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
            if !D_FONT_DESC.is_null() {
                DeleteObject(D_FONT_DESC);
                D_FONT_DESC = std::ptr::null_mut();
            }
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
        for &id in TAB_0_CTRLS {
            assert!(seen.insert(id), "Duplicate ID {} in TAB_0_CTRLS", id);
        }
        for &id in TAB_1_CTRLS {
            assert!(seen.insert(id), "Duplicate ID {} in TAB_1_CTRLS", id);
        }
        for &id in TAB_2_CTRLS {
            assert!(seen.insert(id), "Duplicate ID {} in TAB_2_CTRLS", id);
        }

        // Tab buttons and action button must not collide with content IDs
        for id in [301, 601, 602, 603] {
            assert!(!seen.contains(&id), "Button ID {} collides with control ID", id);
        }
    }

    #[test]
    fn test_all_expected_controls_mapped() {
        let expected = [
            101, 102, 103, 104, 105, 201, 202, 204, 215, 203, 214, // Tab 0
            205, 206, 207, 208, 209, 210, 211, 212, 213, // Tab 1 threshold & timer
            220, 221, 222, 223, 224, 225, 226, 230, 231, 232, 233, // Tab 1 islc & idle
            240, 241, 242, 243, 250, 270, 275, 251, 260, 261, 262, // Tab 2 kernel, de-dup, vram, cpu & network
        ];
        for &id in &expected {
            let in_0 = TAB_0_CTRLS.contains(&id);
            let in_1 = TAB_1_CTRLS.contains(&id);
            let in_2 = TAB_2_CTRLS.contains(&id);
            assert!(in_0 || in_1 || in_2, "Expected control ID {} not mapped in any tab", id);
        }
    }
}
