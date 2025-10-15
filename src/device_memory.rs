use std::{collections::HashMap, fs, path::Path};

#[derive(Clone, Debug, Default)]
pub struct DeviceMemory {
    pub avg_pace: f32,
    pub avg_pause: f32,
    pub avg_articulation: f32,
    pub avg_drift: f32,
    pub avg_res: f32,
    pub sessions: u32,
}

#[derive(Debug)]
pub struct DeviceMemoryStore {
    pub path: String,
    pub data: HashMap<String, DeviceMemory>,
}

impl Default for DeviceMemoryStore {
    fn default() -> Self {
        Self {
            path: String::new(),
            data: HashMap::new(),
        }
    }
}

impl DeviceMemoryStore {
    pub fn load(path: &str) -> Self {
        let mut store = Self {
            path: path.to_string(),
            data: HashMap::new(),
        };

        if Path::new(path).exists() {
            if let Ok(txt) = fs::read_to_string(path) {
                for line in txt.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() == 7 {
                        if let (Ok(pace), Ok(pause), Ok(art), Ok(drift), Ok(res), Ok(sess)) = (
                            parts[1].parse::<f32>(),
                            parts[2].parse::<f32>(),
                            parts[3].parse::<f32>(),
                            parts[4].parse::<f32>(),
                            parts[5].parse::<f32>(),
                            parts[6].parse::<u32>(),
                        ) {
                            store.data.insert(
                                parts[0].to_string(),
                                DeviceMemory {
                                    avg_pace: pace,
                                    avg_pause: pause,
                                    avg_articulation: art,
                                    avg_drift: drift,
                                    avg_res: res,
                                    sessions: sess,
                                },
                            );
                        }
                    }
                }
            }
        }

        store
    }

    pub fn update(&mut self, device: &str, pace: f32, pause: f32, art: f32, drift: f32, res: f32) {
        let entry = self.data.entry(device.to_string()).or_default();
        entry.sessions += 1;
        let n = entry.sessions as f32;
        entry.avg_pace = (entry.avg_pace * (n - 1.0) + pace) / n;
        entry.avg_pause = (entry.avg_pause * (n - 1.0) + pause) / n;
        entry.avg_articulation = (entry.avg_articulation * (n - 1.0) + art) / n;
        entry.avg_drift = (entry.avg_drift * (n - 1.0) + drift) / n;
        entry.avg_res = (entry.avg_res * (n - 1.0) + res) / n;
    }

    pub fn save(&self) {
        let mut out = String::new();
        for (key, value) in &self.data {
            out.push_str(&format!(
                "{}|{:.3}|{:.1}|{:.3}|{:.3}|{:.3}|{}\n",
                key,
                value.avg_pace,
                value.avg_pause,
                value.avg_articulation,
                value.avg_drift,
                value.avg_res,
                value.sessions
            ));
        }
        if !self.path.is_empty() {
            let _ = fs::write(&self.path, out);
        }
    }
}

pub fn suggest_profile(store: &DeviceMemoryStore, device: &str) -> Option<DeviceMemory> {
    store.data.get(device).cloned()
}
