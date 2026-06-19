/// Text bubble overlay using GDI
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::*;

pub struct Bubble {
    pub text: String,
    pub visible: bool,
    pub timer: f32,
    pub duration: f32,
    float_offset: f32,
}

impl Bubble {
    pub fn new() -> Self {
        Self { text: String::new(), visible: false, timer: 0.0, duration: 3.0, float_offset: 0.0 }
    }

    pub fn show(&mut self, text: String, duration: f32) {
        self.text = text;
        self.visible = true;
        self.timer = 0.0;
        self.duration = duration;
        self.float_offset = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if !self.visible { return; }
        self.timer += dt;
        self.float_offset = (self.timer * 2.0).sin() * 2.0;
        if self.timer >= self.duration { self.visible = false; }
    }

    /// Draw bubble at the top of the window DIB.
    /// `center_x` — horizontal center in DIB coords
    /// `dib_w`, `dib_h` — DIB dimensions
    pub fn draw_to_hdc(&self, hdc: HDC, center_x: i32, dib_w: i32, dib_h: i32) {
        if !self.visible || self.text.is_empty() { return; }

        unsafe {
            let text_wide: Vec<u16> = self.text.encode_utf16().chain(std::iter::once(0)).collect();

            let mut size = SIZE { cx: 0, cy: 0 };
            let _ = GetTextExtentPoint32W(hdc, &text_wide, &mut size);

            let padding = 6i32;
            let bubble_w = size.cx + padding * 2;
            let bubble_h = size.cy + padding * 2;

            // Position at top of DIB, with some margin
            let bubble_x = center_x - bubble_w / 2;
            let bubble_y = (self.float_offset as i32).max(0);

            // Clip to DIB bounds
            if bubble_x < 0 || bubble_y + bubble_h > dib_h || bubble_x + bubble_w > dib_w {
                return;
            }

            let rect = RECT {
                left: bubble_x, top: bubble_y,
                right: bubble_x + bubble_w, bottom: bubble_y + bubble_h,
            };

            // White background
            let bg_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
            let _ = FillRect(hdc, &rect, bg_brush);
            let _ = DeleteObject(bg_brush);

            // Gray border
            let border_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00808080));
            let old_pen = SelectObject(hdc, border_pen);
            let old_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH));
            let _ = Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brush);
            let _ = DeleteObject(border_pen);

            // Dark text
            let _ = SetBkMode(hdc, TRANSPARENT);
            let _ = SetTextColor(hdc, COLORREF(0x00323232));
            let _ = TextOutW(hdc, bubble_x + padding, bubble_y + padding, &text_wide);
        }
    }
}
