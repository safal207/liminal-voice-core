mod adaptive_qa;
mod config;
mod device;
mod metrics;
mod prosody;
mod viz;
mod voice_io;

use std::time::Instant;

fn main() {
    let mut cfg = config::from_env_or_args();
    let mode = device::detect(&cfg.mode);
    cfg.mode = match mode {
        device::DeviceMode::Phone => "phone".to_string(),
        device::DeviceMode::Headset => "headset".to_string(),
        device::DeviceMode::Terminal => "terminal".to_string(),
    };
    let prof = device::profile(&mode);

    let mut vm = metrics::start();

    let asr_start = Instant::now();
    let text = voice_io::transcribe_audio(&cfg, &prof);
    vm.asr_ms = asr_start.elapsed().as_millis();

    let p = prosody::analyze(&text, prof.pace_factor, prof.pause_ms);
    let (mut drift, mut res) = adaptive_qa::analyze_prompt(&text);
    (drift, res) = adaptive_qa::apply_prosody_bias(drift, res, &p.tone);

    let tts_start = Instant::now();
    voice_io::synthesize_response(
        &cfg,
        &prof,
        &format!("Semantic Drift: {:.2}, Resonance: {:.2}", drift, res),
    );
    vm.tts_ms = tts_start.elapsed().as_millis();

    metrics::finish(&mut vm);

    viz::print_table(
        drift,
        res,
        p.wpm,
        p.articulation,
        &format!("{:?}", p.tone),
        vm.asr_ms,
        vm.tts_ms,
        vm.total_ms,
    );

    if cfg.enable_metrics {
        metrics::print(&vm);
    }
}
