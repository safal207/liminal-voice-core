use liminal_voice_core::astro::{AstroStore, normalize_theme};

#[test]
fn normalize_theme_uses_script_or_first_line() {
    let from_script = normalize_theme(Some("Focus;Calm"), &["hello".into()]);
    assert_eq!(from_script, "focus;calm");

    let from_lines = normalize_theme(None, &["Reflect".into(), "Calm".into()]);
    assert_eq!(from_lines, "reflect");
}

#[test]
fn consolidate_accumulates_biases() {
    let tmp = std::env::temp_dir().join("astro_integration_test.txt");
    if tmp.exists() {
        let _ = std::fs::remove_file(&tmp);
    }

    let path = tmp.to_string_lossy().to_string();
    let mut store = AstroStore::load(&path);
    store.consolidate("focus", -0.02, 0.015);
    store.consolidate("focus", -0.03, 0.02);
    store.save();

    let reloaded = AstroStore::load(&path);
    let advice = reloaded.suggest("focus").expect("advice exists");
    assert!(advice.drift_bias <= 0.0);
    assert!(advice.res_bias >= 0.0);
    assert!(advice.visits >= 1);
    assert!(advice.stability >= 0.0 && advice.stability <= 1.0);
}
