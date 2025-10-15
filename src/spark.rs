use crate::metrics;

pub static GLYPHS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub fn sparkline(values: &[f32]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let max_index = (GLYPHS.len() - 1) as f32;
    values
        .iter()
        .map(|v| {
            let clamped = metrics::clamp01(*v);
            let idx = (clamped * max_index).round() as usize;
            let idx = idx.min(GLYPHS.len() - 1);
            GLYPHS[idx]
        })
        .collect::<String>()
}
