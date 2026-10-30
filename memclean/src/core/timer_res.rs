use crate::core::ntdll;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA};

type TimeBeginPeriodFn = unsafe extern "system" fn(u32) -> u32;
type TimeEndPeriodFn = unsafe extern "system" fn(u32) -> u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimerResolutionMode {
    Default,
    OneMs,
    HalfMs,
}

impl Default for TimerResolutionMode {
    fn default() -> Self {
        TimerResolutionMode::Default
    }
}

pub struct TimerResolutionGuard {
    active_mode: TimerResolutionMode,
    active_100ns: u32,
    used_winmm_fallback: bool,
}

impl TimerResolutionGuard {
    pub fn new() -> Self {
        Self {
            active_mode: TimerResolutionMode::Default,
            active_100ns: 0,
            used_winmm_fallback: false,
        }
    }

    pub fn set_mode(&mut self, mode: TimerResolutionMode) -> Result<(), String> {
        // First revert existing if active
        self.revert();

        match mode {
            TimerResolutionMode::Default => {
                self.active_mode = TimerResolutionMode::Default;
                Ok(())
            }
            TimerResolutionMode::OneMs => self.apply_resolution(10_000, mode), // 1.0 ms = 10,000 * 100ns
            TimerResolutionMode::HalfMs => self.apply_resolution(5_000, mode),  // 0.5 ms = 5,000 * 100ns
        }
    }

    fn apply_resolution(&mut self, desired_100ns: u32, mode: TimerResolutionMode) -> Result<(), String> {
        if ntdll::set_timer_resolution(desired_100ns, true).is_ok() {
            self.active_mode = mode;
            self.active_100ns = desired_100ns;
            self.used_winmm_fallback = false;
            return Ok(());
        }

        // Fallback to winmm timeBeginPeriod(1)
        unsafe {
            let h_winmm = LoadLibraryA(b"winmm.dll\0".as_ptr());
            if !h_winmm.is_null() {
                if let Some(proc) = GetProcAddress(h_winmm, b"timeBeginPeriod\0".as_ptr()) {
                    let time_begin: TimeBeginPeriodFn = std::mem::transmute(proc);
                    if time_begin(1) == 0 {
                        self.active_mode = mode;
                        self.active_100ns = desired_100ns;
                        self.used_winmm_fallback = true;
                        return Ok(());
                    }
                }
            }
        }

        Err("Failed to set timer resolution via both NTDLL and winmm".to_string())
    }

    pub fn revert(&mut self) {
        if self.active_100ns > 0 {
            let _ = ntdll::set_timer_resolution(self.active_100ns, false);
            self.active_100ns = 0;
        }
        if self.used_winmm_fallback {
            unsafe {
                let h_winmm = GetModuleHandleA(b"winmm.dll\0".as_ptr());
                if !h_winmm.is_null() {
                    if let Some(proc) = GetProcAddress(h_winmm, b"timeEndPeriod\0".as_ptr()) {
                        let time_end: TimeEndPeriodFn = std::mem::transmute(proc);
                        let _ = time_end(1);
                    }
                }
            }
            self.used_winmm_fallback = false;
        }
        self.active_mode = TimerResolutionMode::Default;
    }

    pub fn get_current_resolution_ms() -> f32 {
        if let Ok((_min, _max, cur)) = ntdll::query_timer_resolution() {
            (cur as f32) / 10_000.0
        } else {
            15.625
        }
    }

    pub fn current_mode(&self) -> TimerResolutionMode {
        self.active_mode
    }
}

impl Default for TimerResolutionGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TimerResolutionGuard {
    fn drop(&mut self) {
        self.revert();
    }
}
