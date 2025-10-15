#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceMode {
    Phone,
    Headset,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceProfile {
    pub gain_db: f32,
    pub pace_factor: f32,
    pub pause_ms: u64,
}

pub fn detect(mode_str: &str) -> DeviceMode {
    match mode_str.to_ascii_lowercase().as_str() {
        "phone" => DeviceMode::Phone,
        "headset" => DeviceMode::Headset,
        "terminal" => DeviceMode::Terminal,
        _ => DeviceMode::Phone,
    }
}

pub fn profile(mode: &DeviceMode) -> DeviceProfile {
    match mode {
        DeviceMode::Phone => DeviceProfile {
            gain_db: -2.0,
            pace_factor: 1.05,
            pause_ms: 60,
        },
        DeviceMode::Headset => DeviceProfile {
            gain_db: 0.0,
            pace_factor: 1.00,
            pause_ms: 40,
        },
        DeviceMode::Terminal => DeviceProfile {
            gain_db: 1.5,
            pace_factor: 0.95,
            pause_ms: 80,
        },
    }
}
