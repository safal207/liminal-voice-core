use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::device::DeviceProfile;

use crate::dialog;

#[allow(dead_code)]
pub fn record_audio() -> &'static str {
    "recorded.wav"
}

#[allow(dead_code)]
pub fn transcribe_audio(cfg: &Config, prof: &DeviceProfile) -> String {
    transcribe_audio_like(cfg, prof, dialog::default_utterance())
}

pub fn transcribe_audio_like(cfg: &Config, prof: &DeviceProfile, provided: &str) -> String {
    println!(
        "[voice] cfg mode={} sr={} ch={} frame={}ms",
        cfg.mode, cfg.sample_rate, cfg.channels, cfg.frame_ms
    );
    println!("[voice] ASR capturing...");

    let latency_ms = prof.pause_ms + cfg.frame_ms as u64;
    thread::sleep(Duration::from_millis(latency_ms));

    println!("[voice] ASR done (latency={}ms)", latency_ms);
    println!("[voice] transcript: {}", provided);
    provided.to_string()
}

pub fn synthesize_response(cfg: &Config, prof: &DeviceProfile, text: &str) {
    let latency_ms = (prof.pause_ms / 2).saturating_add(cfg.frame_ms as u64);
    println!("[voice] TTS rendering...");
    thread::sleep(Duration::from_millis(latency_ms));
    println!("[voice] TTS done (latency={}ms)", latency_ms);
    println!("[voice] response: {}", text);
    println!(
        "[voice] audio sr={} ch={} gain={:.1}dB",
        cfg.sample_rate, cfg.channels, prof.gain_db
    );
}

pub fn synthesize_with(cfg: &Config, prof: &DeviceProfile, pace: f32, pause_ms: u64, text: &str) {
    let pace = pace.clamp(0.5, 2.0);
    let pause = pause_ms.clamp(20, 250);
    println!(
        "[voice] TTS rendering (pace={:.2}, pause={}ms)...",
        pace, pause
    );

    let base_latency = (pause / 2).saturating_add(cfg.frame_ms as u64);
    let pace_adjust = if pace < 1.0 {
        ((1.0 - pace) * 80.0).round() as u64
    } else {
        ((pace - 1.0) * 40.0).round() as u64
    };
    let latency_ms = base_latency.saturating_add(pace_adjust);

    thread::sleep(Duration::from_millis(latency_ms));
    println!("[voice] TTS done (latency={}ms)", latency_ms);
    println!("[voice] response: {}", text);
    println!(
        "[voice] audio sr={} ch={} gain={:.1}dB",
        cfg.sample_rate, cfg.channels, prof.gain_db
    );
}
