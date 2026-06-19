/// Renderer — DIB section + BitBlt, with magenta background for LWA_COLORKEY
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::window::layered::COLOR_KEY;

pub struct SpriteImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // BGRA, top-down
}

impl SpriteImage {
    pub fn from_bgra(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self { width, height, pixels }
    }
}

pub struct Renderer {
    hwnd: HWND,
    width: i32,
    height: i32,
    hdc: HDC,
    hbmp: HBITMAP,
    old_bmp: HGDIOBJ,
    bits: *mut u8,
}

impl Renderer {
    pub fn new(hwnd: HWND, width: i32, height: i32) -> Option<Self> {
        unsafe {
            let hdc = CreateCompatibleDC(None);
            if hdc.is_invalid() { return None; }

            let bi = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                biSizeImage: (width * height * 4) as u32,
                ..Default::default()
            };

            let mut bits_ptr: *mut u8 = std::ptr::null_mut();
            let hbmp = CreateDIBSection(
                hdc,
                &BITMAPINFO { bmiHeader: bi, bmiColors: [RGBQUAD::default(); 1] },
                DIB_RGB_COLORS,
                &mut bits_ptr as *mut *mut u8 as *mut *mut std::ffi::c_void,
                None, 0,
            ).ok()?;

            if hbmp.is_invalid() || bits_ptr.is_null() {
                let _ = DeleteDC(hdc);
                return None;
            }

            let old_bmp = SelectObject(hdc, hbmp);
            Some(Self { hwnd, width, height, hdc, hbmp, old_bmp, bits: bits_ptr })
        }
    }

    pub fn hdc(&self) -> HDC { self.hdc }

    /// Fill DIB with magenta (the color-key transparent color)
    fn clear_to_magenta(&self) {
        unsafe {
            let count = (self.width * self.height) as usize;
            let dst = std::slice::from_raw_parts_mut(self.bits, count * 4);
            for i in 0..count {
                let off = i * 4;
                dst[off]     = 0xFF; // B
                dst[off + 1] = 0x00; // G
                dst[off + 2] = 0xFF; // R
                dst[off + 3] = 0xFF; // A
            }
        }
    }

    /// BitBlt to window — SetLayeredWindowAttributes(LWA_COLORKEY) makes magenta transparent
    pub fn update_window(&self) {
        unsafe {
            let hdc_wnd = GetDC(self.hwnd);
            if !hdc_wnd.is_invalid() {
                let _ = BitBlt(hdc_wnd, 0, 0, self.width, self.height,
                    self.hdc, 0, 0, SRCCOPY);
                let _ = ReleaseDC(self.hwnd, hdc_wnd);
            }
            let _ = SetWindowPos(
                self.hwnd, HWND_TOPMOST, 0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }

    /// Paint via WM_PAINT (for system-triggered repaints)
    pub fn paint(&self, hwnd: HWND) {
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            let hdc_wnd = BeginPaint(hwnd, &mut ps);
            let _ = BitBlt(hdc_wnd, 0, 0, self.width, self.height,
                self.hdc, 0, 0, SRCCOPY);
            let _ = EndPaint(hwnd, &ps);
        }
    }

    pub fn prepare_sprite(&self, sprite: &SpriteImage) {
        self.clear_to_magenta();
        self.blit_scaled(sprite, 0, 0, sprite.width, sprite.height);
    }

    pub fn prepare_frame(
        &self, sheet: &SpriteImage,
        frame_x: u32, frame_y: u32, frame_w: u32, frame_h: u32,
        facing_right: bool,
    ) {
        self.clear_to_magenta();
        if frame_w == 0 || frame_h == 0 { return; }

        unsafe {
            let dst = std::slice::from_raw_parts_mut(
                self.bits, (self.width * self.height * 4) as usize);
            let dw = self.width as usize;
            let dh = self.height as usize;
            let sw = sheet.width as usize;
            let fw = frame_w as usize;
            let fh = frame_h as usize;
            let fx0 = frame_x as usize;
            let fy0 = frame_y as usize;

            for dy in 0..dh {
                let sy = fy0 + dy * fh / dh;
                for dx in 0..dw {
                    let raw = dx * fw / dw;
                    let sx = fx0 + raw;
                    let si = (sy * sw + sx) * 4;
                    let di = (dy * dw + dx) * 4;
                    // Only draw opaque pixels; transparent ones keep magenta
                    if sheet.pixels[si + 3] > 0 {
                        dst[di]     = sheet.pixels[si];
                        dst[di + 1] = sheet.pixels[si + 1];
                        dst[di + 2] = sheet.pixels[si + 2];
                        dst[di + 3] = sheet.pixels[si + 3];
                    }
                }
            }
        }
    }

    fn blit_scaled(&self, sprite: &SpriteImage, sx: u32, sy: u32, sw: u32, sh: u32) {
        if sw == 0 || sh == 0 { return; }
        unsafe {
            let dst = std::slice::from_raw_parts_mut(
                self.bits, (self.width * self.height * 4) as usize);
            let dw = self.width as usize;
            let dh = self.height as usize;
            let stride = sprite.width as usize;

            for dy in 0..dh {
                let ry = (sy as usize) + dy * (sh as usize) / dh;
                for dx in 0..dw {
                    let rx = (sx as usize) + dx * (sw as usize) / dw;
                    let si = (ry * stride + rx) * 4;
                    let di = (dy * dw + dx) * 4;
                    if sprite.pixels[si + 3] > 0 {
                        dst[di]     = sprite.pixels[si];
                        dst[di + 1] = sprite.pixels[si + 1];
                        dst[di + 2] = sprite.pixels[si + 2];
                        dst[di + 3] = sprite.pixels[si + 3];
                    }
                }
            }
        }
    }

    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.hdc, self.old_bmp);
            let _ = DeleteObject(self.hbmp);
            let _ = DeleteDC(self.hdc);
        }
    }
}


