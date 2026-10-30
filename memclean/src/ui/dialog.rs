use crate::config::AppConfig;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, RoundRect, SelectObject,
    SetBkMode, SetTextColor, PS_SOLID, TRANSPARENT, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    IsDialogMessageW, LoadIconW, MessageBoxW, RegisterClassW, SendMessageW, ShowWindow,
    BS_AUTOCHECKBOX, BS_OWNERDRAW, MB_ICONINFORMATION, MB_OK, MSG, SW_SHOW,
    WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG, WM_CTLCOLORSTATIC, WM_DESTROY,
    WM_DRAWITEM, WM_SETFONT, WM_SETICON, WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
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
static mut D_FONT_UI: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();
static mut D_FONT_BOLD: windows_sys::Win32::Graphics::Gdi::HFONT = std::ptr::null_mut();

pub fn show_settings_dialog(_parent: HWND) {
    unsafe {
        let class_name = windows_sys::w!("MemCleanSettingsClass");
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
        let app_icon = LoadIconW(hinstance, 1 as _);

        D_BG_BRUSH = CreateSolidBrush(0x00FAFBFB); // Warm Bone #FBFBFA
        D_FONT_UI = CreateFontW(
            -14, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        D_FONT_BOLD = CreateFontW(
            -14, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
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

        let hwnd = CreateWindowExW(
            0,
            class_name,
            windows_sys::w!("MemClean Settings"),
            WS_OVERLAPPEDWINDOW & !(0x00010000 | 0x00040000), // Disable maximize & resize
            200, 200, 380, 310,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if !app_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 1 as _, app_icon as _);
            SendMessageW(hwnd, WM_SETICON, 0 as _, app_icon as _);
        }

        let create_chk = |text: *const u16, cy: i32, id: i32| -> HWND {
            let ctrl = CreateWindowExW(
                0, windows_sys::w!("BUTTON"), text,
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
                24, cy, 320, 24, hwnd, id as _, std::ptr::null_mut(), std::ptr::null_mut(),
            );
            if !ctrl.is_null() && !D_FONT_UI.is_null() {
                SendMessageW(ctrl, WM_SETFONT, D_FONT_UI as WPARAM, 1);
            }
            ctrl
        };

        // Checkboxes for features
        create_chk(windows_sys::w!("Purge Standby Page List"), 24, 201);
        create_chk(windows_sys::w!("Low-Priority Standby Purge"), 54, 202);
        create_chk(windows_sys::w!("Flush Modified Page List"), 84, 203);
        create_chk(windows_sys::w!("Trim System File Cache"), 114, 204);
        create_chk(windows_sys::w!("Trim Background Working Sets"), 144, 205);
        create_chk(windows_sys::w!("Enable Game & Foreground Shield"), 174, 206);

        // Flat Minimalist Save & Close button (BS_OWNERDRAW)
        let btn = CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Save and Close"),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
            24, 218, 320, 36, hwnd, 301 as _, std::ptr::null_mut(), std::ptr::null_mut(),
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
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00111111);
            D_BG_BRUSH as LRESULT
        }
        WM_CTLCOLORBTN => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00111111);
            D_BG_BRUSH as LRESULT
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
                let text = windows_sys::w!("Save and Close");
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
                let config = AppConfig::load();
                let _ = config.save();
                MessageBoxW(
                    hwnd,
                    windows_sys::w!("Settings saved successfully."),
                    windows_sys::w!("MemClean"),
                    MB_OK | MB_ICONINFORMATION,
                );
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            if !D_BG_BRUSH.is_null() {
                DeleteObject(D_BG_BRUSH);
                D_BG_BRUSH = std::ptr::null_mut();
            }
            if !D_FONT_UI.is_null() {
                DeleteObject(D_FONT_UI);
                D_FONT_UI = std::ptr::null_mut();
            }
            if !D_FONT_BOLD.is_null() {
                DeleteObject(D_FONT_BOLD);
                D_FONT_BOLD = std::ptr::null_mut();
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
