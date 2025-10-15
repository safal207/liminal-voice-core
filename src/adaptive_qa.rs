use crate::metrics;
use crate::prosody;
use crate::utils;

pub fn analyze_prompt(input: &str) -> (f32, f32) {
    let (drift, res) = utils::hash01(input);
    (metrics::clamp01(drift), metrics::clamp01(res))
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
