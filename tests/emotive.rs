use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use liminal_voice_core::emotive::{self, EmoteSeed};

fn approx_eq(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-4, "{} != {}", a, b);
}

#[test]
fn decay_no_elapsed_time_preserves_seed() {
    let seed = EmoteSeed {
        ema_drift: 0.41,
        ema_res: 0.63,
        tone: "Calm".to_string(),
        wpm: 152.0,
        ts_unix: 1_000,
    };
    let decayed = emotive::decay(&seed, seed.ts_unix, 180);
    approx_eq(decayed.ema_drift, seed.ema_drift);
    approx_eq(decayed.ema_res, seed.ema_res);
    approx_eq(decayed.wpm, seed.wpm);
    assert_eq!(decayed.tone, seed.tone);
}

#[test]
fn decay_large_elapsed_time_trends_to_neutral() {
    let seed = EmoteSeed {
        ema_drift: 0.65,
        ema_res: 0.45,
        tone: "Energetic".to_string(),
        wpm: 210.0,
        ts_unix: 2_000,
    };
    let now = seed.ts_unix + 60 * 600; // 600 minutes later
    let decayed = emotive::decay(&seed, now, 30);
    approx_eq(decayed.ema_drift, 0.30);
    approx_eq(decayed.ema_res, 0.70);
    approx_eq(decayed.wpm, 160.0);
    assert_eq!(decayed.tone, "Neutral");
}

#[test]
fn boot_bias_increases_resonance() {
    let mut ema_res = 0.70;
    emotive::apply_boot_bias(&mut ema_res, 0.02);
    approx_eq(ema_res, 0.72);

    let mut ema_res_high = 0.99;
    emotive::apply_boot_bias(&mut ema_res_high, 0.05);
    approx_eq(ema_res_high, 1.0);
}

#[test]
fn load_save_roundtrip_appends_and_parses() {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("emote-test-{}.jsonl", unique));

    let seed_a = EmoteSeed {
        ema_drift: 0.25,
        ema_res: 0.74,
        tone: "Calm".to_string(),
        wpm: 154.0,
        ts_unix: 3_000,
    };
    let seed_b = EmoteSeed {
        ema_drift: 0.48,
        ema_res: 0.59,
        tone: "Neutral".to_string(),
        wpm: 168.0,
        ts_unix: 3_600,
    };

    let path_string = path.to_string_lossy().to_string();
    emotive::save_append(&path_string, &seed_a).unwrap();
    emotive::save_append(&path_string, &seed_b).unwrap();

    let loaded = emotive::load_latest(&path_string).expect("seed should load");
    approx_eq(loaded.ema_drift, seed_b.ema_drift);
    approx_eq(loaded.ema_res, seed_b.ema_res);
    approx_eq(loaded.wpm, seed_b.wpm);
    assert_eq!(loaded.tone, seed_b.tone);
    assert_eq!(loaded.ts_unix, seed_b.ts_unix);

    let _ = fs::remove_file(PathBuf::from(path_string));
}
