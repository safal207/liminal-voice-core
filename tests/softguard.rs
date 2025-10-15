use liminal_voice_core::softguard::{GuardAction, GuardConfig, check_and_rephrase};

fn default_cfg() -> GuardConfig {
    GuardConfig::default()
}

#[test]
fn guard_allows_stable_response() {
    let cfg = default_cfg();
    let result = check_and_rephrase("hello", 0.2, 0.9, &cfg);
    assert!(matches!(result, GuardAction::None));
}

#[test]
fn guard_warns_on_high_drift() {
    let cfg = default_cfg();
    let result = check_and_rephrase("hello", cfg.drift_limit + 0.1, cfg.res_limit + 0.1, &cfg);
    match result {
        GuardAction::Warn(msg) => {
            assert!(msg.contains("soft-guard"));
        }
        other => panic!("expected warn, got {:?}", other),
    }
}

#[test]
fn guard_rephrases_when_resonance_low() {
    let cfg = default_cfg();
    let result = check_and_rephrase("excited!", cfg.drift_limit + 0.2, cfg.res_limit - 0.2, &cfg);
    match result {
        GuardAction::Rephrased(text) => {
            assert!(text.contains("[recentered]"));
            assert!(!text.contains("!"));
        }
        other => panic!("expected rephrased, got {:?}", other),
    }
}

#[test]
fn guard_handles_empty_text() {
    let cfg = default_cfg();
    let _ = check_and_rephrase("", cfg.drift_limit + 0.5, cfg.res_limit - 0.5, &cfg);
}
