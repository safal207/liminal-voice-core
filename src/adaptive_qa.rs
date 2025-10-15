use rand::Rng;

use crate::metrics;
use crate::prosody;

pub fn analyze_prompt(input: &str) -> (f32, f32) {
    let _ = input;
    let mut rng = rand::thread_rng();
    let drift: f32 = metrics::clamp01(rng.gen_range(0.0..1.0));
    let res: f32 = metrics::clamp01(rng.gen_range(0.0..1.0));
    (drift, res)
}

pub fn apply_prosody_bias(mut drift: f32, mut res: f32, tone: &prosody::ToneTag) -> (f32, f32) {
    match tone {
        prosody::ToneTag::Calm => {
            res += 0.02;
        }
        prosody::ToneTag::Energetic => {
            res -= 0.01;
            drift += 0.02;
        }
        prosody::ToneTag::Neutral => {}
    }

    drift = metrics::clamp01(drift);
    res = metrics::clamp01(res);

    (drift, res)
}
