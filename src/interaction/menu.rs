/// Context menu using Win32 TrackPopupMenuEx
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::input::handler::*;

pub fn show_context_menu(hwnd: HWND, screen_x: i32, screen_y: i32) {
    unsafe {
        let hmenu = CreatePopupMenu().unwrap_or_default();
        if hmenu.is_invalid() {
            return;
        }

        let skin: Vec<u16> = "\u{1f3a8} \u{6362}\u{76ae}\u{80a4}\0"
            .encode_utf16().collect();
        let feed: Vec<u16> = "\u{1f356} \u{5582}\u{98df}\0"
            .encode_utf16().collect();
        let pet_lbl: Vec<u16> = "\u{1f90a} \u{6478}\u{6478}\0"
            .encode_utf16().collect();
        let play: Vec<u16> = "\u{1f3ae} \u{73a9}\u{800d}\0"
            .encode_utf16().collect();
        let settings: Vec<u16> = "\u{2699} \u{8bbe}\u{7f6e}\0"
            .encode_utf16().collect();
        let quit: Vec<u16> = "\u{274c} \u{9000}\u{51fa}\0"
            .encode_utf16().collect();

        let _ = AppendMenuW(hmenu, MF_STRING, MENU_CHANGE_SKIN as usize, windows::core::PCWSTR(skin.as_ptr()));
        let _ = AppendMenuW(hmenu, MF_STRING, MENU_FEED as usize, windows::core::PCWSTR(feed.as_ptr()));
        let _ = AppendMenuW(hmenu, MF_STRING, MENU_PET as usize, windows::core::PCWSTR(pet_lbl.as_ptr()));
        let _ = AppendMenuW(hmenu, MF_STRING, MENU_PLAY as usize, windows::core::PCWSTR(play.as_ptr()));
        let remove_bg: Vec<u16> = "\u{2728} \u{53bb}\u{9664}\u{767d}\u{5e95}\0"
            .encode_utf16().collect();
        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, windows::core::PCWSTR::null());
        let _ = AppendMenuW(hmenu, MF_STRING, MENU_REMOVE_BG as usize, windows::core::PCWSTR(remove_bg.as_ptr()));
        let _ = AppendMenuW(hmenu, MF_STRING, MENU_SETTINGS as usize, windows::core::PCWSTR(settings.as_ptr()));
        let _ = AppendMenuW(hmenu, MF_STRING, MENU_QUIT as usize, windows::core::PCWSTR(quit.as_ptr()));

        // Without TPM_RETURNCMD: selection sends WM_COMMAND to hwnd
        let _ = TrackPopupMenuEx(
            hmenu,
            TPM_RIGHTBUTTON.0,
            screen_x,
            screen_y,
            hwnd,
            None,
        );

        let _ = DestroyMenu(hmenu);
    }
}

