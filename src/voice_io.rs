use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::device::DeviceProfile;

#[allow(dead_code)]
pub fn record_audio() -> &'static str {
    "recorded.wav"
}

pub fn transcribe_audio(cfg: &Config, prof: &DeviceProfile) -> String {
    println!(
        "[cfg] mode={} sr={} ch={} frame={}ms",
        cfg.mode, cfg.sample_rate, cfg.channels, cfg.frame_ms
    );
    println!("[voice] ASR capturing...");

    let latency_ms = prof.pause_ms + cfg.frame_ms as u64;
    thread::sleep(Duration::from_millis(latency_ms));

    println!("[voice] ASR done (latency={}ms)", latency_ms);
    "hello liminal".to_string()
}

pub fn synthesize_response(cfg: &Config, prof: &DeviceProfile, text: &str) {
    let latency_ms = (prof.pause_ms / 2).saturating_add(cfg.frame_ms as u64);
    println!("[voice] TTS rendering...");
    thread::sleep(Duration::from_millis(latency_ms));
    println!("[voice] TTS done (latency={}ms)", latency_ms);
    println!("→ [voice]: {}", text);
    println!(
        "[voice] audio sr={} ch={} gain={:.1}dB",
        cfg.sample_rate, cfg.channels, prof.gain_db
    );
}
