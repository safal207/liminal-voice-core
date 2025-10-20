use std::collections::{HashMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::metrics;
use crate::prosody::ToneTag;
use crate::utils;

const DEFAULT_ALPHA: f32 = 0.22;
const STABILITY_DECAY_PER_DAY: f32 = 0.08;
const STABILITY_THRESHOLD: f32 = 0.18;
const SYNC_CLAMP: f32 = 0.2;

#[derive(Debug, Clone)]
pub struct AstroTrace {
    pub key: String,
    pub ema_drift: f32,
    pub ema_res: f32,
    pub stability: f32,
    pub visits: u32,
    pub last_ts: i64,
    pub emo_tag: bool,
    pub sync_bias_drift: f32,
    pub sync_bias_res: f32,
    pub sync_stability: f32,
    pub sync_visits: u32,
}

impl AstroTrace {
    fn new(key: String, now: i64) -> Self {
        Self {
            key,
            ema_drift: 0.0,
            ema_res: 0.0,
            stability: 0.0,
            visits: 0,
            last_ts: now,
            emo_tag: false,
            sync_bias_drift: 0.0,
            sync_bias_res: 0.0,
            sync_stability: 0.0,
            sync_visits: 0,
        }
    }

    fn decay(&mut self, now: i64) {
        if now <= self.last_ts {
            return;
        }
        let elapsed = now - self.last_ts;
        if elapsed <= 0 {
            return;
        }
        let days = (elapsed as f32 / 86_400.0).min(30.0);
        if days <= 0.0 {
            return;
        }
        let decay = (days * STABILITY_DECAY_PER_DAY).min(self.stability);
        self.stability = (self.stability - decay).max(0.0);
        if self.stability < STABILITY_THRESHOLD {
            self.emo_tag = false;
        }
    }

    fn to_json_line(&self) -> String {
        format!(
            r#"{{"key":"{}","ema_drift":{:.6},"ema_res":{:.6},"stability":{:.6},"visits":{},"last_ts":{},"emo_tag":{},"sync_bias_drift":{:.6},"sync_bias_res":{:.6},"sync_stability":{:.6},"sync_visits":{}}}"#,
            self.key,
            self.ema_drift,
            self.ema_res,
            self.stability,
            self.visits,
            self.last_ts,
            self.emo_tag,
            self.sync_bias_drift,
            self.sync_bias_res,
            self.sync_stability,
            self.sync_visits
        )
    }

    fn from_json_line(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return None;
        }
        let mut trace = AstroTrace::new(String::new(), 0);
        let inner = &trimmed[1..trimmed.len() - 1];
        for part in inner.split(',') {
            let mut kv = part.splitn(2, ':');
            let key = kv.next()?.trim().trim_matches('"');
            let value = kv.next()?.trim();
            match key {
                "key" => {
                    trace.key = value.trim_matches('"').to_string();
                }
                "ema_drift" => trace.ema_drift = value.parse().ok()?,
                "ema_res" => trace.ema_res = value.parse().ok()?,
                "stability" => trace.stability = value.parse().ok()?,
                "visits" => trace.visits = value.parse().ok()?,
                "last_ts" => trace.last_ts = value.parse().ok()?,
                "emo_tag" => trace.emo_tag = matches!(value, "true" | "1"),
                "sync_bias_drift" => trace.sync_bias_drift = value.parse().unwrap_or(0.0),
                "sync_bias_res" => trace.sync_bias_res = value.parse().unwrap_or(0.0),
                "sync_stability" => trace.sync_stability = value.parse().unwrap_or(0.0),
                "sync_visits" => trace.sync_visits = value.parse().unwrap_or(0),
                _ => {}
            }
        }
        if trace.key.is_empty() {
            return None;
        }
        Some(trace)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AstroAdvice {
    pub drift_bias: f32,
    pub res_bias: f32,
    pub pace_delta: f32,
    pub pause_delta_ms: i64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AstroSessionStats {
    pub hits: u32,
    pub boost_res: f32,
    pub bias_drift: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SyncBias {
    pub drift_bias: f32,
    pub res_bias: f32,
    pub stability: f32,
    pub visits: u32,
}

pub struct AstroStore {
    path: PathBuf,
    cache: HashMap<String, AstroTrace>,
    order: VecDeque<String>,
    capacity: usize,
}

impl AstroStore {
    pub fn load(path: &str, capacity: usize) -> Self {
        let mut store = Self {
            path: PathBuf::from(path),
            cache: HashMap::new(),
            order: VecDeque::new(),
            capacity: capacity.max(1),
        };

        if let Ok(file) = fs::File::open(&store.path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if let Some(trace) = AstroTrace::from_json_line(&line) {
                    store.insert_trace(trace);
                }
            }
        }

        store
    }

    fn insert_trace(&mut self, trace: AstroTrace) {
        let key = trace.key.clone();
        self.cache.insert(key.clone(), trace);
        self.promote(&key);
        self.evict_if_needed();
    }

    fn promote(&mut self, key: &str) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
        self.order.push_front(key.to_string());
    }

    fn evict_if_needed(&mut self) {
        while self.order.len() > self.capacity {
            if let Some(old_key) = self.order.pop_back() {
                self.cache.remove(&old_key);
            }
        }
    }

    pub fn recall(&mut self, key: &str, now: i64) -> Option<AstroAdvice> {
        let advice = {
            let trace = self.cache.get_mut(key)?;
            trace.decay(now);
            if trace.stability < STABILITY_THRESHOLD {
                return None;
            }
            trace.last_ts = now;

            let visit_factor = (trace.visits.min(12) as f32) / 12.0;
            let mut intensity = trace.stability * 0.7 + visit_factor * 0.2 + trace.ema_res * 0.1;
            if trace.emo_tag {
                intensity += 0.12;
            }
            intensity = intensity.clamp(0.0, 1.0);

            let drift_bias = -0.02 - 0.04 * intensity;
            let res_bias = 0.02 + 0.04 * intensity;
            let pace_delta = -0.01 - 0.03 * intensity;
            let pause_delta_ms = (10.0 + 30.0 * intensity).round() as i64;

            AstroAdvice {
                drift_bias,
                res_bias,
                pace_delta,
                pause_delta_ms,
            }
        };

        self.promote(key);

        Some(advice)
    }

    pub fn consolidate(&mut self, key: &str, drift: f32, res: f32, emo_tag: bool, now: i64) {
        let mut trace = self
            .cache
            .get(key)
            .cloned()
            .unwrap_or_else(|| AstroTrace::new(key.to_string(), now));

        trace.visits = trace.visits.saturating_add(1);
        if trace.visits == 1 {
            trace.ema_drift = drift;
            trace.ema_res = res;
        } else {
            trace.ema_drift = DEFAULT_ALPHA * drift + (1.0 - DEFAULT_ALPHA) * trace.ema_drift;
            trace.ema_res = DEFAULT_ALPHA * res + (1.0 - DEFAULT_ALPHA) * trace.ema_res;
        }

        trace.ema_drift = metrics::clamp01(trace.ema_drift);
        trace.ema_res = metrics::clamp01(trace.ema_res);

        let mut stability_boost = 0.06;
        if (trace.ema_res - res).abs() < 0.05 {
            stability_boost += 0.01;
        }
        if emo_tag {
            stability_boost += 0.05;
            trace.emo_tag = true;
        } else {
            trace.emo_tag = trace.emo_tag && trace.stability > STABILITY_THRESHOLD;
        }

        trace.stability = (trace.stability + stability_boost).clamp(0.0, 1.0);
        trace.last_ts = now;

        self.persist_trace(trace);
    }

    pub fn fold_sync_delta(&mut self, key: &str, drift_delta: f32, res_delta: f32, now: i64) {
        if drift_delta == 0.0 && res_delta == 0.0 {
            return;
        }

        let mut trace = self
            .cache
            .get(key)
            .cloned()
            .unwrap_or_else(|| AstroTrace::new(key.to_string(), now));

        trace.sync_visits = trace.sync_visits.saturating_add(1);
        trace.sync_bias_drift = clamp_bias(trace.sync_bias_drift + drift_delta);
        trace.sync_bias_res = clamp_bias(trace.sync_bias_res + res_delta);
        let score = (1.0 - ((drift_delta.abs() + res_delta.abs()) * 0.5)).clamp(0.0, 1.0);
        if trace.sync_visits <= 1 {
            trace.sync_stability = score;
        } else {
            let visits = trace.sync_visits as f32;
            trace.sync_stability = ((trace.sync_stability * (visits - 1.0)) + score) / visits;
        }
        trace.sync_stability = trace.sync_stability.clamp(0.0, 1.0);
        trace.last_ts = now;

        self.persist_trace(trace);
    }

    pub fn suggest_sync(&self, theme: &str) -> Option<SyncBias> {
        let trace = self.cache.get(theme)?;
        if trace.sync_visits == 0 {
            return None;
        }
        Some(SyncBias {
            drift_bias: clamp_bias(trace.sync_bias_drift),
            res_bias: clamp_bias(trace.sync_bias_res),
            stability: trace.sync_stability.clamp(0.0, 1.0),
            visits: trace.sync_visits,
        })
    }

    fn persist_trace(&mut self, trace: AstroTrace) {
        let key = trace.key.clone();
        self.cache.insert(key.clone(), trace.clone());
        self.promote(&key);
        self.evict_if_needed();
        if let Err(err) = self.append_trace(&trace) {
            eprintln!("[astro] failed to persist trace: {}", err);
        }
    }

    fn append_trace(&self, trace: &AstroTrace) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", trace.to_json_line())
    }
}

pub fn normalize_theme(script: Option<&str>, utterances: &[String]) -> String {
    if let Some(script) = script {
        let trimmed = script.trim();
        if !trimmed.is_empty() {
            return trimmed.to_ascii_lowercase();
        }
    }

    utterances
        .first()
        .map(|line| line.trim().to_ascii_lowercase())
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| "default".to_string())
}

pub fn topic_key(text: &str, tone: ToneTag) -> String {
    let normalized = utils::normalize_text(text);
    let collapsed = normalized
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let seed = format!(
        "{}|{}",
        collapsed,
        format!("{:?}", tone).to_ascii_lowercase()
    );
    let (a, b) = utils::hash01(&seed);
    let a_val = (a * 1_048_575.0).round() as u32;
    let b_val = (b * 1_048_575.0).round() as u32;
    format!("astro-{:05x}{:05x}", a_val, b_val)
}

fn clamp_bias(value: f32) -> f32 {
    value.clamp(-SYNC_CLAMP, SYNC_CLAMP)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};

    #[test]
    fn topic_key_deterministic() {
        let text = "Memory Drift and Resonance";
        let key1 = topic_key(text, ToneTag::Calm);
        let key2 = topic_key(text, ToneTag::Calm);
        assert_eq!(key1, key2);
        let energetic = topic_key(text, ToneTag::Energetic);
        assert_ne!(key1, energetic);
    }

    #[test]
    fn trace_roundtrip_json() {
        let trace = AstroTrace {
            key: "astro-001".to_string(),
            ema_drift: 0.31,
            ema_res: 0.77,
            stability: 0.42,
            visits: 3,
            last_ts: 42,
            emo_tag: true,
            sync_bias_drift: -0.04,
            sync_bias_res: 0.06,
            sync_stability: 0.5,
            sync_visits: 2,
        };
        let line = trace.to_json_line();
        let parsed = AstroTrace::from_json_line(&line).expect("parsed");
        assert_eq!(parsed.key, trace.key);
        assert!((parsed.ema_drift - trace.ema_drift).abs() < 1e-6);
        assert!((parsed.sync_bias_res - trace.sync_bias_res).abs() < 1e-6);
        assert_eq!(parsed.sync_visits, trace.sync_visits);
        assert!(parsed.emo_tag);
    }

    #[test]
    fn store_persists_and_recalls() {
        let mut path = env::temp_dir();
        path.push("astro-store-test.jsonl");
        let _ = File::create(&path);

        let mut store = AstroStore::load(path.to_str().unwrap(), 4);
        let key = "astro-test-key";
        store.consolidate(key, 0.4, 0.7, false, 100);
        store.consolidate(key, 0.35, 0.75, true, 120);

        let advice = store.recall(key, 130).expect("advice");
        assert!(advice.res_bias >= 0.02);
        assert!(advice.drift_bias <= -0.02);

        drop(store);

        let mut store2 = AstroStore::load(path.to_str().unwrap(), 4);
        let advice2 = store2.recall(key, 140).expect("advice");
        assert!(advice2.pause_delta_ms >= 10);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn store_lru_evicts() {
        let mut path = env::temp_dir();
        path.push("astro-lru-test.jsonl");
        let _ = fs::remove_file(&path);
        let mut store = AstroStore::load(path.to_str().unwrap(), 2);
        store.consolidate("a", 0.4, 0.6, false, 1);
        store.consolidate("b", 0.3, 0.7, true, 2);
        store.consolidate("b", 0.32, 0.72, true, 3);
        store.consolidate("c", 0.2, 0.8, false, 4);
        assert!(store.recall("a", 5).is_none());
        assert!(store.recall("b", 5).is_some());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn normalize_prefers_script() {
        let key = normalize_theme(Some("Focus;Calm"), &["hello".into()]);
        assert_eq!(key, "focus;calm");
    }

    #[test]
    fn normalize_falls_back_to_utterance() {
        let key = normalize_theme(None, &["Reflect".into()]);
        assert_eq!(key, "reflect");
    }

    #[test]
    fn clamp_bias_limits_range() {
        assert_eq!(clamp_bias(0.5), 0.2);
        assert_eq!(clamp_bias(-0.5), -0.2);
        assert!((clamp_bias(0.05) - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn sync_bias_roundtrip() {
        let mut path = env::temp_dir();
        path.push("astro-sync-test.jsonl");
        let _ = fs::remove_file(&path);

        let mut store = AstroStore::load(path.to_str().unwrap(), 4);
        store.fold_sync_delta("focus", -0.02, 0.015, 10);
        store.fold_sync_delta("focus", -0.03, 0.02, 20);

        let reloaded = AstroStore::load(path.to_str().unwrap(), 4);
        let bias = reloaded.suggest_sync("focus").expect("bias");
        assert!(bias.drift_bias <= 0.0);
        assert!(bias.res_bias >= 0.0);
        assert!(bias.visits >= 1);
        assert!(bias.stability >= 0.0 && bias.stability <= 1.0);

        let _ = fs::remove_file(&path);
    }
}
