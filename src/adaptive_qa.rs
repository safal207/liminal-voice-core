use rand::Rng;

use crate::metrics;

pub fn analyze_prompt(input: &str, pace_factor: f32) -> (f32, f32) {
    let _ = input;
    let mut rng = rand::thread_rng();
    let drift: f32 = metrics::clamp01(rng.gen_range(0.0..1.0));
    let mut res: f32 = rng.gen_range(0.0..1.0);
    let adjustment = (pace_factor - 1.0) * 0.4; // keeps tweaks within ±0.02 for normal ranges
    res = metrics::clamp01(res + adjustment);
    (drift, res)
}
