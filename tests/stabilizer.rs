use liminal_voice_core::stabilizer::{self, EmoState, Stabilizer, StabilizerCfg};

#[test]
fn progression_and_advice_mapping() {
    let cfg = StabilizerCfg {
        win: 5,
        ema_alpha: 0.4,
        warm_drift: 0.32,
        hot_drift: 0.42,
        low_res: 0.58,
        cool_steps: 3,
        calm_boost: 0.08,
    };

    let mut stab = Stabilizer::new(cfg);
    let cool_steps = stab.cfg.cool_steps;

    stab.push(0.20, 0.80);
    assert_eq!(stab.state, EmoState::Normal);
    let normal_adv = stab.advice();
    assert_eq!(normal_adv.pace_delta, 0.0);
    assert_eq!(normal_adv.pause_delta_ms, 0);

    stab.push(0.34, 0.70);
    assert_eq!(stab.state, EmoState::Warming);
    let warming_adv = stab.advice();
    assert!(warming_adv.pace_delta < 0.0);
    assert!(warming_adv.pause_delta_ms > 0);
    assert!(warming_adv.articulation_hint > 0.0);

    stab.push(0.45, 0.55);
    assert_eq!(stab.state, EmoState::Overheat);
    let overheat_adv = stab.advice();
    assert!(overheat_adv.pace_delta < -0.07);
    assert!(overheat_adv.pause_delta_ms >= 30);
    assert!(overheat_adv.articulation_hint > 0.04);

    for _ in 0..cool_steps {
        stab.push(0.30, 0.75);
        assert_eq!(stab.state, EmoState::Cooldown);
    }

    stab.push(0.25, 0.78);
    assert_eq!(stab.state, EmoState::Normal);

    let status = stabilizer::format_status(stab.state, stab.ema_drift, stab.ema_res);
    assert!(!status.is_empty());
    assert!(status.contains("state=Normal"));
}
