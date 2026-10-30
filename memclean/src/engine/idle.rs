use windows_sys::Win32::System::SystemInformation::GetTickCount64;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

/// Returns the elapsed time in seconds since the last keyboard or mouse input event.
pub fn get_user_idle_seconds() -> u64 {
    unsafe {
        let mut lii: LASTINPUTINFO = std::mem::zeroed();
        lii.cbSize = std::mem::size_of::<LASTINPUTINFO>() as u32;

        if GetLastInputInfo(&mut lii) != 0 {
            let now = GetTickCount64();
            let elapsed_ms = now.saturating_sub(lii.dwTime as u64);
            elapsed_ms / 1000
        } else {
            0
        }
    }
}

/// Determines if an idle clean should trigger.
/// Returns true if user idle time >= `idle_seconds_threshold` and an idle clean hasn't already run in this session.
pub fn should_idle_clean(idle_seconds_threshold: u32, already_cleaned: bool) -> bool {
    if idle_seconds_threshold == 0 || already_cleaned {
        return false;
    }

    get_user_idle_seconds() >= idle_seconds_threshold as u64
}
