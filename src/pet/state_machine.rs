/// 状态机
use rand::Rng;
use crate::pet::entity::PetState;
use crate::game::config::GameConfig;

pub struct StateMachine {
    pub current: PetState,
    pub timer: f32,
    walk_duration: f32,
}

pub struct StateTransition {
    pub new_state: Option<PetState>,
    pub should_reverse: bool,
}

impl StateMachine {
    pub fn new(config: &GameConfig) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            current: PetState::Idle,
            timer: rng.gen_range(config.state_change_min_interval..=config.state_change_max_interval),
            walk_duration: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, config: &GameConfig) -> StateTransition {
        self.timer -= dt;

        if self.current == PetState::Walk {
            self.walk_duration += dt;
        } else {
            self.walk_duration = 0.0;
        }

        if self.timer <= 0.0 {
            return self.transition(config);
        }

        StateTransition {
            new_state: None,
            should_reverse: false,
        }
    }

    pub fn force_transition(&mut self, state: PetState, config: &GameConfig) {
        self.current = state;
        self.timer = match state {
            PetState::Eat => 2.0,
            PetState::Happy => 1.5,
            PetState::Startled => 0.8,
            _ => {
                let mut rng = rand::thread_rng();
                rng.gen_range(config.state_change_min_interval..=config.state_change_max_interval)
            }
        };
        self.walk_duration = 0.0;
    }

    fn transition(&mut self, config: &GameConfig) -> StateTransition {
        let mut rng = rand::thread_rng();

        let (new_state, should_reverse) = match self.current {
            PetState::Idle => {
                if rng.gen_bool(0.6) {
                    (PetState::Walk, rng.gen_bool(0.5))
                } else if rng.gen_bool(0.2) {
                    (PetState::Sleep, false)
                } else if rng.gen_bool(0.1) {
                    (PetState::Sit, false)
                } else {
                    (PetState::Idle, false)
                }
            }
            PetState::Walk => {
                if self.walk_duration > 5.0 {
                    (PetState::Idle, false)
                } else if rng.gen_bool(0.3) {
                    (PetState::Idle, false)
                } else {
                    (PetState::Walk, rng.gen_bool(0.3))
                }
            }
            PetState::Sleep => {
                if rng.gen_bool(0.3) {
                    (PetState::Idle, false)
                } else {
                    (PetState::Sleep, false)
                }
            }
            PetState::Sit => {
                if rng.gen_bool(0.4) {
                    (PetState::Idle, false)
                } else {
                    (PetState::Sit, false)
                }
            }
            PetState::Eat => (PetState::Happy, false),
            PetState::Happy => (PetState::Idle, false),
            PetState::Startled => (PetState::Idle, false),
        };

        self.current = new_state;
        self.timer = rng.gen_range(config.state_change_min_interval..=config.state_change_max_interval);
        self.walk_duration = 0.0;

        StateTransition {
            new_state: Some(new_state),
            should_reverse,
        }
    }
}

