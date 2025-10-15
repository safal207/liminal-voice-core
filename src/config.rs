use std::env;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub mode: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub frame_ms: u32,
    pub enable_metrics: bool,
    pub viz_mode: VizMode,
    pub cycles: usize,
    pub enable_logging: bool,
    pub log_dir: String,
    pub script: Option<String>,
    pub inputs_path: Option<String>,
    pub baseline_drift: f32,
    pub baseline_res: f32,
    pub alarm: bool,
    pub strict: bool,
    pub guard: bool,
    pub guard_drift: f32,
    pub guard_res: f32,
    pub guard_factor: f32,
    pub stabilizer: bool,
    pub stab_win: usize,
    pub stab_alpha: f32,
    pub stab_warm: f32,
    pub stab_hot: f32,
    pub stab_low_res: f32,
    pub stab_cool: usize,
    pub stab_calm: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VizMode {
    Compact,
    Full,
}

impl VizMode {
    fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "compact" => Some(VizMode::Compact),
            "full" => Some(VizMode::Full),
            _ => None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: "phone".to_string(),
            sample_rate: 16_000,
            channels: 1,
            frame_ms: 20,
            enable_metrics: true,
            viz_mode: VizMode::Compact,
            cycles: 5,
            enable_logging: false,
            log_dir: "logs".to_string(),
            script: None,
            inputs_path: None,
            baseline_drift: 0.35,
            baseline_res: 0.65,
            alarm: true,
            strict: false,
            guard: true,
            guard_drift: 0.40,
            guard_res: 0.60,
            guard_factor: 0.2,
            stabilizer: true,
            stab_win: 5,
            stab_alpha: 0.4,
            stab_warm: 0.32,
            stab_hot: 0.42,
            stab_low_res: 0.58,
            stab_cool: 3,
            stab_calm: 0.08,
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

fn parse_env_usize(key: &str) -> Option<usize> {
    env::var(key).ok()?.parse().ok()
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

    if let Ok(viz) = env::var("LIMINAL_VIZ_MODE") {
        if let Some(mode) = VizMode::from_str(&viz) {
            cfg.viz_mode = mode;
        }
    }

    if let Some(c) = parse_env_usize("LIMINAL_CYCLES") {
        if c > 0 {
            cfg.cycles = c;
        }
    }

    if let Some(enable_log) = parse_env_bool("LIMINAL_LOG") {
        cfg.enable_logging = enable_log;
    }

    if let Ok(dir) = env::var("LIMINAL_LOG_DIR") {
        if !dir.trim().is_empty() {
            cfg.log_dir = dir;
        }
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
            "--viz" => {
                if let Some(val) = args.next() {
                    if let Some(mode) = VizMode::from_str(&val) {
                        cfg.viz_mode = mode;
                    }
                }
            }
            "--cycles" => {
                if let Some(val) = args.next() {
                    if let Ok(c) = val.parse::<usize>() {
                        if c > 0 {
                            cfg.cycles = c;
                        }
                    }
                }
            }
            "--log" => {
                cfg.enable_logging = true;
            }
            "--log-dir" => {
                if let Some(val) = args.next() {
                    if !val.trim().is_empty() {
                        cfg.log_dir = val;
                    }
                }
            }
            "--script" => {
                if let Some(val) = args.next() {
                    cfg.script = Some(val);
                }
            }
            "--inputs" => {
                if let Some(val) = args.next() {
                    if !val.trim().is_empty() {
                        cfg.inputs_path = Some(val);
                    }
                }
            }
            "--baseline-drift" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.baseline_drift = v;
                    }
                }
            }
            "--baseline-res" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.baseline_res = v;
                    }
                }
            }
            "--alarm" => {
                cfg.alarm = true;
            }
            "--no-alarm" => {
                cfg.alarm = false;
            }
            "--strict" => {
                cfg.strict = true;
            }
            "--guard" => {
                cfg.guard = true;
            }
            "--no-guard" => {
                cfg.guard = false;
            }
            "--guard-drift" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.guard_drift = v;
                    }
                }
            }
            "--guard-res" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.guard_res = v;
                    }
                }
            }
            "--guard-factor" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.guard_factor = v;
                    }
                }
            }
            "--stabilizer" => {
                cfg.stabilizer = true;
            }
            "--no-stabilizer" => {
                cfg.stabilizer = false;
            }
            "--stab-win" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<usize>() {
                        if v > 0 {
                            cfg.stab_win = v;
                        }
                    }
                }
            }
            "--stab-alpha" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.stab_alpha = v;
                    }
                }
            }
            "--stab-warm" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.stab_warm = v;
                    }
                }
            }
            "--stab-hot" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.stab_hot = v;
                    }
                }
            }
            "--stab-lowres" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.stab_low_res = v;
                    }
                }
            }
            "--stab-cool" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<usize>() {
                        if v > 0 {
                            cfg.stab_cool = v;
                        }
                    }
                }
            }
            "--stab-calm" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<f32>() {
                        cfg.stab_calm = v;
                    }
                }
            }
            _ => {}
        }
    }

    cfg
}
