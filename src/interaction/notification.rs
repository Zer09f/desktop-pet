/// 通知系统
use rand::Rng;

pub struct NotificationSystem {
    timer: f32,
    interval: f32,
    pub pending_message: Option<String>,
    enabled: bool,
}

impl NotificationSystem {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            timer: 0.0,
            interval: rng.gen_range(300.0..600.0),
            pending_message: None,
            enabled: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.enabled {
            return;
        }

        self.timer += dt;
        if self.timer >= self.interval {
            self.timer = 0.0;
            let mut rng = rand::thread_rng();
            self.interval = rng.gen_range(300.0..600.0);

            let messages = vec![
                "该喝水啦！💧",
                "休息一下眼睛吧~",
                "站起来活动活动！",
                "深呼吸~放松一下",
                "记得保持好心情哦！",
            ];
            let msg = messages[rng.gen_range(0..messages.len())];
            self.pending_message = Some(msg.to_string());
        }
    }

    pub fn consume_message(&mut self) -> Option<String> {
        self.pending_message.take()
    }
}
