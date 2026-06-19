/// Application — owns all state, called from WndProc and main loop
use crate::game::config::PetConfig;
use crate::game::save_data::SaveData;
use crate::pet::entity::{self, Pet, FrameRect, load_sprite};
use crate::renderer::draw::{Renderer, SpriteImage};
use crate::interaction::bubble::Bubble;
use crate::input::handler::*;

pub struct App {
    pub pet: Pet,
    pub renderer: Renderer,
    pub config: PetConfig,
    pub save_data: SaveData,
    pub bubble: Bubble,
    sprite: Option<SpriteImage>,
    sheet: Option<SpriteImage>,
    frames: Vec<FrameRect>,
    is_sheet: bool,
    save_timer: f32,
    bubble_timer: f32,
    pub running: bool,
}

impl App {
    pub fn new(hwnd: windows::Win32::Foundation::HWND, screen_w: f32, screen_h: f32) -> Option<Self> {
        let config = PetConfig::load();
        let save_data = SaveData::load();

        let pet_w = config.pet_width as f32;
        let pet_h = config.pet_height as f32;

        let renderer = Renderer::new(hwnd, config.pet_width as i32, config.pet_height as i32)?;
        let pet = Pet::new(
            pet_w, pet_h, config.walk_speed, config.anim_fps,
            screen_w, screen_h, config.frames_per_row as usize,
        );

        let (sprite, sheet, frames, is_sheet) = Self::load_image(&config);

        let mut app = Self {
            pet, renderer, config, save_data,
            bubble: Bubble::new(),
            sprite, sheet, frames, is_sheet,
            save_timer: 0.0, bubble_timer: 0.0,
            running: true,
        };

        app.pet.affection = app.save_data.affection;
        Some(app)
    }

    fn load_image(config: &PetConfig) -> (Option<SpriteImage>, Option<SpriteImage>, Vec<FrameRect>, bool) {
        if config.frames_per_row > 0 && config.rows > 0 {
            if let Some((sheet, frames)) = entity::load_sprite_sheet(
                &config.image_path, config.frames_per_row, config.rows,
            ) {
                return (None, Some(sheet), frames, true);
            }
        }
        match load_sprite(&config.image_path, config.pet_width, config.pet_height) {
            Some(sprite) => (Some(sprite), None, vec![], false),
            None => (None, None, vec![], false),
        }
    }

    pub fn reload_image(&mut self) {
        let (sprite, sheet, frames, is_sheet) = Self::load_image(&self.config);
        self.sprite = sprite;
        self.sheet = sheet;
        self.frames = frames;
        self.is_sheet = is_sheet;
    }

    pub fn update(&mut self, dt: f32) {
        self.pet.update(dt);
        self.bubble.update(dt);

        self.save_timer += dt;
        if self.save_timer >= 60.0 {
            self.save_timer = 0.0;
            self.save_data.affection = self.pet.affection;
            let _ = self.save_data.save();
        }

        self.bubble_timer += dt;
        if self.bubble_timer >= 12.0 && !self.bubble.visible {
            self.bubble_timer = 0.0;
            self.trigger_random_bubble();
        }
    }

    pub fn render(&mut self) {
        if self.is_sheet {
            if let Some(ref sheet) = self.sheet {
                if !self.frames.is_empty() {
                    let frame_idx = self.pet.current_frame_rect();
                    let frame_idx = frame_idx % self.frames.len();
                    let f = self.frames[frame_idx];
                    self.renderer.prepare_frame(sheet, f.x, f.y, f.w, f.h, self.pet.facing_right());
                }
            }
        } else if let Some(ref sprite) = self.sprite {
            self.renderer.prepare_sprite(sprite);
        }

        let center_x = self.renderer.width() / 2;
        self.bubble.draw_to_hdc(
            self.renderer.hdc(),
            center_x,
            self.renderer.width(),
            self.renderer.height(),
        );

        self.renderer.update_window();
    }

    pub fn on_left_click(&mut self, _x: f32, _y: f32) {
        self.pet.on_left_click();
        self.bubble.show("\u{1f631}!".into(), 2.0);
        self.save_data.affection = self.pet.affection;
        let _ = self.save_data.save();
    }

    pub fn on_right_click(&mut self, x: f32, y: f32, hwnd: windows::Win32::Foundation::HWND) {
        use windows::Win32::Graphics::Gdi::*;
        let mut pt = windows::Win32::Foundation::POINT { x: x as i32, y: y as i32 };
        unsafe { let _ = ClientToScreen(hwnd, &mut pt); }
        self.pet.on_right_click();
        crate::interaction::menu::show_context_menu(hwnd, pt.x, pt.y);
    }

    pub fn on_mouse_move(&mut self, x: f32, y: f32) {
        self.pet.on_mouse_move(x, y);
    }

    pub fn on_mouse_leave(&mut self) {
        self.pet.on_mouse_leave();
    }

    pub fn on_menu_action(&mut self, menu_id: u32) {
        match menu_id {
            MENU_CHANGE_SKIN => {
                if let Some(path) = Self::pick_skin_file() {
                    let dest = std::env::current_dir()
                        .unwrap_or_default().join("assets").join("pet.png");
                    if std::fs::copy(&path, &dest).is_ok() {
                        self.config.image_path = dest.to_string_lossy().to_string();
                        let _ = self.config.save();
                        self.reload_image();
                        self.bubble.show("\u{1f457}".into(), 3.0);
                    }
                }
            }
            MENU_FEED => {
                self.pet.affection = self.pet.affection.saturating_add(self.config.feed_affection_gain);
                self.save_data.affection = self.pet.affection;
                self.save_data.total_feeds += 1;
                let _ = self.save_data.save();
                self.bubble.show("\u{1f356} yummy~".into(), 3.0);
            }
            MENU_PET => {
                self.pet.affection = self.pet.affection.saturating_add(2);
                self.save_data.affection = self.pet.affection;
                let _ = self.save_data.save();
                self.bubble.show("\u{1f60a}".into(), 3.0);
            }
            MENU_PLAY => {
                self.bubble.show("\u{1f3ae} whee~".into(), 3.0);
            }
            MENU_REMOVE_BG => {
                self.remove_white_background();
            }
            MENU_QUIT => {
                self.running = false;
            }
            _ => {}
        }
    }

    /// Remove white/near-white background from the pet image and save as transparent PNG
    fn remove_white_background(&mut self) {
        let path = std::env::current_dir()
            .unwrap_or_default().join(&self.config.image_path);
        if !path.exists() {
            self.bubble.show("no image!".into(), 2.0);
            return;
        }
        match image::open(&path) {
            Ok(img) => {
                let mut rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let threshold = 230u8; // pixels brighter than this become transparent
                let soft_range = 40u8; // gradual fade range below threshold

                for y in 0..h {
                    for x in 0..w {
                        let px = rgba.get_pixel_mut(x, y);
                        let r = px[0];
                        let g = px[1];
                        let b = px[2];
                        // Check if pixel is white/near-white
                        let min_ch = r.min(g).min(b);
                        if min_ch >= threshold {
                            px[3] = 0; // fully transparent
                        } else if min_ch >= threshold - soft_range {
                            // Soft edge: fade alpha gradually
                            let fade = ((min_ch as u32 - (threshold - soft_range) as u32) * 255 / soft_range as u32) as u8;
                            px[3] = px[3].min(255 - fade);
                        }
                    }
                }

                // Save backup then overwrite
                let backup = path.with_extension("png.bak");
                let _ = std::fs::copy(&path, &backup);
                match rgba.save(&path) {
                    Ok(_) => {
                        self.config.image_path = path.to_string_lossy().to_string();
                        let _ = self.config.save();
                        self.reload_image();
                        self.bubble.show("\u{2728} done!".into(), 3.0);
                    }
                    Err(e) => {
                        self.bubble.show(format!("error: {}", e), 3.0);
                    }
                }
            }
            Err(e) => {
                self.bubble.show(format!("load error: {}", e), 3.0);
            }
        }
    }

    fn pick_skin_file() -> Option<std::path::PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose pet image")
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
            .pick_file()
    }

    fn trigger_random_bubble(&mut self) {
        use rand::Rng;
        let texts: Vec<&str> = if self.pet.affection > 50 {
            vec!["\u{2665}", "hi~", "pat me!", "hehe", "I like you!"]
        } else {
            vec!["...", "hungry", "bored", "hmm?", "zzz"]
        };
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..texts.len());
        self.bubble.show(texts[idx].into(), 3.0);
    }
}


