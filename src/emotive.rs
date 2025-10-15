use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

// Persisted seed of emotional state
#[derive(Clone, Debug, Default)]
pub struct EmoteSeed {
    pub ema_drift: f32, // 0..1
    pub ema_res: f32,   // 0..1
    pub tone: String,   // "Calm" | "Neutral" | "Energetic"
    pub wpm: f32,       // last observed
    pub ts_unix: i64,   // seconds
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EmoteCfg {
    pub path: String,       // default "emote_seed.jsonl"
    pub enable: bool,       // default true
    pub half_life_min: u32, // default 180 (3h)
    pub warm_bias: f32,     // default 0.02
}

impl Default for EmoteCfg {
    fn default() -> Self {
        Self {
            path: "emote_seed.jsonl".to_string(),
            enable: true,
            half_life_min: 180,
            warm_bias: 0.02,
        }
    }
}

pub fn load_latest(path: &str) -> Option<EmoteSeed> {
    let file = OpenOptions::new().read(true).open(path).ok()?;
    let reader = BufReader::new(file);
    let mut lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    while let Some(line) = lines.pop() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(seed) = parse_seed(trimmed) {
            return Some(seed);
        }
    }
    None
}

pub fn save_append(path: &str, seed: &EmoteSeed) -> io::Result<()> {
    let parent = Path::new(path).parent();
    if let Some(dir) = parent {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    let line = format!(
        "{{\"ema_drift\":{:.6},\"ema_res\":{:.6},\"tone\":\"{}\",\"wpm\":{:.3},\"ts\":{}}}\n",
        seed.ema_drift.clamp(0.0, 1.0),
        seed.ema_res.clamp(0.0, 1.0),
        escape_json(&seed.tone),
        seed.wpm,
        seed.ts_unix
    );

    file.write_all(line.as_bytes())
}

pub fn decay(seed: &EmoteSeed, now: i64, half_life_min: u32) -> EmoteSeed {
    let elapsed_secs = now.saturating_sub(seed.ts_unix);
    let elapsed_mins = (elapsed_secs as f32).max(0.0) / 60.0;
    let k = if half_life_min == 0 {
        0.0
    } else {
        let hl = half_life_min as f32;
        0.5_f32.powf((elapsed_mins / hl).max(0.0))
    };

    let ema_drift = lerp(0.30, seed.ema_drift, k);
    let ema_res = lerp(0.70, seed.ema_res, k);
    let wpm = lerp(160.0, seed.wpm, k);
    let tone = if k > 0.3 {
        seed.tone.clone()
    } else {
        "Neutral".to_string()
    };

    EmoteSeed {
        ema_drift,
        ema_res,
        tone,
        wpm,
        ts_unix: seed.ts_unix,
    }
}

pub fn apply_boot_bias(ema_res: &mut f32, warm_bias: f32) {
    *ema_res = (*ema_res + warm_bias).min(1.0);
}

fn parse_seed(line: &str) -> Option<EmoteSeed> {
    let ema_drift = parse_f32_field(line, "\"ema_drift\":")?;
    let ema_res = parse_f32_field(line, "\"ema_res\":")?;
    let tone = parse_string_field(line, "\"tone\":")?;
    let wpm = parse_f32_field(line, "\"wpm\":")?;
    let ts = parse_i64_field(line, "\"ts\":")?;

    Some(EmoteSeed {
        ema_drift,
        ema_res,
        tone,
        wpm,
        ts_unix: ts,
    })
}

fn parse_f32_field(line: &str, key: &str) -> Option<f32> {
    let raw = parse_raw_field(line, key)?;
    raw.parse::<f32>().ok()
}

fn parse_i64_field(line: &str, key: &str) -> Option<i64> {
    let raw = parse_raw_field(line, key)?;
    raw.parse::<i64>().ok()
}

fn parse_raw_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let idx = line.find(key)?;
    let start = idx + key.len();
    let rest = line[start..].trim_start();
    let end = rest
        .find(|c| c == ',' || c == '}')
        .unwrap_or_else(|| rest.len());
    let value = rest[..end].trim();
    if value.is_empty() { None } else { Some(value) }
}

fn parse_string_field(line: &str, key: &str) -> Option<String> {
    let idx = line.find(key)?;
    let start = idx + key.len();
    let rest = line[start..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(unescape_json(&rest[..end]))
}

fn lerp(target: f32, value: f32, k: f32) -> f32 {
    target + (value - target) * k
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn unescape_json(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    other => result.push(other),
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::lerp;

    #[test]
    fn lerp_interpolates() {
        assert!((lerp(0.3, 0.7, 0.0) - 0.3).abs() < 1e-6);
        assert!((lerp(0.3, 0.7, 1.0) - 0.7).abs() < 1e-6);
        assert!((lerp(0.3, 0.7, 0.5) - 0.5).abs() < 1e-6);
    }
}
