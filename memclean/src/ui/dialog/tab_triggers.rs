use crate::config::{
    clamp_idle_seconds, clamp_islc_mb, clamp_threshold_percent, clamp_timer_seconds, AppConfig,
};
use windows_sys::Win32::Foundation::{HWND, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetDlgItem, GetWindowTextW, SendMessageW, SetWindowTextW, ES_CENTER,
    ES_NUMBER, WM_SETFONT, WS_BORDER, WS_CHILD, WS_VISIBLE,
};

use super::common::{create_chk, font_subheader, font_ui, is_checked, set_checked};

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
    // Card 1: RAM Threshold Controls
    let lbl_thresh = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Auto-optimize when RAM exceeds:"),
        WS_CHILD | WS_VISIBLE,
        30,
        88,
        220,
        20,
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
        254,
        86,
        44,
        22,
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
        304,
        88,
        20,
        20,
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
        30,
        112,
        300,
        16,
        hwnd,
        208 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_note.is_null() && !font_subheader().is_null() {
        SendMessageW(lbl_note, WM_SETFONT, font_subheader() as WPARAM, 1);
    }

    // Card 1: Periodic Auto-Clean Timer Controls
    create_chk(
        hwnd,
        windows_sys::w!("Enable periodic auto-clean timer"),
        30,
        164,
        420,
        209,
    );

    let lbl_timer = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Interval:"),
        WS_CHILD | WS_VISIBLE,
        50,
        190,
        60,
        20,
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
        114,
        188,
        50,
        22,
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
        170,
        190,
        60,
        20,
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
        windows_sys::w!("(Range: 60s - 3600s, Default: 300s)"),
        WS_CHILD | WS_VISIBLE,
        236,
        190,
        240,
        18,
        hwnd,
        213 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_note2.is_null() && !font_subheader().is_null() {
        SendMessageW(lbl_note2, WM_SETFONT, font_subheader() as WPARAM, 1);
    }

    // Card 2: ISLC Standby Trigger Controls
    create_chk(
        hwnd,
        windows_sys::w!("Enable ISLC Standby Trigger (Gaming)"),
        30,
        268,
        420,
        220,
    );

    let lbl_islc_s = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Standby >="),
        WS_CHILD | WS_VISIBLE,
        50,
        294,
        75,
        20,
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
        130,
        292,
        54,
        22,
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
        190,
        294,
        28,
        20,
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
        232,
        294,
        58,
        20,
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
        294,
        292,
        54,
        22,
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
        354,
        294,
        28,
        20,
        hwnd,
        226 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_islc_f_mb.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_islc_f_mb, WM_SETFONT, font_ui() as WPARAM, 1);
    }

    // Card 2: User Inactivity / AFK Idle Clean
    create_chk(
        hwnd,
        windows_sys::w!("Auto-clean on user inactivity / AFK"),
        30,
        352,
        420,
        230,
    );

    let lbl_idle = CreateWindowExW(
        0,
        windows_sys::w!("STATIC"),
        windows_sys::w!("Idle timeout:"),
        WS_CHILD | WS_VISIBLE,
        50,
        376,
        85,
        20,
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
        140,
        374,
        50,
        22,
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
        196,
        376,
        200,
        20,
        hwnd,
        233 as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !lbl_idle_sec.is_null() && !font_ui().is_null() {
        SendMessageW(lbl_idle_sec, WM_SETFONT, font_ui() as WPARAM, 1);
    }
}

/// Populates Tab 1 controls from application configuration.
///
/// # Safety
/// Caller must pass a valid dialog window handle where Tab 1 controls exist.
pub unsafe fn load(hwnd: HWND, cfg: &AppConfig) {
    let thresh_str = format!("{}\0", cfg.triggers.threshold_percent);
    let thresh_wide: Vec<u16> = thresh_str.encode_utf16().collect();
    let edit_thresh = GetDlgItem(hwnd, 205);
    if !edit_thresh.is_null() {
        SetWindowTextW(edit_thresh, thresh_wide.as_ptr());
    }

    set_checked(hwnd, 209, cfg.triggers.auto_timer_enabled);
    let timer_str = format!("{}\0", cfg.triggers.timer_seconds);
    let timer_wide: Vec<u16> = timer_str.encode_utf16().collect();
    let edit_timer = GetDlgItem(hwnd, 211);
    if !edit_timer.is_null() {
        SetWindowTextW(edit_timer, timer_wide.as_ptr());
    }

    set_checked(hwnd, 220, cfg.triggers.islc_enabled);
    let islc_s_str = format!("{}\0", cfg.triggers.islc_standby_mb);
    let islc_s_wide: Vec<u16> = islc_s_str.encode_utf16().collect();
    let edit_islc_standby = GetDlgItem(hwnd, 221);
    if !edit_islc_standby.is_null() {
        SetWindowTextW(edit_islc_standby, islc_s_wide.as_ptr());
    }

    let islc_f_str = format!("{}\0", cfg.triggers.islc_free_mb);
    let islc_f_wide: Vec<u16> = islc_f_str.encode_utf16().collect();
    let edit_islc_free = GetDlgItem(hwnd, 222);
    if !edit_islc_free.is_null() {
        SetWindowTextW(edit_islc_free, islc_f_wide.as_ptr());
    }

    set_checked(hwnd, 230, cfg.triggers.idle_clean_enabled);
    let idle_str = format!("{}\0", cfg.triggers.idle_seconds);
    let idle_wide: Vec<u16> = idle_str.encode_utf16().collect();
    let edit_idle = GetDlgItem(hwnd, 231);
    if !edit_idle.is_null() {
        SetWindowTextW(edit_idle, idle_wide.as_ptr());
    }
}

/// Reads Tab 1 control states and writes them into `AppConfig`.
///
/// # Safety
/// Caller must pass a valid dialog window handle where Tab 1 controls exist.
pub unsafe fn save(hwnd: HWND, cfg: &mut AppConfig) {
    let mut buf = [0u16; 16];
    let len = GetWindowTextW(GetDlgItem(hwnd, 205), buf.as_mut_ptr(), buf.len() as i32);
    let text = String::from_utf16_lossy(&buf[..len.max(0) as usize]);
    if let Ok(val) = text.trim().parse::<u32>() {
        cfg.triggers.threshold_percent = clamp_threshold_percent(val);
    }
    cfg.triggers.auto_timer_enabled = is_checked(hwnd, 209);
    let mut buf_time = [0u16; 16];
    let len_time =
        GetWindowTextW(GetDlgItem(hwnd, 211), buf_time.as_mut_ptr(), buf_time.len() as i32);
    let text_time = String::from_utf16_lossy(&buf_time[..len_time.max(0) as usize]);
    if let Ok(val) = text_time.trim().parse::<u32>() {
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
