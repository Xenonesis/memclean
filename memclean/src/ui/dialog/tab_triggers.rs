use crate::config::{
    clamp_idle_seconds, clamp_islc_mb, clamp_threshold_percent, clamp_timer_seconds, AppConfig,
};
use windows_sys::Win32::Foundation::{HWND, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetDlgItem, GetWindowTextW, SendMessageW, SetWindowTextW, ES_CENTER,
    ES_NUMBER, WM_SETFONT, WS_BORDER, WS_CHILD, WS_VISIBLE,
};

use super::common::{create_chk, font_desc, font_ui, is_checked, set_checked};

pub const CTRLS: &[i32] = &[
    205, 206, 207, 208, 209, 210, 211, 212, 213, 220, 221, 222, 223, 224, 225, 226, 230, 231,
    232, 233,
];

/// Creates all controls for Tab 1 (Auto-Triggers & Thresholds).
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog window handle on the UI thread.
pub unsafe fn create(hwnd: HWND) {
    // --- TAB 1: AUTO-TRIGGERS ---
    // Left Column: RAM Threshold Controls
    let lbl_thresh = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Auto-optimize when RAM exceeds:"),
        WS_CHILD | WS_VISIBLE,
        34,
        224,
        230,
        22,
        hwnd,
        206 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_thresh.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_thresh, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let edit_thresh = CreateWindowExW(
        0,
        windows_sys::w!("EDIT"),
        windows_sys::w!("80"),
        WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
        270,
        220,
        50,
        26,
        hwnd,
        205 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !edit_thresh.is_null() && !font_ui().is_null() {
        SendMessageW(edit_thresh, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let lbl_pct = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("%"),
        WS_CHILD | WS_VISIBLE,
        326,
        224,
        20,
        22,
        hwnd,
        207 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_pct.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_pct, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let lbl_note = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("(Range: 50% - 95%, Default: 80%)"),
        WS_CHILD | WS_VISIBLE,
        34,
        254,
        354,
        18,
        hwnd,
        208 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    let f_desc = font_desc();
    if !lbl_note.is_null() && !f_desc.is_null() {
        SendMessageW(lbl_note, WM_SETFONT, f_desc as WPARAM, 1);
    }

    // Left Column: Periodic Auto-Clean Timer Controls
    create_chk(
        hwnd,
        windows_sys::w!("Enable Periodic Auto-Clean Timer"),
        34,
        330,
        354,
        209,
    );

    let lbl_timer = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Clean Interval:"),
        WS_CHILD | WS_VISIBLE,
        34,
        384,
        100,
        22,
        hwnd,
        210 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_timer.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_timer, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let edit_timer = CreateWindowExW(
        0,
        windows_sys::w!("EDIT"),
        windows_sys::w!("300"),
        WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
        138,
        380,
        60,
        26,
        hwnd,
        211 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !edit_timer.is_null() && !font_ui().is_null() {
        SendMessageW(edit_timer, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let lbl_secs = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("seconds"),
        WS_CHILD | WS_VISIBLE,
        206,
        384,
        60,
        22,
        hwnd,
        212 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_secs.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_secs, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let lbl_note2 = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("(Range: 60s - 3600s, Default: 300s / 5m)"),
        WS_CHILD | WS_VISIBLE,
        34,
        414,
        354,
        18,
        hwnd,
        213 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_note2.is_null() && !f_desc.is_null() {
        SendMessageW(lbl_note2, WM_SETFONT, f_desc as WPARAM, 1);
    }

    // --- RIGHT COLUMN: ISLC Standby Trigger Controls ---
    create_chk(
        hwnd,
        windows_sys::w!("Enable ISLC Standby Trigger (Gaming)"),
        432,
        220,
        354,
        220,
    );

    let lbl_islc_s = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Standby >="),
        WS_CHILD | WS_VISIBLE,
        432,
        274,
        76,
        22,
        hwnd,
        223 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_islc_s.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_islc_s, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let edit_islc_standby = CreateWindowExW(
        0,
        windows_sys::w!("EDIT"),
        windows_sys::w!("1024"),
        WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
        512,
        270,
        60,
        26,
        hwnd,
        221 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !edit_islc_standby.is_null() && !font_ui().is_null() {
        SendMessageW(edit_islc_standby, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let lbl_islc_s_mb = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("MB"),
        WS_CHILD | WS_VISIBLE,
        578,
        274,
        26,
        22,
        hwnd,
        224 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_islc_s_mb.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_islc_s_mb, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let lbl_islc_f = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Free <="),
        WS_CHILD | WS_VISIBLE,
        620,
        274,
        62,
        22,
        hwnd,
        225 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_islc_f.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_islc_f, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let edit_islc_free = CreateWindowExW(
        0,
        windows_sys::w!("EDIT"),
        windows_sys::w!("1024"),
        WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
        686,
        270,
        60,
        26,
        hwnd,
        222 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !edit_islc_free.is_null() && !font_ui().is_null() {
        SendMessageW(edit_islc_free, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let lbl_islc_f_mb = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("MB"),
        WS_CHILD | WS_VISIBLE,
        752,
        274,
        26,
        22,
        hwnd,
        226 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_islc_f_mb.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_islc_f_mb, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    // Right Column: User Inactivity / AFK Idle Clean
    create_chk(
        hwnd,
        windows_sys::w!("Auto-Clean on User Inactivity / AFK"),
        432,
        360,
        354,
        230,
    );

    let lbl_idle = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Idle Timeout:"),
        WS_CHILD | WS_VISIBLE,
        432,
        414,
        86,
        22,
        hwnd,
        232 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_idle.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_idle, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let edit_idle = CreateWindowExW(
        0,
        windows_sys::w!("EDIT"),
        windows_sys::w!("300"),
        WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER as u32 | ES_CENTER as u32,
        522,
        410,
        60,
        26,
        hwnd,
        231 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !edit_idle.is_null() && !font_ui().is_null() {
        SendMessageW(edit_idle, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    let lbl_idle_sec = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("seconds (Default: 300s)"),
        WS_CHILD | WS_VISIBLE,
        588,
        414,
        198,
        22,
        hwnd,
        233 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_idle_sec.is_null() && !f_desc.is_null() {
        SendMessageW(lbl_idle_sec, WM_SETFONT, f_desc as WPARAM, 1);
    }
}

/// Loads configuration state into Tab 1 controls.
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog handle containing Tab 1 controls.
pub unsafe fn load(hwnd: HWND, cfg: &AppConfig) {
    let s_thresh = format!("{}", cfg.triggers.threshold_percent);
    let wide_thresh: Vec<u16> = s_thresh.encode_utf16().chain(std::iter::once(0)).collect();
    SetWindowTextW(GetDlgItem(hwnd, 205), wide_thresh.as_ptr());

    set_checked(hwnd, 209, cfg.triggers.auto_timer_enabled);

    let s_timer = format!("{}", cfg.triggers.timer_seconds);
    let wide_timer: Vec<u16> = s_timer.encode_utf16().chain(std::iter::once(0)).collect();
    SetWindowTextW(GetDlgItem(hwnd, 211), wide_timer.as_ptr());

    set_checked(hwnd, 220, cfg.triggers.islc_enabled);

    let s_standby = format!("{}", cfg.triggers.islc_standby_mb);
    let wide_standby: Vec<u16> = s_standby.encode_utf16().chain(std::iter::once(0)).collect();
    SetWindowTextW(GetDlgItem(hwnd, 221), wide_standby.as_ptr());

    let s_free = format!("{}", cfg.triggers.islc_free_mb);
    let wide_free: Vec<u16> = s_free.encode_utf16().chain(std::iter::once(0)).collect();
    SetWindowTextW(GetDlgItem(hwnd, 222), wide_free.as_ptr());

    set_checked(hwnd, 230, cfg.triggers.idle_clean_enabled);

    let s_idle = format!("{}", cfg.triggers.idle_seconds);
    let wide_idle: Vec<u16> = s_idle.encode_utf16().chain(std::iter::once(0)).collect();
    SetWindowTextW(GetDlgItem(hwnd, 231), wide_idle.as_ptr());
}

/// Reads Tab 1 control states and updates `AppConfig`.
///
/// # Safety
/// Caller must ensure `hwnd` is a valid dialog handle containing Tab 1 controls.
pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig) {
    let mut buf_thresh = [0u16; 16];
    let len_thresh = GetWindowTextW(
        GetDlgItem(hwnd, 205),
        buf_thresh.as_mut_ptr(),
        buf_thresh.len() as i32,
    );
    let text_thresh = String::from_utf16_lossy(&buf_thresh[..len_thresh.max(0) as usize]);
    if let Ok(val) = text_thresh.trim().parse::<u32>() {
        cfg.triggers.threshold_percent = clamp_threshold_percent(val);
    }

    cfg.triggers.auto_timer_enabled = is_checked(hwnd, 209);
    let mut buf_timer = [0u16; 16];
    let len_timer = GetWindowTextW(
        GetDlgItem(hwnd, 211),
        buf_timer.as_mut_ptr(),
        buf_timer.len() as i32,
    );
    let text_timer = String::from_utf16_lossy(&buf_timer[..len_timer.max(0) as usize]);
    if let Ok(val) = text_timer.trim().parse::<u32>() {
        cfg.triggers.timer_seconds = clamp_timer_seconds(val);
    }

    cfg.triggers.islc_enabled = is_checked(hwnd, 220);
    let mut buf_s = [0u16; 16];
    let len_s = GetWindowTextW(GetDlgItem(hwnd, 221), buf_s.as_mut_ptr(), buf_s.len() as i32);
    let text_s = String::from_utf16_lossy(&buf_s[..len_s.max(0) as usize]);
    if let Ok(val) = text_s.trim().parse::<u64>() {
        cfg.triggers.islc_standby_mb = clamp_islc_mb(val);
    }

    let mut buf_f = [0u16; 16];
    let len_f = GetWindowTextW(GetDlgItem(hwnd, 222), buf_f.as_mut_ptr(), buf_f.len() as i32);
    let text_f = String::from_utf16_lossy(&buf_f[..len_f.max(0) as usize]);
    if let Ok(val) = text_f.trim().parse::<u64>() {
        cfg.triggers.islc_free_mb = clamp_islc_mb(val);
    }

    cfg.triggers.idle_clean_enabled = is_checked(hwnd, 230);
    let mut buf_idle = [0u16; 16];
    let len_idle =
        GetWindowTextW(GetDlgItem(hwnd, 231), buf_idle.as_mut_ptr(), buf_idle.len() as i32);
    let text_idle = String::from_utf16_lossy(&buf_idle[..len_idle.max(0) as usize]);
    if let Ok(val) = text_idle.trim().parse::<u32>() {
        cfg.triggers.idle_seconds = clamp_idle_seconds(val);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_triggers_ctrls_unique_and_count() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for &id in CTRLS {
            assert!(seen.insert(id), "Duplicate ID {} in CTRLS", id);
        }
        assert_eq!(CTRLS.len(), 20);
    }
}
