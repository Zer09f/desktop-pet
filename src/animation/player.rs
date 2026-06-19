/// 动画播放器
/// 管理精灵表帧动画
use macroquad::prelude::*;
use crate::game::config::SpriteConfig;

/// 动画播放器
pub struct AnimationPlayer {
    /// 当前动画行（对应状态）
    pub current_row: usize,
    /// 当前帧索引
    pub current_frame: usize,
    /// 帧计时器
    frame_timer: f32,
    /// 每帧持续时间
    frame_duration: f32,
    /// 总帧数
    total_frames: usize,
    /// 是否正在播放
    pub playing: bool,
    /// 动画完成一次循环后是否暂停
    pub pause_on_complete: bool,
    /// 是否已完成一次循环
    pub completed_once: bool,
}

impl AnimationPlayer {
    pub fn new(config: &SpriteConfig) -> Self {
        Self {
            current_row: 0,
            current_frame: 0,
            frame_timer: 0.0,
            frame_duration: 1.0 / config.anim_fps,
            total_frames: config.frames_per_row as usize,
            playing: true,
            pause_on_complete: false,
            completed_once: false,
        }
    }

    /// 切换到新的动画行
    pub fn switch_to(&mut self, row: usize, frame_count: usize, pause_on_complete: bool) {
        if self.current_row != row || !self.playing {
            self.current_row = row;
            self.current_frame = 0;
            self.frame_timer = 0.0;
            self.total_frames = frame_count;
            self.playing = true;
            self.pause_on_complete = pause_on_complete;
            self.completed_once = false;
        }
    }

    /// 每帧更新
    pub fn update(&mut self, dt: f32) {
        if !self.playing {
            return;
        }

        self.frame_timer += dt;
        if self.frame_timer >= self.frame_duration {
            self.frame_timer -= self.frame_duration;
            self.current_frame += 1;

            if self.current_frame >= self.total_frames {
                if self.pause_on_complete {
                    self.current_frame = self.total_frames - 1;
                    self.playing = false;
                    self.completed_once = true;
                } else {
                    self.current_frame = 0;
                }
            }
        }
    }

    /// 获取当前帧在精灵表中的源矩形
    pub fn source_rect(&self, config: &SpriteConfig) -> Rect {
        Rect::new(
            (self.current_frame as u32 * config.frame_width) as f32,
            (self.current_row as u32 * config.frame_height) as f32,
            config.frame_width as f32,
            config.frame_height as f32,
        )
    }
}
