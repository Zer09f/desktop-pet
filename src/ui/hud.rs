/// HUD
use macroquad::prelude::*;
use crate::game::types::AffectionLevel;

pub struct Hud {
    pub visible: bool,
}

impl Hud {
    pub fn new() -> Self {
        Self { visible: true }
    }

    pub fn draw(&self, affection: u32, _state: &str, x: f32, y: f32) {
        if !self.visible {
            return;
        }

        let level = AffectionLevel::from_score(affection);
        let level_text = match level {
            AffectionLevel::Stranger => "陌生人",
            AffectionLevel::Acquaintance => "认识",
            AffectionLevel::Friend => "朋友",
            AffectionLevel::Close => "挚友",
        };

        let status = format!("{} | ♡{}", level_text, affection);
        let font_size = 12.0;
        let dim = measure_text(&status, None, font_size as u16, 1.0);
        let tx = x - dim.width / 2.0;
        let ty = y - 10.0;

        draw_rectangle(
            tx - 4.0,
            ty - font_size - 2.0,
            dim.width + 8.0,
            font_size + 6.0,
            Color::new(0.0, 0.0, 0.0, 0.4),
        );

        draw_text(&status, tx, ty, font_size, Color::new(1.0, 1.0, 0.8, 0.9));
    }
}
