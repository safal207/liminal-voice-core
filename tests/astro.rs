// Note: These tests are currently disabled as the astro module methods
// (normalize_theme, fold_sync_delta, suggest_sync) are not yet implemented.
// They will be re-enabled once the astro module is complete.

// use liminal_voice_core::astro::{AstroStore, normalize_theme};

// #[test]
// fn normalize_theme_uses_script_or_first_line() {
//     let from_script = normalize_theme(Some("Focus;Calm"), &["hello".into()]);
//     assert_eq!(from_script, "focus;calm");

//     let from_lines = normalize_theme(None, &["Reflect".into(), "Calm".into()]);
//     assert_eq!(from_lines, "reflect");
// }

// #[test]
// fn fold_sync_delta_accumulates_biases() {
//     let tmp = std::env::temp_dir().join("astro_integration_test.jsonl");
//     if tmp.exists() {
//         let _ = std::fs::remove_file(&tmp);
//     }

//     let path = tmp.to_string_lossy().to_string();
//     {
//         let mut store = AstroStore::load(&path, 8);
//         store.fold_sync_delta("focus", -0.02, 0.015, 10);
//         store.fold_sync_delta("focus", -0.03, 0.02, 20);
//     }

//     let reloaded = AstroStore::load(&path, 8);
//     let bias = reloaded.suggest_sync("focus").expect("bias exists");
//     assert!(bias.drift_bias <= 0.0);
//     assert!(bias.res_bias >= 0.0);
//     assert!(bias.visits >= 1);
//     assert!(bias.stability >= 0.0 && bias.stability <= 1.0);

//     let _ = std::fs::remove_file(&tmp);
// }
