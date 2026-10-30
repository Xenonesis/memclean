use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId};

pub struct ForegroundContext {
    pub pid: u32,
    pub is_fullscreen: bool,
}

pub fn get_foreground_context() -> Option<ForegroundContext> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }

        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }

        let own_pid = windows_sys::Win32::System::Threading::GetCurrentProcessId();
        if pid == own_pid {
            return None;
        }

        let mut class_buf = [0u16; 64];
        let len = windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW(
            hwnd,
            class_buf.as_mut_ptr(),
            class_buf.len() as i32,
        );
        if len > 0 {
            let class_str = String::from_utf16_lossy(&class_buf[..len as usize]);
            if class_str == "Progman" || class_str == "WorkerW" || class_str == "Shell_TrayWnd" {
                return None;
            }
        }

        let mut win_rect: RECT = std::mem::zeroed();
        GetWindowRect(hwnd, &mut win_rect);

        let h_monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let mut is_fullscreen = false;
        if MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) != std::ptr::null_mut()
            && GetMonitorInfoW(h_monitor, &mut mi) != 0
        {
            is_fullscreen = win_rect.left <= mi.rcMonitor.left
                && win_rect.top <= mi.rcMonitor.top
                && win_rect.right >= mi.rcMonitor.right
                && win_rect.bottom >= mi.rcMonitor.bottom;
        }

        Some(ForegroundContext {
            pid,
            is_fullscreen,
        })
    }
}

pub fn is_protected_process(
    name: &str,
    pid: u32,
    fg_pid: Option<u32>,
    custom_exclusions: &[String],
) -> bool {
    let lower_name = name.to_lowercase();

    // 0. Always protect self regardless of executable name
    let own_pid = unsafe { windows_sys::Win32::System::Threading::GetCurrentProcessId() };
    if pid == own_pid {
        return true;
    }

    // 1. Whitelist of Windows core system critical processes
    const SYSTEM_WHITELIST: &[&str] = &[
        "system",
        "registry",
        "smss.exe",
        "csrss.exe",
        "wininit.exe",
        "services.exe",
        "lsass.exe",
        "dwm.exe",
        "fontdrvhost.exe",
        "memclean.exe",
    ];

    if SYSTEM_WHITELIST.contains(&lower_name.as_str()) {
        return true;
    }
    // 2. Foreground process protection (shield)
    if let Some(foreground_id) = fg_pid {
        if pid == foreground_id {
            return true;
        }
    }

    // 3. User configured exclusion list
    if custom_exclusions.iter().any(|ex| ex.trim().eq_ignore_ascii_case(&lower_name)) {
        return true;
    }
    false
}
