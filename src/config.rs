use std::env;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub mode: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub frame_ms: u32,
    pub enable_metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: "phone".to_string(),
            sample_rate: 16_000,
            channels: 1,
            frame_ms: 20,
            enable_metrics: true,
        }
    }
}

fn parse_env_u32(key: &str) -> Option<u32> {
    env::var(key).ok()?.parse().ok()
}

fn parse_env_u16(key: &str) -> Option<u16> {
    env::var(key).ok()?.parse().ok()
}

fn parse_env_bool(key: &str) -> Option<bool> {
    env::var(key)
        .ok()
        .and_then(|v| match v.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
}

pub fn from_env_or_args() -> Config {
    let mut cfg = Config::default();

    if let Ok(mode) = env::var("LIMINAL_MODE") {
        if !mode.trim().is_empty() {
            cfg.mode = mode.to_ascii_lowercase();
        }
    }

    if let Some(sr) = parse_env_u32("LIMINAL_SAMPLE_RATE") {
        cfg.sample_rate = sr;
    }

    if let Some(ch) = parse_env_u16("LIMINAL_CHANNELS") {
        cfg.channels = ch;
    }

    if let Some(frame) = parse_env_u32("LIMINAL_FRAME_MS") {
        cfg.frame_ms = frame;
    }

    if let Some(enable) = parse_env_bool("LIMINAL_ENABLE_METRICS") {
        cfg.enable_metrics = enable;
    }

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mode" => {
                if let Some(val) = args.next() {
                    cfg.mode = val.to_ascii_lowercase();
                }
            }
            "--sample-rate" => {
                if let Some(val) = args.next() {
                    if let Ok(sr) = val.parse() {
                        cfg.sample_rate = sr;
                    }
                }
            }
            "--channels" => {
                if let Some(val) = args.next() {
                    if let Ok(channels) = val.parse() {
                        cfg.channels = channels;
                    }
                }
            }
            "--frame-ms" => {
                if let Some(val) = args.next() {
                    if let Ok(frame) = val.parse() {
                        cfg.frame_ms = frame;
                    }
                }
            }
            "--no-metrics" => {
                cfg.enable_metrics = false;
            }
            _ => {}
        }
    }

    cfg
}
