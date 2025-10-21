use liminal_voice_core::stabilizer::EmoState;
use liminal_voice_core::sync::{Baselines, Seeds, SyncCfg, SyncState};

fn default_cfg() -> SyncCfg {
    SyncCfg {
        lr_fast: 0.15,
        lr_slow: 0.05,
        clamp_step: 0.02,
    }
}

#[test]
fn step_reacts_to_high_drift_low_res() {
    let mut sync = SyncState::default();
    sync.warm_start(
        Seeds::default(),
        Baselines {
            drift: 0.35,
            res: 0.65,
        },
    );
    let cfg = default_cfg();

    let (pace, pause, res_boost, drift_relief) = sync.step(0.50, 0.55, EmoState::Normal, &cfg);

    assert!(
        pace < 0.0,
        "pace should slow down when drift exceeds baseline"
    );
    assert!(pause > 0, "pause should lengthen when resonance is low");
    assert!(
        res_boost > 0.0,
        "res_boost should encourage resonance recovery"
    );
    assert!(
        drift_relief <= f32::EPSILON,
        "drift_relief stays near zero when drift is high"
    );
}

#[test]
fn to_slow_increments_reflects_means() {
    let mut sync = SyncState::default();
    sync.warm_start(
        Seeds::default(),
        Baselines {
            drift: 0.30,
            res: 0.70,
        },
    );
    let cfg = default_cfg();

    for _ in 0..5 {
        let _ = sync.step(0.45, 0.60, EmoState::Normal, &cfg);
    }

    let (drift_bias, res_bias) = sync.to_slow_increments(&cfg);
    assert!(
        drift_bias < 0.0,
        "drift bias should push Astro toward lower drift"
    );
    assert!(res_bias > 0.0, "res bias should encourage higher resonance");
    assert!(drift_bias >= -0.03 && drift_bias <= 0.0);
    assert!(res_bias >= 0.0 && res_bias <= 0.03);
}

#[test]
fn no_steps_no_bias() {
    let sync = SyncState::default();
    let cfg = default_cfg();
    let (drift_bias, res_bias) = sync.to_slow_increments(&cfg);
    assert_eq!((drift_bias, res_bias), (0.0, 0.0));
}

#[test]
fn clamp_limits_slow_bias() {
    let mut sync = SyncState::default();
    sync.warm_start(
        Seeds::default(),
        Baselines {
            drift: 0.4,
            res: 0.4,
        },
    );
    sync.accum_drift = 10.0;
    sync.accum_res = 10.0;
    sync.steps = 1;
    let cfg = default_cfg();

    let (drift_bias, res_bias) = sync.to_slow_increments(&cfg);
    assert_eq!(drift_bias, -0.03);
    assert_eq!(res_bias, 0.03);
}
