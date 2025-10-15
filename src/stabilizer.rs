use std::cmp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmoState {
    Normal,
    Warming,
    Overheat,
    Cooldown,
}

#[derive(Debug, Clone, Copy)]
pub struct StabilizerCfg {
    pub win: usize,
    pub ema_alpha: f32,
    pub warm_drift: f32,
    pub hot_drift: f32,
    pub low_res: f32,
    pub cool_steps: usize,
    pub calm_boost: f32,
}

#[derive(Debug, Clone)]
pub struct Stabilizer {
    pub cfg: StabilizerCfg,
    pub state: EmoState,
    pub steps_in_state: usize,
    pub ema_drift: f32,
    pub ema_res: f32,
    ring_drift: Vec<f32>,
    ring_res: Vec<f32>,
    idx: usize,
    initialized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Advice {
    pub pace_delta: f32,
    pub pause_delta_ms: i64,
    pub articulation_hint: f32,
}

impl Stabilizer {
    pub fn new(mut cfg: StabilizerCfg) -> Self {
        cfg.win = cfg.win.max(1);
        cfg.ema_alpha = cfg.ema_alpha.clamp(0.0, 1.0);
        cfg.warm_drift = cfg.warm_drift.clamp(0.0, 1.0);
        cfg.hot_drift = cfg.hot_drift.clamp(0.0, 1.0);
        cfg.low_res = cfg.low_res.clamp(0.0, 1.0);
        cfg.cool_steps = cfg.cool_steps.max(1);
        cfg.calm_boost = cfg.calm_boost.clamp(0.0, 0.2);

        Self {
            ring_drift: vec![0.0; cfg.win],
            ring_res: vec![0.0; cfg.win],
            idx: 0,
            initialized: false,
            cfg,
            state: EmoState::Normal,
            steps_in_state: 0,
            ema_drift: 0.0,
            ema_res: 0.0,
        }
    }

    pub fn push(&mut self, drift: f32, res: f32) {
        if self.ring_drift.is_empty() {
            return;
        }

        let drift = drift.clamp(0.0, 1.0);
        let res = res.clamp(0.0, 1.0);

        self.ring_drift[self.idx] = drift;
        self.ring_res[self.idx] = res;
        self.idx = (self.idx + 1) % self.ring_drift.len();
        if !self.initialized {
            self.ema_drift = drift;
            self.ema_res = res;
            self.initialized = true;
        } else {
            let alpha = self.cfg.ema_alpha;
            self.ema_drift = alpha * drift + (1.0 - alpha) * self.ema_drift;
            self.ema_res = alpha * res + (1.0 - alpha) * self.ema_res;
        }

        self.ema_drift = self.ema_drift.clamp(0.0, 1.0);
        self.ema_res = self.ema_res.clamp(0.0, 1.0);

        let next_state = if drift >= self.cfg.hot_drift && res <= self.cfg.low_res {
            EmoState::Overheat
        } else if drift >= self.cfg.warm_drift {
            EmoState::Warming
        } else {
            match self.state {
                EmoState::Overheat => {
                    if self.steps_in_state + 1 < self.cfg.cool_steps {
                        EmoState::Cooldown
                    } else {
                        EmoState::Normal
                    }
                }
                EmoState::Cooldown => {
                    if self.steps_in_state + 1 >= self.cfg.cool_steps {
                        EmoState::Normal
                    } else {
                        EmoState::Cooldown
                    }
                }
                _ => EmoState::Normal,
            }
        };

        if next_state != self.state {
            self.state = next_state;
            self.steps_in_state = 0;
        } else {
            self.steps_in_state = cmp::min(
                self.steps_in_state + 1,
                self.cfg.cool_steps.saturating_mul(2),
            );
        }
    }

    pub fn advice(&self) -> Advice {
        match self.state {
            EmoState::Normal => Advice {
                pace_delta: 0.0,
                pause_delta_ms: 0,
                articulation_hint: 0.0,
            },
            EmoState::Warming => Advice {
                pace_delta: -0.03,
                pause_delta_ms: 10,
                articulation_hint: 0.02,
            },
            EmoState::Overheat => Advice {
                pace_delta: -0.07 - self.cfg.calm_boost,
                pause_delta_ms: 30 + (self.cfg.calm_boost * 100.0).round() as i64,
                articulation_hint: 0.05,
            },
            EmoState::Cooldown => Advice {
                pace_delta: -0.04,
                pause_delta_ms: 20,
                articulation_hint: 0.03,
            },
        }
    }
}

pub fn format_status(state: EmoState, ema_drift: f32, ema_res: f32) -> String {
    format!(
        "[stabilizer] state={:?} ema_drift={:.2} ema_res={:.2}",
        state,
        ema_drift.clamp(0.0, 1.0),
        ema_res.clamp(0.0, 1.0)
    )
}
