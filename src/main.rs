mod adaptive_qa;
mod alerts;
mod config;
mod device;
mod device_memory;
mod dialog;
mod metrics;
mod prosody;
mod session;
mod softguard;
mod spark;
mod stabilizer;
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
    let mut prof = device::profile(&mode);

    let device_key = format!("{:?}", mode);
    let mut mem_store = if cfg.memory {
        device_memory::DeviceMemoryStore::load(&cfg.memory_path)
    } else {
        device_memory::DeviceMemoryStore::default()
    };
    if let Some(memory) = device_memory::suggest_profile(&mem_store, &device_key) {
        println!(
            "[memory] loaded avg_pace={:.2} pause={:.1} art={:.2}",
            memory.avg_pace, memory.avg_pause, memory.avg_articulation
        );
        prof.pace_factor = (prof.pace_factor + memory.avg_pace * 0.1).clamp(0.7, 1.3);
    }

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

    let mut stabilizer = if cfg.stabilizer {
        Some(stabilizer::Stabilizer::new(stabilizer::StabilizerCfg {
            win: cfg.stab_win,
            ema_alpha: cfg.stab_alpha,
            warm_drift: cfg.stab_warm,
            hot_drift: cfg.stab_hot,
            low_res: cfg.stab_low_res,
            cool_steps: cfg.stab_cool,
            calm_boost: cfg.stab_calm,
        }))
    } else {
        None
    };

    let mut last_articulation: Option<f32> = None;
    let mut last_drift: Option<f32> = None;
    let mut last_res: Option<f32> = None;

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

        let mut articulation = prosody.articulation;
        let mut effective_pace = prof.pace_factor;
        let mut effective_pause_ms = prof.pause_ms as i64;
        let mut stab_state_label: Option<String> = None;

        if let Some(stab) = stabilizer.as_mut() {
            stab.push(drift, res);
            let advice = stab.advice();
            effective_pace = (prof.pace_factor + advice.pace_delta).clamp(0.7, 1.3);
            effective_pause_ms = (prof.pause_ms as i64 + advice.pause_delta_ms).clamp(20, 250);
            articulation =
                prosody::apply_articulation_hint(prosody.articulation, advice.articulation_hint);
            println!(
                "{}",
                stabilizer::format_status(stab.state, stab.ema_drift, stab.ema_res)
            );
            if let VizMode::Compact = cfg.viz_mode {
                viz::print_compact_stabilizer(stab.state, stab.ema_drift, stab.ema_res);
            }
            stab_state_label = Some(format!("{:?}", stab.state));
        }

        let effective_pause_ms = effective_pause_ms.clamp(20, 250);
        let effective_pause_u64 = effective_pause_ms as u64;
        let effective_pace = effective_pace.clamp(0.7, 1.3);

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
                    if cfg.stabilizer {
                        voice_io::synthesize_with(
                            &cfg,
                            &prof,
                            effective_pace,
                            effective_pause_u64,
                            &new_text,
                        );
                    } else {
                        voice_io::synthesize_response(&cfg, &prof, &new_text);
                    }
                    guard_flag = Some("rephrased".to_string());
                }
            }
        }

        let tts_start = Instant::now();
        if cfg.stabilizer {
            voice_io::synthesize_with(
                &cfg,
                &prof,
                effective_pace,
                effective_pause_u64,
                &format!("Semantic Drift: {:.2}, Resonance: {:.2}", drift, res),
            );
        } else {
            voice_io::synthesize_response(
                &cfg,
                &prof,
                &format!("Semantic Drift: {:.2}, Resonance: {:.2}", drift, res),
            );
        }
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
            articulation,
            tone: format!("{:?}", prosody.tone),
            asr_ms: vm.asr_ms,
            tts_ms: vm.tts_ms,
            total_ms: vm.total_ms,
            idx,
            utterance: text.clone(),
            guard: guard_flag.clone(),
            state: stab_state_label.clone(),
        };

        last_articulation = Some(articulation);
        last_drift = Some(drift);
        last_res = Some(res);

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
            let stab_detail = stabilizer.as_ref().map(|stab| {
                format!(
                    "{:?} (EMA d={:.2} r={:.2})",
                    stab.state, stab.ema_drift, stab.ema_res
                )
            });
            viz::print_table(
                snap.drift,
                snap.resonance,
                snap.wpm,
                snap.articulation,
                &snap.tone,
                snap.asr_ms,
                snap.tts_ms,
                snap.total_ms,
                stab_detail.as_deref(),
            );
        }
    }

    if cfg.memory {
        if let (Some(art), Some(drift), Some(res)) = (last_articulation, last_drift, last_res) {
            mem_store.update(
                &device_key,
                prof.pace_factor,
                prof.pause_ms as f32,
                art,
                drift,
                res,
            );
            mem_store.save();
            println!("[memory] saved updated profile for {:?}", mode);
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
