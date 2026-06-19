//! Desktop Pet — walks along your screen edge, no application window frame
mod game;
mod window;
mod renderer;
mod pet;
mod input;
mod interaction;
mod app;

use windows::Win32::Foundation::*;
use windows::Win32::System::Performance::*;
use windows::Win32::System::Threading::Sleep;
use windows::Win32::UI::WindowsAndMessaging::*;

use app::App;
use input::handler::{register_quit_hotkey, wnd_proc};

const UPDATE_INTERVAL: f64 = 1.0 / 30.0;

fn main() {
    unsafe {
        let _ = SetProcessDPIAware();

        let screen_w = GetSystemMetrics(SM_CXSCREEN) as f32;
        let screen_h = GetSystemMetrics(SM_CYSCREEN) as f32;

        // Get work area (screen minus taskbar)
        let mut work_rect = RECT::default();
        let _ = SystemParametersInfoW(SPI_GETWORKAREA, 0,
            Some(&mut work_rect as *mut _ as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));

        let config = game::config::PetConfig::load();

        let pet_w = config.pet_width as i32;
        let pet_h = config.pet_height as i32;
        let x = ((screen_w as i32) - pet_w) / 2;
        let y = work_rect.bottom - pet_h; // above taskbar

        let mut app_box = Box::new(
            App::new(HWND::default(), screen_w, work_rect.bottom as f32)
                .expect("Failed to init — check assets/pet.png exists"),
        );

        let app_ptr: *mut App = &mut *app_box;

        let hwnd = window::layered::create_window(
            x, y, pet_w, pet_h,
            Some(wnd_proc),
            app_ptr as *mut std::ffi::c_void,
        ).expect("Failed to create layered window");

        app_box.renderer = renderer::draw::Renderer::new(hwnd, pet_w, pet_h)
            .expect("Failed to create renderer");

        register_quit_hotkey(hwnd);
        let _ = SetForegroundWindow(hwnd);

        let mut freq = 0i64;
        QueryPerformanceFrequency(&mut freq).ok();
        let freq = freq as f64;

        let mut last_time = 0i64;
        QueryPerformanceCounter(&mut last_time).ok();
        let mut accumulator = 0.0f64;

        let mut msg = MSG::default();
        loop {
            loop {
                let has_msg = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE);
                if has_msg.as_bool() {
                    if msg.message == WM_QUIT {
                        save_and_exit(&mut app_box);
                        return;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else {
                    break;
                }
            }

            if !app_box.running {
                save_and_exit(&mut app_box);
                break;
            }

            let mut now = 0i64;
            QueryPerformanceCounter(&mut now).ok();
            let dt = (now - last_time) as f64 / freq;
            last_time = now;
            accumulator += dt;

            while accumulator >= UPDATE_INTERVAL {
                app_box.update(UPDATE_INTERVAL as f32);
                let _ = SetWindowPos(
                    hwnd, HWND_TOPMOST,
                    app_box.pet.pos_x as i32,
                    app_box.pet.pos_y as i32,
                    0, 0,
                    SWP_NOSIZE | SWP_NOACTIVATE,
                );
                accumulator -= UPDATE_INTERVAL;
            }

            app_box.render();
            Sleep(1);
        }
    }
}

fn save_and_exit(app: &mut app::App) {
    app.save_data.affection = app.pet.affection;
    let _ = app.save_data.save();
}
