use crate::config::AppConfig;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    IsDialogMessageW, LoadIconW, MessageBoxW, RegisterClassW, SendMessageW, ShowWindow,
    BS_AUTOCHECKBOX, BS_PUSHBUTTON, MB_ICONINFORMATION, MB_OK, MSG, SW_SHOW,
    WM_COMMAND, WM_DESTROY, WM_SETICON, WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

pub fn show_settings_dialog(_parent: HWND) {
    unsafe {
        let class_name = windows_sys::w!("MemCleanSettingsClass");
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
        let app_icon = LoadIconW(hinstance, 1 as _);

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(dialog_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: app_icon,
            hCursor: std::ptr::null_mut(),
            hbrBackground: 6 as _, // COLOR_WINDOW
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name,
            windows_sys::w!("MemClean Settings"),
            WS_OVERLAPPEDWINDOW & !(0x00010000 | 0x00040000), // Disable maximize & resize
            200, 200, 360, 280,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if !app_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, 1 as _, app_icon as _);
            SendMessageW(hwnd, WM_SETICON, 0 as _, app_icon as _);
        }

        // Checkboxes for features
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Purge Standby List"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 20, 280, 24, hwnd, 201 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Flush Modified List"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 50, 280, 24, hwnd, 202 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Trim System File Cache"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 80, 280, 24, hwnd, 203 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Trim Background Working Sets"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 110, 280, 24, hwnd, 204 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Enable Game & Foreground Shield"),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
            20, 140, 280, 24, hwnd, 205 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );

        // Save & Close button
        CreateWindowExW(
            0, windows_sys::w!("BUTTON"), windows_sys::w!("Save and Close"),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
            100, 190, 140, 32, hwnd, 301 as _, std::ptr::null_mut(), std::ptr::null_mut(),
        );

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
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
