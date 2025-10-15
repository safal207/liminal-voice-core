use std::time::Instant;

#[derive(Debug)]
pub struct VoiceMetrics {
    pub start_ts: Instant,
    pub asr_ms: u128,
    pub tts_ms: u128,
    pub total_ms: u128,
}

pub fn start() -> VoiceMetrics {
    VoiceMetrics {
        start_ts: Instant::now(),
        asr_ms: 0,
        tts_ms: 0,
        total_ms: 0,
    }
}

pub fn finish(vm: &mut VoiceMetrics) {
    vm.total_ms = vm.start_ts.elapsed().as_millis();
}

pub fn print(vm: &VoiceMetrics) {
    println!(
        "[metrics] asr={}ms tts={}ms total={}ms",
        vm.asr_ms, vm.tts_ms, vm.total_ms
    );
}

pub fn clamp01(v: f32) -> f32 {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}
