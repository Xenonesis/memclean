use crate::config::{TrayColorMode, TrayConfig, TrayStyle};
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW, CreatePen, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, RoundRect, SelectObject, SetBkMode, SetTextColor,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, FW_BOLD, PS_SOLID, TRANSPARENT,
    DT_CENTER, DT_SINGLELINE, DT_VCENTER,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconFromResourceEx, CreateIconIndirect, LoadIconW, HICON, ICONINFO,
};

const EMBEDDED_ICO: &[u8] = include_bytes!("../../../memclean.ico");

/// Loads an icon at the desired size from the embedded multi-resolution ICO file.
pub fn load_embedded_icon(desired_size: u32) -> HICON {
    unsafe {
        if EMBEDDED_ICO.len() < 6 {
            return std::ptr::null_mut();
        }
        let count = u16::from_le_bytes([EMBEDDED_ICO[4], EMBEDDED_ICO[5]]) as usize;
        let mut best_offset = 0usize;
        let mut best_size = 0usize;
        let mut best_dim = 0u32;
        let mut best_diff = i32::MAX;

        for i in 0..count {
            let entry_off = 6 + i * 16;
            if entry_off + 16 > EMBEDDED_ICO.len() {
                break;
            }
            let raw_w = EMBEDDED_ICO[entry_off];
            let dim = if raw_w == 0 { 256 } else { raw_w as u32 };
            let size = u32::from_le_bytes([
                EMBEDDED_ICO[entry_off + 8],
                EMBEDDED_ICO[entry_off + 9],
                EMBEDDED_ICO[entry_off + 10],
                EMBEDDED_ICO[entry_off + 11],
            ]) as usize;
            let offset = u32::from_le_bytes([
                EMBEDDED_ICO[entry_off + 12],
                EMBEDDED_ICO[entry_off + 13],
                EMBEDDED_ICO[entry_off + 14],
                EMBEDDED_ICO[entry_off + 15],
            ]) as usize;

            let diff = (dim as i32 - desired_size as i32).abs();
            if diff < best_diff {
                best_diff = diff;
                best_dim = dim;
                best_offset = offset;
                best_size = size;
            }
        }

        if best_size > 0 && best_offset + best_size <= EMBEDDED_ICO.len() {
            let res_bytes = &EMBEDDED_ICO[best_offset..best_offset + best_size];
            let hicon = CreateIconFromResourceEx(
                res_bytes.as_ptr(),
                best_size as u32,
                1,
                0x00030000,
                best_dim as i32,
                best_dim as i32,
                0,
            );
            if !hicon.is_null() {
                return hicon;
            }
        }
        std::ptr::null_mut()
    }
}

/// Formats the notification tray icon tooltip with memory and latency facts.
pub fn format_tray_tooltip(percent: u32, standby_mb: u64, timer_res_ms: f32) -> String {
    format!("MemClean: {}% | Standby: {}MB | Timer: {:.1}ms", percent, standby_mb, timer_res_ms)
}

/// Dynamically renders a 16x16 GDI percentage tray badge according to current tray config.
pub fn create_percent_icon_with_config(
    percent: u32,
    tray_cfg: &TrayConfig,
) -> HICON {
    unsafe {
        if tray_cfg.style == TrayStyle::Logo {
            let logo_icon = load_embedded_icon(16);
            if !logo_icon.is_null() {
                return logo_icon;
            }
            let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
            let app_icon = LoadIconW(hinstance, 1 as _);
            if !app_icon.is_null() {
                return app_icon;
            }
        }

        let hdc = CreateCompatibleDC(std::ptr::null_mut());

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = 16;
        bmi.bmiHeader.biHeight = -16; // Top-down DIB so row 0 is top
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbm_color = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);

        let old_bm = SelectObject(hdc, hbm_color);

        // Text color in BGR format
        let text_color = match tray_cfg.color_mode {
            TrayColorMode::Monochrome => 0x00FFFFFF, // Crisp White
            TrayColorMode::Dynamic => {
                if percent < 70 {
                    0x005EC522 // Mint/Forest Green #22C55E (BGR: 0x5E, 0xC5, 0x22)
                } else if percent < 85 {
                    0x000B9EF5 // Warm Amber #F59E0B (BGR: 0x0B, 0x9E, 0xF5)
                } else {
                    0x004444EF // Carmine Coral #EF4444 (BGR: 0x44, 0x44, 0xEF)
                }
            }
        };

        if tray_cfg.style == TrayStyle::Capsule {
            // Draw a rounded capsule container
            let capsule_brush = CreateSolidBrush(0x00241E18); // Dark Slate #181E24
            let capsule_pen = CreatePen(PS_SOLID as i32, 1, 0x0041342A); // Border #2A3441
            let old_brush = SelectObject(hdc, capsule_brush);
            let old_pen = SelectObject(hdc, capsule_pen);
            RoundRect(hdc, 0, 1, 16, 15, 6, 6);
            SelectObject(hdc, old_brush);
            SelectObject(hdc, old_pen);
            DeleteObject(capsule_brush);
            DeleteObject(capsule_pen);
        }

        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, text_color);

        let font = CreateFontW(
            -11, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 1, 0, 0, 5, 0,
            windows_sys::w!("Segoe UI"),
        );
        let old_font = SelectObject(hdc, font);

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 16,
            bottom: 16,
        };
        let text = format!("{:02}", percent.min(99));
        let wide: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(hdc, wide.as_ptr(), wide.len() as i32, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        SelectObject(hdc, old_font);
        DeleteObject(font);

        // Build 1-bit transparency mask (1 = transparent, 0 = opaque)
        let mut mask_bytes = [0xFFu8; 32];
        if !bits.is_null() {
            let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, 256);
            for y in 0..16 {
                for x in 0..16 {
                    let idx = y * 16 + x;
                    if (pixels[idx] & 0x00FFFFFF) != 0 {
                        let byte_idx = y * 2 + (x / 8);
                        let bit_idx = 7 - (x % 8);
                        mask_bytes[byte_idx] &= !(1 << bit_idx);
                        pixels[idx] |= 0xFF000000; // Alpha 255
                    }
                }
            }
        }

        let hbm_mask = CreateBitmap(16, 16, 1, 1, mask_bytes.as_ptr() as _);

        SelectObject(hdc, old_bm);
        DeleteDC(hdc);

        let mut icon_info = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let icon = CreateIconIndirect(&mut icon_info);
        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);

        icon
    }
}

/// Helper that fetches the current configuration and builds the percentage icon.
pub fn create_percent_icon(percent: u32) -> HICON {
    let cfg = crate::config::AppConfig::load();
    create_percent_icon_with_config(percent, &cfg.tray)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tray_tooltip() {
        let tt = format_tray_tooltip(42, 2048, 0.5);
        assert_eq!(tt, "MemClean: 42% | Standby: 2048MB | Timer: 0.5ms");
    }

    #[test]
    fn test_create_percent_icon_all_ranges() {
        let mut cfg = TrayConfig::default();
        cfg.style = TrayStyle::Floating;
        cfg.color_mode = TrayColorMode::Dynamic;
        let icon_green = create_percent_icon_with_config(50, &cfg);
        assert!(!icon_green.is_null());
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon(icon_green); }

        let icon_amber = create_percent_icon_with_config(75, &cfg);
        assert!(!icon_amber.is_null());
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon(icon_amber); }

        let icon_red = create_percent_icon_with_config(90, &cfg);
        assert!(!icon_red.is_null());
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon(icon_red); }
    }
}
