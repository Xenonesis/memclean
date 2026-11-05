use windows_sys::Win32::Foundation::{HWND, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, HBRUSH, HFONT, HPEN, PS_SOLID,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetDlgItem, SendMessageW, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX,
    BS_AUTORADIOBUTTON, WM_SETFONT, WS_CHILD, WS_VISIBLE,
};

pub const ODS_SELECTED: u32 = 0x0001;
pub const BST_CHECKED: usize = 1;
pub const BST_UNCHECKED: usize = 0;

pub static mut D_BG_BRUSH: HBRUSH = std::ptr::null_mut();
pub static mut D_CARD_BRUSH: HBRUSH = std::ptr::null_mut();
pub static mut D_BORDER_PEN: HPEN = std::ptr::null_mut();
pub static mut D_FONT_UI: HFONT = std::ptr::null_mut();
pub static mut D_FONT_BOLD: HFONT = std::ptr::null_mut();
pub static mut D_FONT_SUBHEADER: HFONT = std::ptr::null_mut();
pub static mut D_FONT_DESC: HFONT = std::ptr::null_mut();

/// Initializes shared dialog fonts and brushes.
///
/// # Safety
/// Must be called on the main UI thread before controls or dialog cards are painted.
pub unsafe fn init_resources() {
    D_BG_BRUSH = CreateSolidBrush(0x00FAFBFB); // Warm Bone #FBFBFA
    D_CARD_BRUSH = CreateSolidBrush(0x00FFFFFF); // Pure White #FFFFFF
    D_BORDER_PEN = CreatePen(PS_SOLID, 1, 0x00EAEAEA); // Hairline #EAEAEA

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
}

/// Frees shared GDI objects allocated for the dialog.
///
/// # Safety
/// Must be called on dialog destruction when handles are no longer in use.
pub unsafe fn free_resources() {
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
}

/// Returns the primary dialog UI font handle.
///
/// # Safety
/// Requires `init_resources` to have been called.
pub unsafe fn font_ui() -> HFONT {
    D_FONT_UI
}

/// Returns the bold UI font handle.
///
/// # Safety
/// Requires `init_resources` to have been called.
pub unsafe fn font_bold() -> HFONT {
    D_FONT_BOLD
}

/// Returns the subheader font handle.
///
/// # Safety
/// Requires `init_resources` to have been called.
pub unsafe fn font_subheader() -> HFONT {
    D_FONT_SUBHEADER
}

/// Returns the description font handle.
///
/// # Safety
/// Requires `init_resources` to have been called.
pub unsafe fn font_desc() -> HFONT {
    D_FONT_DESC
}

/// Returns the dialog background brush handle.
///
/// # Safety
/// Requires `init_resources` to have been called.
pub unsafe fn bg_brush() -> HBRUSH {
    D_BG_BRUSH
}

/// Returns the dialog card surface brush handle.
///
/// # Safety
/// Requires `init_resources` to have been called.
pub unsafe fn card_brush() -> HBRUSH {
    D_CARD_BRUSH
}

/// Returns the card border pen handle.
///
/// # Safety
/// Requires `init_resources` to have been called.
pub unsafe fn border_pen() -> HPEN {
    D_BORDER_PEN
}

/// Creates a standard Win32 checkbox control.
///
/// # Safety
/// Caller must ensure `hwnd` is a valid window handle and `text` is a valid null-terminated UTF-16 string pointer.
pub unsafe fn create_chk(hwnd: HWND, text: *const u16, x: i32, y: i32, w: i32, id: i32) -> HWND {
    let ctrl = CreateWindowExW(
        0,
        windows_sys::w!("BUTTON"),
        text,
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
        x,
        y,
        w,
        20,
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

/// Creates a standard Win32 auto-radio button control.
///
/// # Safety
/// Caller must ensure `hwnd` is a valid window handle and `text` is a valid null-terminated UTF-16 string pointer.
pub unsafe fn create_radio(hwnd: HWND, text: *const u16, x: i32, y: i32, w: i32, id: i32) -> HWND {
    let ctrl = CreateWindowExW(
        0,
        windows_sys::w!("BUTTON"),
        text,
        WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON as u32,
        x,
        y,
        w,
        20,
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
///
/// # Safety
/// Caller must pass a valid dialog `hwnd`.
pub unsafe fn set_checked(hwnd: HWND, id: i32, checked: bool) {
    let ctrl = GetDlgItem(hwnd, id);
    if !ctrl.is_null() {
        let state = if checked { BST_CHECKED } else { BST_UNCHECKED };
        SendMessageW(ctrl, BM_SETCHECK, state as WPARAM, 0);
    }
}

/// Queries the check state of a button control by ID.
///
/// # Safety
/// Caller must pass a valid dialog `hwnd`.
pub unsafe fn is_checked(hwnd: HWND, id: i32) -> bool {
    let ctrl = GetDlgItem(hwnd, id);
    if !ctrl.is_null() {
        SendMessageW(ctrl, BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT
    } else {
        false
    }
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
