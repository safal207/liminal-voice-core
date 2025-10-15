use crate::metrics;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToneTag {
    Neutral,
    Calm,
    Energetic,
}

pub struct Prosody {
    pub wpm: f32,
    pub articulation: f32,
    pub tone: ToneTag,
}

pub fn analyze(text: &str, pace_factor: f32, pause_ms: u64) -> Prosody {
    let words = text
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .count()
        .max(1);
    let _ = words; // reserved for future heuristics

    let base_wpm = 150.0_f32;
    let pause = (pause_ms as f32).max(20.0);
    let raw = (base_wpm * pace_factor * (40.0 / pause)) / 200.0;
    let wpm = metrics::clamp01(raw) * 220.0;

    let articulation = metrics::clamp01((0.85 / pace_factor.max(0.1)) * (pause / 80.0));

    let tone = if wpm < 120.0 {
        ToneTag::Calm
    } else if wpm > 180.0 {
        ToneTag::Energetic
    } else {
        ToneTag::Neutral
    };

    Prosody {
        wpm,
        articulation,
        tone,
    }
}

pub fn apply_articulation_hint(articulation: f32, hint: f32) -> f32 {
    metrics::clamp01(articulation + hint)
}
