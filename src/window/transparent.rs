/// Windows platform transparent window implementation

#[cfg(target_os = "windows")]
pub mod win32 {
    use windows::Win32::Foundation::{COLORREF, HWND};
    use windows::Win32::UI::WindowsAndMessaging::*;

    /// Setup window for color-key transparency
    pub unsafe fn setup_transparent_window(hwnd: HWND, color_key: u32) {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        // WS_EX_LAYERED + LWA_COLORKEY: color-keyed area visually transparent
        // No WS_EX_TRANSPARENT: whole window captures mouse input
        // Pet click detection done via GetAsyncKeyState + coordinate transform
        let new_style = ex_style
            | WS_EX_LAYERED.0 as i32
            | WS_EX_TOPMOST.0 as i32;
        SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(color_key), 0, LWA_COLORKEY);
    }

    pub fn find_window_by_title(title: &str) -> Option<HWND> {
        use windows::core::PCWSTR;
        let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let hwnd = FindWindowW(None, PCWSTR(wide.as_ptr()));
            if hwnd.0 == 0 {
                None
            } else {
                Some(hwnd)
            }
        }
    }

    pub fn make_window_transparent(title: &str, color_key: u32) -> bool {
        if let Some(hwnd) = find_window_by_title(title) {
            unsafe {
                setup_transparent_window(hwnd, color_key);
            }
            true
        } else {
            false
        }
    }
}

/// Magenta color key (RGB: 255, 0, 255)
pub const COLOR_KEY: u32 = 0x00FF00FF;

/// Magenta color key as macroquad Color
pub const COLOR_KEY_COLOR: macroquad::prelude::Color = macroquad::prelude::Color::new(
    1.0, 0.0, 1.0, 1.0,
);
