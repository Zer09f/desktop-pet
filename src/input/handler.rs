/// Input handling — WndProc-based mouse input for layered windows
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::app::App;

pub const MENU_CHANGE_SKIN: u32 = 1001;
pub const MENU_FEED: u32 = 1002;
pub const MENU_PET: u32 = 1003;
pub const MENU_PLAY: u32 = 1004;
pub const MENU_SETTINGS: u32 = 1005;
pub const MENU_REMOVE_BG: u32 = 1007;
pub const MENU_QUIT: u32 = 1008;

pub fn register_quit_hotkey(hwnd: HWND) {
    unsafe {
        let _ = RegisterHotKey(hwnd, 1, MOD_CONTROL | MOD_SHIFT, 0x51);
    }
}

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = &*(lparam.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let app = get_app(hwnd);
            if !app.is_null() {
                (*app).on_left_click(0.0, 0.0);
            }
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            let app = get_app(hwnd);
            if !app.is_null() {
                let x = (lparam.0 as i32) & 0xFFFF;
                let y = ((lparam.0 as i32) >> 16) & 0xFFFF;
                (*app).on_right_click(x as f32, y as f32, hwnd);
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let app = get_app(hwnd);
            if !app.is_null() {
                let x = (lparam.0 as i32) & 0xFFFF;
                let y = ((lparam.0 as i32) >> 16) & 0xFFFF;
                (*app).on_mouse_move(x as f32, y as f32);
            }
            // Request WM_MOUSELEAVE notification
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            let _ = TrackMouseEvent(&mut tme);
            LRESULT(0)
        }
        0x02A3 => { // WM_MOUSELEAVE
            let app = get_app(hwnd);
            if !app.is_null() {
                (*app).on_mouse_leave();
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let app = get_app(hwnd);
            if !app.is_null() {
                let menu_id = wparam.0 as u32 & 0xFFFF;
                (*app).on_menu_action(menu_id);
            }
            LRESULT(0)
        }
        WM_HOTKEY => {
            let app = get_app(hwnd);
            if !app.is_null() {
                (*app).running = false;
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let app = get_app(hwnd);
            if !app.is_null() {
                (*app).renderer.paint(hwnd);
            } else {
                let mut ps = PAINTSTRUCT::default();
                let _ = BeginPaint(hwnd, &mut ps);
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn get_app(hwnd: HWND) -> *mut App {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    ptr as *mut App
}


