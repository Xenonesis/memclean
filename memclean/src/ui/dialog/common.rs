use windows_sys::Win32::Foundation::{HWND, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, GetStockObject, RoundRect,
    SelectObject, SetBkMode, SetTextColor, HBRUSH, HDC, HFONT, HPEN, DT_CENTER, DT_NOPREFIX,
    DT_SINGLELINE, DT_VCENTER, PS_SOLID, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetDlgItem, SendMessageW, BM_GETCHECK, BM_SETCHECK, BS_OWNERDRAW, WM_SETFONT,
    WS_CHILD, WS_VISIBLE,
};

pub const ODS_SELECTED: u32 = 0x0001;
pub const BST_CHECKED: usize = 1;
pub const BST_UNCHECKED: usize = 0;

// Color System (0x00BBGGRR format for Win32 GDI)
pub const COLOR_BG: u32 = 0x00120D0B; // #0B0D12
pub const COLOR_CARD: u32 = 0x00211814; // #141821
pub const COLOR_SURFACE: u32 = 0x0030221C; // #1C2230
pub const COLOR_SURFACE_HOVER: u32 = 0x003D2E26; // #262E3D
pub const COLOR_ACCENT: u32 = 0x00FF7C8B; // #8B7CFF
pub const COLOR_ACCENT_HOVER: u32 = 0x00FF909D; // #9D90FF
pub const COLOR_ACCENT_DARK: u32 = 0x00E66A7A; // #7A6AE6
pub const COLOR_SUCCESS: u32 = 0x009AD335; // #35D39A
pub const COLOR_WARNING: u32 = 0x0054B4FF; // #FFB454
pub const COLOR_DANGER: u32 = 0x005454FF; // #FF5454
pub const COLOR_TEXT_PRIMARY: u32 = 0x00FCF6F4; // #F4F6FC
pub const COLOR_TEXT_SECONDARY: u32 = 0x00ADB992; // #929BAD
pub const COLOR_TEXT_MUTED: u32 = 0x00787A73; // #737A78
pub const COLOR_BORDER: u32 = 0x003B2B23; // #232B3B
pub const COLOR_BORDER_HOVER: u32 = 0x00554035; // #354055
pub const COLOR_BORDER_ACCENT: u32 = 0x00FF7C8B; // #8B7CFF

pub static mut D_BG_BRUSH: HBRUSH = std::ptr::null_mut();
pub static mut D_CARD_BRUSH: HBRUSH = std::ptr::null_mut();
pub static mut D_SURFACE_BRUSH: HBRUSH = std::ptr::null_mut();
pub static mut D_SURFACE_HOVER_BRUSH: HBRUSH = std::ptr::null_mut();
pub static mut D_ACCENT_BRUSH: HBRUSH = std::ptr::null_mut();
pub static mut D_SUCCESS_BRUSH: HBRUSH = std::ptr::null_mut();

pub static mut D_BORDER_PEN: HPEN = std::ptr::null_mut();
pub static mut D_BORDER_HOVER_PEN: HPEN = std::ptr::null_mut();
pub static mut D_ACCENT_PEN: HPEN = std::ptr::null_mut();
pub static mut D_ACCENT_PEN_THICK: HPEN = std::ptr::null_mut();
pub static mut D_DIVIDER_PEN: HPEN = std::ptr::null_mut();

pub static mut D_FONT_TITLE: HFONT = std::ptr::null_mut();
pub static mut D_FONT_HEADER: HFONT = std::ptr::null_mut();
pub static mut D_FONT_UI: HFONT = std::ptr::null_mut();
pub static mut D_FONT_BOLD: HFONT = std::ptr::null_mut();
pub static mut D_FONT_SUBHEADER: HFONT = std::ptr::null_mut();
pub static mut D_FONT_DESC: HFONT = std::ptr::null_mut();
pub static mut D_FONT_BADGE: HFONT = std::ptr::null_mut();
pub static mut D_FONT_STATS_BIG: HFONT = std::ptr::null_mut();

pub const CHECKED_STATES_LEN: usize = 1000;
pub static mut CHECKED_STATES: [bool; CHECKED_STATES_LEN] = [false; CHECKED_STATES_LEN];

static mut PREV_IDLE: u64 = 0;
static mut PREV_KERNEL: u64 = 0;
static mut PREV_USER: u64 = 0;
static mut CACHED_CPU_PERCENT: u32 = 0;

/// Query live system CPU usage using Win32 GetSystemTimes.
pub fn get_live_cpu_usage() -> u32 {
    unsafe {
        let mut idle = std::mem::zeroed();
        let mut kernel = std::mem::zeroed();
        let mut user = std::mem::zeroed();
        if windows_sys::Win32::System::Threading::GetSystemTimes(&mut idle, &mut kernel, &mut user)
            != 0
        {
            let idle_u64 = ((idle.dwHighDateTime as u64) << 32) | (idle.dwLowDateTime as u64);
            let kernel_u64 =
                ((kernel.dwHighDateTime as u64) << 32) | (kernel.dwLowDateTime as u64);
            let user_u64 = ((user.dwHighDateTime as u64) << 32) | (user.dwLowDateTime as u64);

            if PREV_KERNEL != 0 || PREV_USER != 0 {
                let diff_idle = idle_u64.saturating_sub(PREV_IDLE);
                let diff_kernel = kernel_u64.saturating_sub(PREV_KERNEL);
                let diff_user = user_u64.saturating_sub(PREV_USER);
                let total = diff_kernel + diff_user;
                if total > 0 && total >= diff_idle {
                    let pct = ((total - diff_idle) * 100) / total;
                    CACHED_CPU_PERCENT = (pct as u32).min(100);
                }
            }
            PREV_IDLE = idle_u64;
            PREV_KERNEL = kernel_u64;
            PREV_USER = user_u64;
        }
        CACHED_CPU_PERCENT
    }
}

/// Initializes shared dialog dark theme fonts, brushes, and pens.
pub unsafe fn init_resources() {
    D_BG_BRUSH = CreateSolidBrush(COLOR_BG);
    D_CARD_BRUSH = CreateSolidBrush(COLOR_CARD);
    D_SURFACE_BRUSH = CreateSolidBrush(COLOR_SURFACE);
    D_SURFACE_HOVER_BRUSH = CreateSolidBrush(COLOR_SURFACE_HOVER);
    D_ACCENT_BRUSH = CreateSolidBrush(COLOR_ACCENT);
    D_SUCCESS_BRUSH = CreateSolidBrush(COLOR_SUCCESS);

    D_BORDER_PEN = CreatePen(PS_SOLID, 1, COLOR_BORDER);
    D_BORDER_HOVER_PEN = CreatePen(PS_SOLID, 1, COLOR_BORDER_HOVER);
    D_ACCENT_PEN = CreatePen(PS_SOLID, 1, COLOR_ACCENT);
    D_ACCENT_PEN_THICK = CreatePen(PS_SOLID, 2, COLOR_ACCENT);
    D_DIVIDER_PEN = CreatePen(PS_SOLID, 1, 0x002B211A);

    let font_name = windows_sys::w!("Segoe UI");

    D_FONT_TITLE = CreateFontW(
        -18, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0,
        font_name,
    );
    D_FONT_HEADER = CreateFontW(
        -15, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
        font_name,
    );
    D_FONT_UI = CreateFontW(
        -13, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0,
        font_name,
    );
    D_FONT_BOLD = CreateFontW(
        -13, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
        font_name,
    );
    D_FONT_SUBHEADER = CreateFontW(
        -11, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0,
        font_name,
    );
    D_FONT_DESC = CreateFontW(
        -11, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0,
        font_name,
    );
    D_FONT_BADGE = CreateFontW(
        -10, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0,
        font_name,
    );
    D_FONT_STATS_BIG = CreateFontW(
        -14, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0,
        font_name,
    );
}

/// Frees shared GDI objects allocated for the dialog.
pub unsafe fn free_resources() {
    macro_rules! free_gdi {
        ($obj:expr) => {
            if !$obj.is_null() {
                DeleteObject($obj as _);
                $obj = std::ptr::null_mut();
            }
        };
    }

    free_gdi!(D_BG_BRUSH);
    free_gdi!(D_CARD_BRUSH);
    free_gdi!(D_SURFACE_BRUSH);
    free_gdi!(D_SURFACE_HOVER_BRUSH);
    free_gdi!(D_ACCENT_BRUSH);
    free_gdi!(D_SUCCESS_BRUSH);

    free_gdi!(D_BORDER_PEN);
    free_gdi!(D_BORDER_HOVER_PEN);
    free_gdi!(D_ACCENT_PEN);
    free_gdi!(D_ACCENT_PEN_THICK);
    free_gdi!(D_DIVIDER_PEN);

    free_gdi!(D_FONT_TITLE);
    free_gdi!(D_FONT_HEADER);
    free_gdi!(D_FONT_UI);
    free_gdi!(D_FONT_BOLD);
    free_gdi!(D_FONT_SUBHEADER);
    free_gdi!(D_FONT_DESC);
    free_gdi!(D_FONT_BADGE);
    free_gdi!(D_FONT_STATS_BIG);
}

pub unsafe fn font_title() -> HFONT {
    D_FONT_TITLE
}
pub unsafe fn font_header() -> HFONT {
    D_FONT_HEADER
}
pub unsafe fn font_ui() -> HFONT {
    D_FONT_UI
}
pub unsafe fn font_bold() -> HFONT {
    D_FONT_BOLD
}
pub unsafe fn font_subheader() -> HFONT {
    D_FONT_SUBHEADER
}
pub unsafe fn font_desc() -> HFONT {
    D_FONT_DESC
}
pub unsafe fn font_badge() -> HFONT {
    D_FONT_BADGE
}
pub unsafe fn font_stats_big() -> HFONT {
    D_FONT_STATS_BIG
}
pub unsafe fn bg_brush() -> HBRUSH {
    D_BG_BRUSH
}
pub unsafe fn card_brush() -> HBRUSH {
    D_CARD_BRUSH
}
pub unsafe fn surface_brush() -> HBRUSH {
    D_SURFACE_BRUSH
}
pub unsafe fn border_pen() -> HPEN {
    D_BORDER_PEN
}

/// Creates a standard Win32 owner-drawn toggle/button control.
pub unsafe fn create_chk(hwnd: HWND, text: *const u16, x: i32, y: i32, w: i32, id: i32) -> HWND {
    let ctrl = CreateWindowExW(
        0,
        windows_sys::w!("BUTTON"),
        text,
        WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
        x,
        y,
        w,
        42,
        hwnd,
        id as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !ctrl.is_null() && !D_FONT_UI.is_null() {
        SendMessageW(ctrl, WM_SETFONT, D_FONT_UI as WPARAM, 1);
    }
    ctrl
}

/// Creates a standard Win32 owner-drawn radio/segmented button control.
pub unsafe fn create_radio(hwnd: HWND, text: *const u16, x: i32, y: i32, w: i32, id: i32) -> HWND {
    let ctrl = CreateWindowExW(
        0,
        windows_sys::w!("BUTTON"),
        text,
        WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
        x,
        y,
        w,
        34,
        hwnd,
        id as _,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !ctrl.is_null() && !D_FONT_UI.is_null() {
        SendMessageW(ctrl, WM_SETFONT, D_FONT_UI as WPARAM, 1);
    }
    ctrl
}

/// Sets the check state of a button control by ID.
pub unsafe fn set_checked(hwnd: HWND, id: i32, checked: bool) {
    if id >= 0 && (id as usize) < CHECKED_STATES_LEN {
        CHECKED_STATES[id as usize] = checked;
    }
    let ctrl = GetDlgItem(hwnd, id);
    if !ctrl.is_null() {
        let state = if checked { BST_CHECKED } else { BST_UNCHECKED };
        SendMessageW(ctrl, BM_SETCHECK, state as WPARAM, 0);
        windows_sys::Win32::Graphics::Gdi::InvalidateRect(ctrl, std::ptr::null(), 1);
    }
}

/// Queries the check state of a button control by ID.
pub unsafe fn is_checked(hwnd: HWND, id: i32) -> bool {
    let ctrl = GetDlgItem(hwnd, id);
    if !ctrl.is_null() {
        let win_chk = SendMessageW(ctrl, BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT;
        if win_chk {
            return true;
        }
    }
    if id >= 0 && (id as usize) < CHECKED_STATES_LEN {
        CHECKED_STATES[id as usize]
    } else {
        false
    }
}

/// Helper to render a rounded rectangle with custom brush and pen.
pub unsafe fn draw_rounded_card(
    hdc: HDC,
    rc: &RECT,
    r: i32,
    bg_brush: HBRUSH,
    border_pen: HPEN,
) {
    let old_brush = SelectObject(hdc, bg_brush);
    let old_pen = SelectObject(hdc, border_pen);
    RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, r, r);
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
}

/// Draws a modern Windows 11-style toggle switch row.
pub unsafe fn draw_toggle_row(
    hdc: HDC,
    rc: &RECT,
    title: &str,
    desc: &str,
    is_on: bool,
    is_hover: bool,
) {
    let card_bg = if is_hover {
        D_SURFACE_HOVER_BRUSH
    } else {
        D_CARD_BRUSH
    };
    let border = if is_hover {
        D_BORDER_HOVER_PEN
    } else {
        D_BORDER_PEN
    };

    draw_rounded_card(hdc, rc, 8, card_bg, border);

    SetBkMode(hdc, TRANSPARENT as i32);

    // Title
    SetTextColor(hdc, COLOR_TEXT_PRIMARY);
    let old_font = SelectObject(hdc, font_bold());
    let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let mut title_rc = RECT {
        left: rc.left + 14,
        top: rc.top + 6,
        right: rc.right - 80,
        bottom: rc.top + 22,
    };
    DrawTextW(
        hdc,
        wide_title.as_ptr(),
        -1,
        &mut title_rc,
        DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );

    // Description
    SetTextColor(hdc, COLOR_TEXT_SECONDARY);
    SelectObject(hdc, font_desc());
    let wide_desc: Vec<u16> = desc.encode_utf16().chain(std::iter::once(0)).collect();
    let mut desc_rc = RECT {
        left: rc.left + 14,
        top: rc.top + 22,
        right: rc.right - 80,
        bottom: rc.bottom - 4,
    };
    DrawTextW(
        hdc,
        wide_desc.as_ptr(),
        -1,
        &mut desc_rc,
        DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );

    // Switch Track on Right: (Width: 44, Height: 22)
    let track_w = 44;
    let track_h = 22;
    let track_x = rc.right - track_w - 14;
    let track_y = rc.top + (rc.bottom - rc.top - track_h) / 2;
    let track_rc = RECT {
        left: track_x,
        top: track_y,
        right: track_x + track_w,
        bottom: track_y + track_h,
    };

    let track_brush = if is_on {
        D_ACCENT_BRUSH
    } else {
        D_SURFACE_BRUSH
    };
    let track_pen = if is_on {
        D_ACCENT_PEN
    } else {
        D_BORDER_PEN
    };
    draw_rounded_card(hdc, &track_rc, track_h, track_brush, track_pen);

    // Switch Thumb: 16x16 circle
    let thumb_size = 16;
    let thumb_x = if is_on {
        track_x + track_w - thumb_size - 3
    } else {
        track_x + 3
    };
    let thumb_y = track_y + (track_h - thumb_size) / 2;
    let thumb_rc = RECT {
        left: thumb_x,
        top: thumb_y,
        right: thumb_x + thumb_size,
        bottom: thumb_y + thumb_size,
    };

    let thumb_color = if is_on {
        COLOR_TEXT_PRIMARY
    } else {
        COLOR_TEXT_SECONDARY
    };
    let thumb_brush = CreateSolidBrush(thumb_color);
    let old_brush = SelectObject(hdc, thumb_brush);
    let old_pen = SelectObject(hdc, GetStockObject(windows_sys::Win32::Graphics::Gdi::NULL_PEN as i32));
    RoundRect(
        hdc,
        thumb_rc.left,
        thumb_rc.top,
        thumb_rc.right,
        thumb_rc.bottom,
        thumb_size,
        thumb_size,
    );
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    DeleteObject(thumb_brush);

    // Status Badge label
    SelectObject(hdc, font_badge());
    SetTextColor(hdc, if is_on { COLOR_SUCCESS } else { COLOR_TEXT_MUTED });
    let state_text: *const u16 = if is_on {
        windows_sys::w!("ON")
    } else {
        windows_sys::w!("OFF")
    };
    let mut badge_rc = RECT {
        left: track_x - 32,
        top: track_y,
        right: track_x - 6,
        bottom: track_y + track_h,
    };
    DrawTextW(
        hdc,
        state_text,
        -1,
        &mut badge_rc,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    SelectObject(hdc, old_font);
}

/// Draws an interactive Tray Icon Preview Card with live mini taskbar preview.
pub unsafe fn draw_tray_preview_card(
    hdc: HDC,
    rc: &RECT,
    style_idx: usize,
    title: &str,
    is_active: bool,
    is_hover: bool,
) {
    let bg = if is_active {
        D_SURFACE_BRUSH
    } else if is_hover {
        D_SURFACE_HOVER_BRUSH
    } else {
        D_CARD_BRUSH
    };
    let pen = if is_active {
        D_ACCENT_PEN_THICK
    } else if is_hover {
        D_BORDER_HOVER_PEN
    } else {
        D_BORDER_PEN
    };

    draw_rounded_card(hdc, rc, 10, bg, pen);

    // Mini Taskbar Preview Area (Top half)
    let preview_rc = RECT {
        left: rc.left + 8,
        top: rc.top + 8,
        right: rc.right - 8,
        bottom: rc.top + 50,
    };
    let tb_brush = CreateSolidBrush(0x000F0D0B); // Simulated dark taskbar
    let tb_pen = CreatePen(PS_SOLID, 1, 0x00261D17);
    draw_rounded_card(hdc, &preview_rc, 6, tb_brush, tb_pen);
    DeleteObject(tb_pen);
    DeleteObject(tb_brush);

    SetBkMode(hdc, TRANSPARENT as i32);

    match style_idx {
        0 => {
            // Floating Number (42)
            let old_font = SelectObject(hdc, font_stats_big());
            SetTextColor(hdc, COLOR_SUCCESS);
            let mut num_rc = preview_rc;
            DrawTextW(
                hdc,
                windows_sys::w!("42"),
                -1,
                &mut num_rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            SelectObject(hdc, old_font);
        }
        1 => {
            // Capsule Badge: Rounded pill with "42%"
            let cap_w = 46;
            let cap_h = 22;
            let cap_x = preview_rc.left + (preview_rc.right - preview_rc.left - cap_w) / 2;
            let cap_y = preview_rc.top + (preview_rc.bottom - preview_rc.top - cap_h) / 2;
            let cap_rc = RECT {
                left: cap_x,
                top: cap_y,
                right: cap_x + cap_w,
                bottom: cap_y + cap_h,
            };
            let cap_brush = CreateSolidBrush(0x00241A14);
            let cap_pen = CreatePen(PS_SOLID, 1, 0x00453326);
            draw_rounded_card(hdc, &cap_rc, 10, cap_brush, cap_pen);
            DeleteObject(cap_pen);
            DeleteObject(cap_brush);

            let old_font = SelectObject(hdc, font_badge());
            SetTextColor(hdc, COLOR_TEXT_PRIMARY);
            let mut text_rc = cap_rc;
            DrawTextW(
                hdc,
                windows_sys::w!("42%"),
                -1,
                &mut text_rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            SelectObject(hdc, old_font);
        }
        _ => {
            // Monogram Logo Icon: 'M' Emblem
            let m_w = 26;
            let m_h = 26;
            let m_x = preview_rc.left + (preview_rc.right - preview_rc.left - m_w) / 2;
            let m_y = preview_rc.top + (preview_rc.bottom - preview_rc.top - m_h) / 2;
            let m_rc = RECT {
                left: m_x,
                top: m_y,
                right: m_x + m_w,
                bottom: m_y + m_h,
            };
            let m_brush = CreateSolidBrush(COLOR_ACCENT);
            let m_pen = CreatePen(PS_SOLID, 1, COLOR_ACCENT_HOVER);
            draw_rounded_card(hdc, &m_rc, 6, m_brush, m_pen);
            DeleteObject(m_pen);
            DeleteObject(m_brush);

            let old_font = SelectObject(hdc, font_bold());
            SetTextColor(hdc, 0x000F0D0B);
            let mut letter_rc = m_rc;
            DrawTextW(
                hdc,
                windows_sys::w!("M"),
                -1,
                &mut letter_rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            SelectObject(hdc, old_font);
        }
    }

    // Bottom title + active indicator
    let old_font = SelectObject(hdc, font_bold());
    SetTextColor(hdc, if is_active { COLOR_TEXT_PRIMARY } else { COLOR_TEXT_SECONDARY });
    let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let mut title_rc = RECT {
        left: rc.left + 4,
        top: preview_rc.bottom + 4,
        right: rc.right - 4,
        bottom: preview_rc.bottom + 20,
    };
    DrawTextW(
        hdc,
        wide_title.as_ptr(),
        -1,
        &mut title_rc,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    // Active pill badge
    SelectObject(hdc, font_badge());
    SetTextColor(hdc, if is_active { COLOR_ACCENT } else { COLOR_TEXT_MUTED });
    let mut badge_rc = RECT {
        left: rc.left + 4,
        top: title_rc.bottom,
        right: rc.right - 4,
        bottom: rc.bottom - 4,
    };
    let badge_text: *const u16 = if is_active {
        windows_sys::w!("● Active")
    } else {
        windows_sys::w!("Select")
    };
    DrawTextW(
        hdc,
        badge_text,
        -1,
        &mut badge_rc,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );
    SelectObject(hdc, old_font);
}

/// Draws modern segmented option controls.
pub unsafe fn draw_segmented_btn(
    hdc: HDC,
    rc: &RECT,
    text: &str,
    is_active: bool,
    is_hover: bool,
) {
    let bg = if is_active {
        D_SURFACE_BRUSH
    } else if is_hover {
        D_SURFACE_HOVER_BRUSH
    } else {
        D_CARD_BRUSH
    };
    let pen = if is_active {
        D_ACCENT_PEN
    } else if is_hover {
        D_BORDER_HOVER_PEN
    } else {
        D_BORDER_PEN
    };

    draw_rounded_card(hdc, rc, 6, bg, pen);

    SetBkMode(hdc, TRANSPARENT as i32);
    let old_font = SelectObject(hdc, if is_active { font_bold() } else { font_ui() });
    SetTextColor(hdc, if is_active { COLOR_TEXT_PRIMARY } else { COLOR_TEXT_SECONDARY });

    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let mut text_rc = *rc;
    DrawTextW(
        hdc,
        wide.as_ptr(),
        -1,
        &mut text_rc,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
    );
    SelectObject(hdc, old_font);
}

/// Draws prominent footer action buttons.
pub unsafe fn draw_action_btn(
    hdc: HDC,
    rc: &RECT,
    text: &str,
    is_primary: bool,
    is_danger: bool,
    is_hover: bool,
    is_pressed: bool,
) {
    let (bg_color, border_color, text_color) = if is_primary {
        if is_pressed {
            (COLOR_ACCENT_DARK, COLOR_ACCENT_DARK, 0x00FFFFFF)
        } else if is_hover {
            (COLOR_ACCENT_HOVER, COLOR_ACCENT_HOVER, 0x00FFFFFF)
        } else {
            (COLOR_ACCENT, COLOR_ACCENT, 0x00FFFFFF)
        }
    } else if is_danger {
        if is_hover {
            (COLOR_SURFACE_HOVER, COLOR_WARNING, COLOR_WARNING)
        } else {
            (COLOR_CARD, COLOR_BORDER, COLOR_WARNING)
        }
    } else {
        if is_hover {
            (COLOR_SURFACE_HOVER, COLOR_BORDER_HOVER, COLOR_TEXT_PRIMARY)
        } else {
            (COLOR_CARD, COLOR_BORDER, COLOR_TEXT_SECONDARY)
        }
    };

    let brush = CreateSolidBrush(bg_color);
    let pen = CreatePen(PS_SOLID, 1, border_color);
    draw_rounded_card(hdc, rc, 8, brush, pen);
    DeleteObject(pen);
    DeleteObject(brush);

    SetBkMode(hdc, TRANSPARENT as i32);
    let old_font = SelectObject(hdc, font_bold());
    SetTextColor(hdc, text_color);
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let mut text_rc = *rc;
    DrawTextW(
        hdc,
        wide.as_ptr(),
        -1,
        &mut text_rc,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );
    SelectObject(hdc, old_font);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(ODS_SELECTED, 0x0001);
        assert_eq!(BST_CHECKED, 1);
        assert_eq!(BST_UNCHECKED, 0);
    }
}
