/// Win32 layered window with color-key transparency
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const COLOR_KEY: u32 = 0x00FF00FF; // Magenta = transparent

pub fn create_window(
    x: i32, y: i32, width: i32, height: i32,
    wnd_proc: WNDPROC,
    app_ptr: *mut std::ffi::c_void,
) -> Option<HWND> {
    unsafe {
        let class_name: Vec<u16> = "DesktopPet\0"
            .encode_utf16().chain(std::iter::once(0)).collect();

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: wnd_proc,
            hInstance: HINSTANCE::default(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR::null(),
            WS_POPUP,
            x, y, width, height,
            None, None,
            HINSTANCE::default(),
            Some(app_ptr as *const _),
        );

        if hwnd.0 == 0 { return None; }

        // Color-key: all magenta pixels become transparent
        let _ = SetLayeredWindowAttributes(
            hwnd, COLORREF(COLOR_KEY), 0, LWA_COLORKEY);

        let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
        Some(hwnd)
    }
}
