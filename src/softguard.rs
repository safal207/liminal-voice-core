// Minimal soft guard heuristics
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardConfig {
    pub drift_limit: f32,
    pub res_limit: f32,
    pub rephrase_factor: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuardAction {
    None,
    Warn(String),
    Rephrased(String),
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            drift_limit: 0.40,
            res_limit: 0.60,
            rephrase_factor: 0.2,
        }
    }
}

pub fn check_and_rephrase(text: &str, drift: f32, res: f32, cfg: &GuardConfig) -> GuardAction {
    use std::fmt::Write;

    if drift <= cfg.drift_limit && res >= cfg.res_limit {
        return GuardAction::None;
    }

    if drift > cfg.drift_limit && res >= cfg.res_limit {
        let mut msg = String::new();
        write!(
            &mut msg,
            "[soft-guard] high drift {:.2} → adjusting tone",
            drift
        )
        .ok();
        return GuardAction::Warn(msg);
    }

    let mut t = text.trim().to_string();
    if drift > cfg.drift_limit && res < cfg.res_limit {
        t = format!("{} [recentered]", t.replace("!", ".").replace("  ", " "));
        return GuardAction::Rephrased(t);
    }

    GuardAction::None
}
