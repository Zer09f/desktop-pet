/// Pet entity — movement, animation, state machine, image loading
use crate::game::types::Direction;
use crate::renderer::draw::SpriteImage;

// Image loading

pub fn load_sprite(path: &str, target_w: u32, target_h: u32) -> Option<SpriteImage> {
    let img = image::open(path).ok()?;
    let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Nearest);
    let rgba = resized.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut bgra = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let px = rgba[(x as u32, y as u32)];
            let dst = (y * w as usize + x) * 4;
            bgra[dst]     = px[2];
            bgra[dst + 1] = px[1];
            bgra[dst + 2] = px[0];
            bgra[dst + 3] = px[3];
        }
    }
    Some(SpriteImage::from_bgra(w, h, bgra))
}

pub fn load_sprite_sheet(
    path: &str, frames_per_row: u32, rows: u32,
) -> Option<(SpriteImage, Vec<FrameRect>)> {
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    let (sheet_w, sheet_h) = rgba.dimensions();
    let mut bgra = vec![0u8; (sheet_w * sheet_h * 4) as usize];
    let sw = sheet_w as usize;
    let sh = sheet_h as usize;
    for y in 0..sh {
        for x in 0..sw {
            let px = rgba[(x as u32, y as u32)];
            let dst = (y * sw + x) * 4;
            bgra[dst]     = px[2];
            bgra[dst + 1] = px[1];
            bgra[dst + 2] = px[0];
            bgra[dst + 3] = px[3];
        }
    }
    let sheet = SpriteImage::from_bgra(sheet_w, sheet_h, bgra);
    let frame_w = sheet_w / frames_per_row;
    let frame_h = sheet_h / rows;
    let mut frames = Vec::new();
    for row in 0..rows {
        for col in 0..frames_per_row {
            frames.push(FrameRect { x: col * frame_w, y: row * frame_h, w: frame_w, h: frame_h });
        }
    }
    Some((sheet, frames))
}

#[derive(Debug, Clone, Copy)]
pub struct FrameRect {
    pub x: u32, pub y: u32, pub w: u32, pub h: u32,
}

// Sprite sheet rows (0-indexed), 8 rows of 32x32:
//   0 = idle, 1 = walk right, 2 = walk left, 3 = sleep,
//   4 = happy, 5 = scared, 6 = sit, 7 = spare

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetState {
    Idle,
    WalkRight,
    WalkLeft,
    Sleep,
    Happy,
    Scared,
    Sit,
}

impl PetState {
    pub fn row_index(self) -> usize {
        match self {
            PetState::Idle      => 0,
            PetState::WalkRight => 1,
            PetState::WalkLeft  => 2,
            PetState::Sleep     => 3,
            PetState::Happy     => 4,
            PetState::Scared    => 5,
            PetState::Sit       => 6,
        }
    }
    pub fn first_frame(self, frames_per_row: usize) -> usize {
        self.row_index() * frames_per_row
    }
}

pub struct Pet {
    pub pos_x: f32,
    pub pos_y: f32,
    pub direction: Direction,
    pub walk_speed: f32,
    pub screen_w: f32,
    pub pet_w: f32,
    pub pet_h: f32,
    pub affection: u32,
    pub state: PetState,
    pub anim_frame: usize,
    pub frames_per_row: usize,
    anim_timer: f32,
    anim_fps: f32,
    state_timer: f32,
    mouse_hovering: bool,
}

impl Pet {
    pub fn new(
        pet_w: f32, pet_h: f32, walk_speed: f32, anim_fps: f32,
        screen_w: f32, screen_h: f32, frames_per_row: usize,
    ) -> Self {
        let start_x = (screen_w - pet_w) / 2.0;
        let start_y = screen_h - pet_h;
        Self {
            pos_x: start_x, pos_y: start_y,
            direction: Direction::Right,
            walk_speed, screen_w, pet_w, pet_h,
            affection: 0,
            state: PetState::WalkRight,
            anim_frame: 0, frames_per_row,
            anim_timer: 0.0, anim_fps,
            state_timer: 0.0, mouse_hovering: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.anim_timer += dt;
        let interval = 1.0 / self.anim_fps;
        if self.anim_timer >= interval {
            self.anim_timer -= interval;
            self.anim_frame = (self.anim_frame + 1) % self.frames_per_row;
        }

        // Temporary states auto-revert
        if matches!(self.state, PetState::Scared | PetState::Sit) {
            self.state_timer -= dt;
            if self.state_timer <= 0.0 {
                self.state = if self.mouse_hovering {
                    PetState::Idle
                } else {
                    match self.direction {
                        Direction::Right => PetState::WalkRight,
                        Direction::Left  => PetState::WalkLeft,
                    }
                };
            }
            return;
        }

        // Stop at edges or idle
        if matches!(self.state, PetState::Sleep | PetState::Happy | PetState::Idle) {
            return;
        }

        // Walking
        let speed = self.walk_speed * dt;
        match self.direction {
            Direction::Right => {
                self.pos_x += speed;
                if self.pos_x + self.pet_w >= self.screen_w {
                    self.pos_x = self.screen_w - self.pet_w;
                    self.state = PetState::Happy;
                    self.direction = Direction::Left;
                }
            }
            Direction::Left => {
                self.pos_x -= speed;
                if self.pos_x <= 0.0 {
                    self.pos_x = 0.0;
                    self.state = PetState::Sleep;
                    self.direction = Direction::Right;
                }
            }
        }

        if matches!(self.state, PetState::WalkRight | PetState::WalkLeft) {
            self.state = match self.direction {
                Direction::Right => PetState::WalkRight,
                Direction::Left  => PetState::WalkLeft,
            };
        }
    }

    pub fn on_mouse_move(&mut self, _x: f32, _y: f32) {
        self.mouse_hovering = true;
        if matches!(self.state, PetState::WalkRight | PetState::WalkLeft) {
            self.state = PetState::Idle;
        }
    }

    pub fn on_mouse_leave(&mut self) {
        self.mouse_hovering = false;
        if self.state == PetState::Idle {
            self.state = match self.direction {
                Direction::Right => PetState::WalkRight,
                Direction::Left  => PetState::WalkLeft,
            };
        }
    }

    pub fn on_left_click(&mut self) {
        self.state = PetState::Scared;
        self.state_timer = 2.0;
        self.anim_frame = 0;
    }

    pub fn on_right_click(&mut self) {
        self.state = PetState::Sit;
        self.state_timer = 3.0;
        self.anim_frame = 0;
    }

    pub fn current_frame_rect(&self) -> usize {
        self.state.first_frame(self.frames_per_row) + self.anim_frame
    }

    pub fn facing_right(&self) -> bool {
        self.state == PetState::WalkRight
    }
}
