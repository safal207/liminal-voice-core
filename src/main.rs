mod adaptive_qa;
mod alerts;
mod config;
mod device;
mod dialog;
mod metrics;
mod prosody;
mod session;
mod softguard;
mod spark;
mod utils;
mod viz;
mod voice_io;

use std::time::Instant;

use alerts::AlertStats;
use config::VizMode;
use softguard::{GuardAction, GuardConfig};

fn main() {
    let mut cfg = config::from_env_or_args();
    let mut utterances = dialog::load_inputs(&cfg);
    if utterances.len() > cfg.cycles {
        cfg.cycles = utterances.len();
    }

    if cfg.cycles > utterances.len() {
        let default = dialog::default_utterance().to_string();
        let mut padded = Vec::with_capacity(cfg.cycles);
        for idx in 0..cfg.cycles {
            let text = utterances
                .get(idx)
                .cloned()
                .unwrap_or_else(|| default.clone());
            padded.push(text);
        }
        utterances = padded;
    }

    let mode = device::detect(&cfg.mode);
    cfg.mode = match mode {
        device::DeviceMode::Phone => "phone".to_string(),
        device::DeviceMode::Headset => "headset".to_string(),
        device::DeviceMode::Terminal => "terminal".to_string(),
    };
    let prof = device::profile(&mode);

    let mut session_handle = if cfg.enable_logging {
        let mut sess = session::start(cfg.cycles, &cfg.log_dir);
        match session::open_file(&mut sess) {
            Ok(()) => Some(sess),
            Err(err) => {
                eprintln!("[log] failed to open session log: {}", err);
                None
            }
        }
    } else {
        None
    };

    let mut drift_history = Vec::with_capacity(cfg.cycles);
    let mut resonance_history = Vec::with_capacity(cfg.cycles);
    let mut last_snapshot: Option<session::Snapshot> = None;
    let mut alert_stats = if cfg.alarm {
        Some(AlertStats::default())
    } else {
        None
    };

    let guard_cfg = GuardConfig {
        drift_limit: cfg.guard_drift,
        res_limit: cfg.guard_res,
        rephrase_factor: cfg.guard_factor,
    };

    for (idx, utterance) in utterances.iter().enumerate() {
        let mut vm = metrics::start();

        let asr_start = Instant::now();
        let text = voice_io::transcribe_audio_like(&cfg, &prof, utterance);
        vm.asr_ms = asr_start.elapsed().as_millis();

        let prosody = prosody::analyze(&text, prof.pace_factor, prof.pause_ms);
        let (mut drift, mut res) = adaptive_qa::analyze_prompt(&text);
        (drift, res) = adaptive_qa::apply_prosody_bias(drift, res, &prosody.tone);
        drift = metrics::clamp01(drift);
        res = metrics::clamp01(res);

        let mut guard_flag = None;
        if cfg.guard {
            match softguard::check_and_rephrase(&text, drift, res, &guard_cfg) {
                GuardAction::None => {}
                GuardAction::Warn(msg) => {
                    println!("{}", msg);
                    guard_flag = Some("warn".to_string());
                }
                GuardAction::Rephrased(new_text) => {
                    println!("[voice-core] {}", new_text);
                    voice_io::synthesize_response(&cfg, &prof, &new_text);
                    guard_flag = Some("rephrased".to_string());
                }
            }
        }

        let tts_start = Instant::now();
        voice_io::synthesize_response(
            &cfg,
            &prof,
            &format!("Semantic Drift: {:.2}, Resonance: {:.2}", drift, res),
        );
        vm.tts_ms = tts_start.elapsed().as_millis();

        metrics::finish(&mut vm);

        if cfg.enable_metrics {
            metrics::print(&vm);
        }

        drift_history.push(drift);
        resonance_history.push(res);

        let snapshot = session::Snapshot {
            ts: now_rfc3339(),
            device: cfg.mode.clone(),
            drift,
            resonance: res,
            wpm: prosody.wpm,
            articulation: prosody.articulation,
            tone: format!("{:?}", prosody.tone),
            asr_ms: vm.asr_ms,
            tts_ms: vm.tts_ms,
            total_ms: vm.total_ms,
            idx,
            utterance: text.clone(),
            guard: guard_flag.clone(),
        };

        if let Some(sess) = session_handle.as_mut() {
            if let Err(err) = session::write(sess, &snapshot) {
                eprintln!("[log] failed to write snapshot: {}", err);
            }
        }

        last_snapshot = Some(snapshot);

        if let Some(stats) = alert_stats.as_mut() {
            alerts::update(stats, drift, res, cfg.baseline_drift, cfg.baseline_res);
        }
    }

    println!("[viz] resonance  {}", spark::sparkline(&resonance_history));
    println!("[viz] drift      {}", spark::sparkline(&drift_history));

    if let VizMode::Full = cfg.viz_mode {
        if let Some(ref snap) = last_snapshot {
            viz::print_table(
                snap.drift,
                snap.resonance,
                snap.wpm,
                snap.articulation,
                &snap.tone,
                snap.asr_ms,
                snap.tts_ms,
                snap.total_ms,
            );
        }
    }

    let mut strict_exit = false;
    if let Some(ref stats) = alert_stats {
        alerts::print_summary(stats, cfg.baseline_drift, cfg.baseline_res);
        strict_exit = cfg.strict && (stats.drift_breaches > 0 || stats.res_breaches > 0);
    }

    if let Some(sess) = session_handle.take() {
        session::close(sess);
    }

    if strict_exit {
        std::process::exit(2);
    }
}

fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    format_rfc3339(duration.as_secs(), duration.subsec_nanos())
}

fn format_rfc3339(seconds: u64, nanos: u32) -> String {
    const SECONDS_PER_DAY: u64 = 86_400;

    let days = (seconds / SECONDS_PER_DAY) as i64;
    let secs_of_day = (seconds % SECONDS_PER_DAY) as u32;

    let (year, month, day) = civil_from_days(days);

    let hour = secs_of_day / 3_600;
    let minute = (secs_of_day % 3_600) / 60;
    let second = secs_of_day % 60;
    let millis = nanos / 1_000_000;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hour, minute, second, millis
    )
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let mut month = mp + if mp < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    if month <= 0 {
        month += 12;
    }

    (year, month as u32, day as u32)
}
