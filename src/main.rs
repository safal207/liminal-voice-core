mod adaptive_qa;
mod alerts;
mod astro;
mod config;
mod device;
mod device_memory;
mod dialog;
mod emotive;
mod metrics;
mod prosody;
mod session;
mod softguard;
mod spark;
mod stabilizer;
mod sync;
mod utils;
mod viz;
mod voice_io;

use std::time::Instant;

use alerts::AlertStats;
use astro::{AstroSessionStats, AstroStore};
use config::VizMode;
use session::SyncDelta;
use softguard::{GuardAction, GuardConfig};
use sync::{Baselines as SyncBaselines, SyncCfg, SyncState};

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

    let astro_theme = astro::normalize_theme(cfg.script.as_deref(), &utterances);

    let mode = device::detect(&cfg.mode);
    cfg.mode = match mode {
        device::DeviceMode::Phone => "phone".to_string(),
        device::DeviceMode::Headset => "headset".to_string(),
        device::DeviceMode::Terminal => "terminal".to_string(),
    };
    let mut prof = device::profile(&mode);
    let mut astro_store = if cfg.astro {
        Some(AstroStore::load(&cfg.astro_path, cfg.astro_cache))
    } else {
        None
    };
    let mut astro_session_stats = AstroSessionStats::default();

    let mut astro_seed_res = 0.0;
    let mut astro_seed_drift = 0.0;
    if cfg.sync {
        if let Some(store) = astro_store.as_ref() {
            if let Some(bias) = store.suggest_sync(&astro_theme) {
                println!(
                    "[astro] warm theme={} drift_bias={:.3} res_bias={:.3} stability={:.2}",
                    astro_theme, bias.drift_bias, bias.res_bias, bias.stability
                );
                astro_seed_res = bias.res_bias.clamp(-0.05, 0.05);
                astro_seed_drift = bias.drift_bias.clamp(-0.05, 0.05);
            }
        }
    }

    let device_key = format!("{:?}", mode);
    let mut mem_store = if cfg.memory {
        device_memory::DeviceMemoryStore::load(&cfg.memory_path)
    } else {
        device_memory::DeviceMemoryStore::default()
    };
    let base_pace = prof.pace_factor;
    let base_pause = prof.pause_ms as f32;
    let mut device_seed_pace = 0.0;
    let mut device_seed_pause: i64 = 0;
    if let Some(memory) = device_memory::suggest_profile(&mem_store, &device_key) {
        println!(
            "[memory] loaded avg_pace={:.2} pause={:.1} art={:.2}",
            memory.avg_pace, memory.avg_pause, memory.avg_articulation
        );
        prof.pace_factor = (prof.pace_factor + memory.avg_pace * 0.1).clamp(0.7, 1.3);
        device_seed_pace = (memory.avg_pace - base_pace).clamp(-0.2, 0.2);
        let pause_bias = (memory.avg_pause - base_pause).round() as i64;
        device_seed_pause = pause_bias.clamp(-40, 60);
    }

    let mut emote_seed_opt: Option<emotive::EmoteSeed> = None;
    let mut emote_seed_display: Option<String> = None;
    let mut emotive_seed_res = 0.0;
    let mut emotive_seed_drift = 0.0;
    if cfg.emote {
        if let Some(seed) = emotive::load_latest(&cfg.emote_path) {
            let mut dec = emotive::decay(&seed, current_unix_secs(), cfg.emote_half_life);
            emotive::apply_boot_bias(&mut dec.ema_res, cfg.emote_warm);
            println!(
                "[emote] seed loaded tone={} ema_drift={:.2} ema_res={:.2} wpm={:.0}",
                dec.tone, dec.ema_drift, dec.ema_res, dec.wpm
            );
            emote_seed_display = Some(format!(
                "tone={} ema_d={:.2} ema_r={:.2} wpm={:.0}",
                dec.tone, dec.ema_drift, dec.ema_res, dec.wpm
            ));
            emotive_seed_res = (dec.ema_res - cfg.baseline_res).max(0.0).min(0.05);
            emotive_seed_drift = (cfg.baseline_drift - dec.ema_drift).max(0.0).min(0.05);
            emote_seed_opt = Some(dec);
        }
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

    let sync_baselines = SyncBaselines {
        drift: cfg.baseline_drift,
        res: cfg.baseline_res,
    };
    let sync_cfg = SyncCfg {
        lr_fast: cfg.sync_lr_fast,
        lr_slow: cfg.sync_lr_slow,
        clamp_step: cfg.sync_step,
    };
    let mut sync_state = SyncState::default();
    if cfg.sync {
        let seeds = sync::merge_seeds(
            emotive_seed_res,
            emotive_seed_drift,
            device_seed_pace,
            device_seed_pause,
            astro_seed_res,
            astro_seed_drift,
        );
        sync_state.warm_start(seeds, sync_baselines);
    }

    let mut drift_history = Vec::with_capacity(cfg.cycles);
    let mut resonance_history = Vec::with_capacity(cfg.cycles);
    let mut last_snapshot: Option<session::Snapshot> = None;
    let mut last_sync_delta: Option<SyncDelta> = None;
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
    let mut last_tone: Option<prosody::ToneTag> = None;
    let mut last_wpm: Option<f32> = None;
    let mut seed_bias_applied = false;

    if let (Some(stab), Some(seed)) = (stabilizer.as_mut(), emote_seed_opt.as_ref()) {
        stab.push(seed.ema_drift, seed.ema_res);
    }

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
        let measured_drift = drift;
        let measured_res = res;

        let mut astro_advice: Option<astro::AstroAdvice> = None;
        let mut astro_key: Option<String> = None;
        let mut astro_recall_ts: Option<i64> = None;
        if cfg.astro {
            astro_key = Some(astro::topic_key(&text, prosody.tone));
        }
        if let (Some(store), Some(ref key)) = (astro_store.as_mut(), astro_key.as_ref()) {
            let now_ts = current_unix_secs();
            if let Some(mut advice) = store.recall(key, now_ts) {
                if let Some(seed) = emote_seed_opt.as_ref() {
                    if idx < 2
                        && seed
                            .tone
                            .eq_ignore_ascii_case(&format!("{:?}", prosody.tone))
                    {
                        let extra = 0.02 + (advice.res_bias.abs().min(0.06) * 0.5);
                        advice.res_bias += extra;
                        advice.drift_bias -= extra * 0.6;
                    }
                }
                drift = metrics::clamp01(drift + advice.drift_bias);
                res = metrics::clamp01(res + advice.res_bias);
                astro_session_stats.hits = astro_session_stats.hits.saturating_add(1);
                astro_session_stats.boost_res += advice.res_bias;
                astro_session_stats.bias_drift += advice.drift_bias;
                astro_recall_ts = Some(now_ts);
                astro_advice = Some(advice);
            }
        }

        let emo_flag = matches!(prosody.tone, prosody::ToneTag::Energetic)
            && (measured_drift > cfg.baseline_drift || measured_res > 0.75);

        let mut articulation = prosody.articulation;
        let mut effective_pace = prof.pace_factor;
        let mut effective_pause_ms = prof.pause_ms as i64;
        let mut stab_state_label: Option<String> = None;
        let mut current_state = stabilizer::EmoState::Normal;

        if !seed_bias_applied {
            if let Some(seed) = emote_seed_opt.as_ref() {
                let pace_bias = (seed.wpm / 160.0).clamp(0.8, 1.2);
                effective_pace = (effective_pace * pace_bias).clamp(0.7, 1.3);
            }
            if cfg.sync {
                effective_pace = (effective_pace + sync_state.seeds.pace_bias).clamp(0.7, 1.3);
                effective_pause_ms =
                    (effective_pause_ms + sync_state.seeds.pause_bias_ms).clamp(20, 250);
                res = metrics::clamp01(res + sync_state.seeds.res_warm);
                drift = metrics::clamp01(drift - sync_state.seeds.drift_soft);
            }
            seed_bias_applied = true;
        }

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
            current_state = stab.state;
        }

        if let Some(mut advice) = astro_advice {
            if let Some(stab) = stabilizer.as_ref() {
                if matches!(stab.state, stabilizer::EmoState::Overheat) {
                    advice.pace_delta -= 0.02;
                    advice.pause_delta_ms += 15;
                }
            }
            effective_pace = (effective_pace + advice.pace_delta).clamp(0.7, 1.3);
            effective_pause_ms = (effective_pause_ms + advice.pause_delta_ms).clamp(20, 250);
        }

        let mut sync_delta: Option<SyncDelta> = None;
        if cfg.sync {
            let (pace_delta, pause_delta_ms, res_boost, drift_relief) =
                sync_state.step(drift, res, current_state, &sync_cfg);
            effective_pace += pace_delta;
            effective_pause_ms += pause_delta_ms;
            res = metrics::clamp01(res + res_boost);
            drift = metrics::clamp01(drift - drift_relief);
            sync_delta = Some(SyncDelta {
                pace_delta,
                pause_delta_ms,
                res_boost,
                drift_relief,
            });
        }

        effective_pause_ms = effective_pause_ms.clamp(20, 250);
        let effective_pause_u64 = effective_pause_ms as u64;
        effective_pace = effective_pace.clamp(0.7, 1.3);

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
            emote_state: if idx + 1 == utterances.len() {
                Some(format!("{:?}", prosody.tone))
            } else {
                None
            },
            sync: if idx + 1 == utterances.len() {
                sync_delta
            } else {
                None
            },
        };

        if idx + 1 == utterances.len() {
            last_sync_delta = snapshot.sync;
        }

        last_articulation = Some(articulation);
        last_drift = Some(drift);
        last_res = Some(res);
        last_tone = Some(prosody.tone);
        last_wpm = Some(prosody.wpm);

        if let Some(sess) = session_handle.as_mut() {
            if let Err(err) = session::write(sess, &snapshot) {
                eprintln!("[log] failed to write snapshot: {}", err);
            }
        }

        if let (Some(store), Some(ref key)) = (astro_store.as_mut(), astro_key.as_ref()) {
            let ts = astro_recall_ts.unwrap_or_else(|| current_unix_secs());
            store.consolidate(key, measured_drift, measured_res, emo_flag, ts);
        }

        last_snapshot = Some(snapshot);

        if let Some(stats) = alert_stats.as_mut() {
            alerts::update(stats, drift, res, cfg.baseline_drift, cfg.baseline_res);
        }
    }

    println!("[viz] resonance  {}", spark::sparkline(&resonance_history));
    println!("[viz] drift      {}", spark::sparkline(&drift_history));

    if cfg.sync {
        if let Some(delta) = last_sync_delta {
            println!(
                "[sync] last_step pace_delta={:.3} pause_delta={} res_boost={:.3} drift_relief={:.3}",
                delta.pace_delta, delta.pause_delta_ms, delta.res_boost, delta.drift_relief
            );
        }
        let (astro_drift_bias, astro_res_bias) = sync_state.to_slow_increments(&sync_cfg);
        if (astro_drift_bias != 0.0 || astro_res_bias != 0.0) && cfg.astro {
            if let Some(store) = astro_store.as_mut() {
                let now_ts = current_unix_secs();
                store.fold_sync_delta(&astro_theme, astro_drift_bias, astro_res_bias, now_ts);
                println!(
                    "[astro] consolidate theme={} drift_bias={:.3} res_bias={:.3}",
                    astro_theme, astro_drift_bias, astro_res_bias
                );
            }
        }
    }

    if cfg.astro {
        println!(
            "[astro] hits={} boost_res={:.3} bias_drift={:.3}",
            astro_session_stats.hits, astro_session_stats.boost_res, astro_session_stats.bias_drift
        );
    }

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
                emote_seed_display.as_deref(),
            );
        }
    }

    if cfg.emote {
        if let (Some(last_wpm), Some(last_tone)) = (last_wpm, last_tone) {
            let (ema_drift, ema_res) = if let Some(stab) = stabilizer.as_ref() {
                (stab.ema_drift, stab.ema_res)
            } else {
                (
                    last_drift.unwrap_or(cfg.baseline_drift),
                    last_res.unwrap_or(cfg.baseline_res),
                )
            };
            let final_tone = format!("{:?}", last_tone);
            let seed = emotive::EmoteSeed {
                ema_drift,
                ema_res,
                tone: final_tone.clone(),
                wpm: last_wpm,
                ts_unix: current_unix_secs(),
            };
            match emotive::save_append(&cfg.emote_path, &seed) {
                Ok(()) => {
                    println!(
                        "[emote] saved tone={} ema_drift={:.2} ema_res={:.2} wpm={:.0}",
                        seed.tone, seed.ema_drift, seed.ema_res, seed.wpm
                    );
                }
                Err(err) => {
                    eprintln!("[emote] failed to save seed: {}", err);
                }
            }
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

fn current_unix_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
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
