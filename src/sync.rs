use crate::stabilizer::EmoState;

#[derive(Clone, Copy, Debug, Default)]
pub struct Baselines {
    pub drift: f32,
    pub res: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Seeds {
    pub pace_bias: f32,
    pub pause_bias_ms: i64,
    pub res_warm: f32,
    pub drift_soft: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Residual {
    pub d_drift: f32,
    pub d_res: f32,
}

pub struct SyncCfg {
    pub lr_fast: f32,
    pub lr_slow: f32,
    pub clamp_step: f32,
}

#[derive(Default)]
pub struct SyncState {
    pub baselines: Baselines,
    pub seeds: Seeds,
    pub accum_drift: f32,
    pub accum_res: f32,
    pub steps: usize,
}

impl SyncState {
    pub fn warm_start(&mut self, seeds: Seeds, base: Baselines) {
        self.seeds = seeds;
        self.baselines = base;
        self.accum_drift = 0.0;
        self.accum_res = 0.0;
        self.steps = 0;
    }

    pub fn step(
        &mut self,
        drift: f32,
        res: f32,
        state: EmoState,
        cfg: &SyncCfg,
    ) -> (f32, i64, f32, f32) {
        let r = Residual {
            d_drift: (drift - self.baselines.drift).max(-1.0).min(1.0),
            d_res: (self.baselines.res - res).max(-1.0).min(1.0),
        };

        self.accum_drift += r.d_drift;
        self.accum_res += r.d_res;
        self.steps += 1;

        let mut pace = -cfg.lr_fast * r.d_drift;
        let mut pause = (cfg.lr_fast * r.d_res * 80.0) as i64;
        let mut res_boost = cfg.lr_fast * r.d_res.max(0.0) * 0.05;
        let mut drift_relief = cfg.lr_fast * (-r.d_drift).max(0.0) * 0.05;

        let c = cfg.clamp_step;
        pace = pace.clamp(-c, c);
        pause = pause.clamp(-20, 40);
        res_boost = res_boost.clamp(0.0, c);
        drift_relief = drift_relief.clamp(0.0, c);

        if matches!(state, EmoState::Overheat) {
            pace -= 0.01;
            pause += 10;
        }

        (pace, pause, res_boost, drift_relief)
    }

    pub fn to_slow_increments(&self, cfg: &SyncCfg) -> (f32, f32) {
        if self.steps == 0 {
            return (0.0, 0.0);
        }
        let mean_drift = self.accum_drift / self.steps as f32;
        let mean_res = self.accum_res / self.steps as f32;
        let drift_bias = (-mean_drift * cfg.lr_slow).clamp(-0.03, 0.03);
        let res_bias = (mean_res * cfg.lr_slow).clamp(-0.03, 0.03);
        (drift_bias, res_bias)
    }
}

pub fn merge_seeds(
    emote_res: f32,
    emote_drift: f32,
    dev_pace: f32,
    dev_pause: i64,
    astro_res: f32,
    astro_drift: f32,
) -> Seeds {
    Seeds {
        pace_bias: dev_pace,
        pause_bias_ms: dev_pause,
        res_warm: (emote_res + astro_res) * 0.5,
        drift_soft: (emote_drift + astro_drift) * 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::{Seeds, merge_seeds};

    #[test]
    fn merge_seeds_blends_inputs() {
        let seeds = merge_seeds(0.8, 0.2, 1.1, 25, 0.6, 0.4);
        assert!((seeds.res_warm - 0.7).abs() < 1e-6);
        assert!((seeds.drift_soft - 0.3).abs() < 1e-6);
        assert!((seeds.pace_bias - 1.1).abs() < f32::EPSILON);
        assert_eq!(seeds.pause_bias_ms, 25);
    }

    #[test]
    fn warm_start_resets_accumulators() {
        let mut state = super::SyncState::default();
        state.accum_drift = 1.0;
        state.accum_res = 2.0;
        state.steps = 3;
        state.warm_start(
            Seeds {
                pace_bias: 1.0,
                pause_bias_ms: 0,
                res_warm: 0.1,
                drift_soft: 0.2,
            },
            super::Baselines {
                drift: 0.5,
                res: 0.5,
            },
        );
        assert_eq!(state.accum_drift, 0.0);
        assert_eq!(state.accum_res, 0.0);
        assert_eq!(state.steps, 0);
    }
}
