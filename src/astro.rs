use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AstroAdvice {
    pub drift_bias: f32,
    pub res_bias: f32,
    pub stability: f32,
    pub visits: u32,
}

#[derive(Debug, Default)]
pub struct AstroStore {
    pub path: String,
    pub data: HashMap<String, AstroAdvice>,
}

impl AstroStore {
    pub fn load(path: &str) -> Self {
        let mut store = Self {
            path: path.to_string(),
            data: HashMap::new(),
        };

        if Path::new(path).exists() {
            if let Ok(txt) = fs::read_to_string(path) {
                for line in txt.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = trimmed.split('|').collect();
                    if parts.len() != 5 {
                        continue;
                    }
                    if let (Ok(drift_bias), Ok(res_bias), Ok(stability), Ok(visits)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                        parts[4].parse::<u32>(),
                    ) {
                        store.data.insert(
                            parts[0].to_string(),
                            AstroAdvice {
                                drift_bias,
                                res_bias,
                                stability: stability.clamp(0.0, 1.0),
                                visits,
                            },
                        );
                    }
                }
            }
        }

        store
    }

    pub fn suggest(&self, theme: &str) -> Option<AstroAdvice> {
        self.data.get(theme).cloned()
    }

    pub fn consolidate(&mut self, theme: &str, drift_delta: f32, res_delta: f32) {
        if drift_delta == 0.0 && res_delta == 0.0 {
            return;
        }

        let entry = self.data.entry(theme.to_string()).or_default();
        entry.visits = entry.visits.saturating_add(1);
        entry.drift_bias = clamp_bias(entry.drift_bias + drift_delta);
        entry.res_bias = clamp_bias(entry.res_bias + res_delta);

        let visits = entry.visits as f32;
        let score = (1.0 - ((drift_delta.abs() + res_delta.abs()) * 0.5)).clamp(0.0, 1.0);
        if visits <= 1.0 {
            entry.stability = score;
        } else {
            entry.stability = ((entry.stability * (visits - 1.0)) + score) / visits;
        }
        entry.stability = entry.stability.clamp(0.0, 1.0);
    }

    pub fn save(&self) {
        if self.path.is_empty() {
            return;
        }

        if let Some(parent) = Path::new(&self.path).parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                eprintln!("[astro] failed to create dir {}: {}", parent.display(), err);
                return;
            }
        }

        let mut out = String::new();
        for (theme, advice) in &self.data {
            out.push_str(&format!(
                "{}|{:.4}|{:.4}|{:.4}|{}\n",
                theme, advice.drift_bias, advice.res_bias, advice.stability, advice.visits
            ));
        }

        if let Err(err) = fs::write(&self.path, out) {
            eprintln!("[astro] failed to write {}: {}", self.path, err);
        }
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

fn clamp_bias(value: f32) -> f32 {
    value.clamp(-0.2, 0.2)
}

#[cfg(test)]
mod tests {
    use super::{AstroStore, clamp_bias, normalize_theme};
    use std::fs;

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
    fn store_roundtrip() {
        let tmp = std::env::temp_dir().join("astro_store_test.txt");
        if tmp.exists() {
            let _ = fs::remove_file(&tmp);
        }

        {
            let mut store = AstroStore::load(tmp.to_str().unwrap());
            store.consolidate("focus", -0.02, 0.03);
            store.save();
        }

        let loaded = AstroStore::load(tmp.to_str().unwrap());
        let advice = loaded.suggest("focus").expect("advice exists");
        assert!(advice.drift_bias < 0.0);
        assert!(advice.res_bias > 0.0);
        assert!(advice.stability >= 0.0 && advice.stability <= 1.0);
    }
}
