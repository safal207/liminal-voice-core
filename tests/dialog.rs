use std::fs;
use std::io::Write;

use liminal_voice_core::config::Config;
use liminal_voice_core::dialog::{default_utterance, load_inputs};

#[test]
fn load_inputs_from_script() {
    let mut cfg = Config::default();
    cfg.script = Some("a;b;c".to_string());

    let items = load_inputs(&cfg);
    assert_eq!(items, vec!["a", "b", "c"]);
}

#[test]
fn load_inputs_from_file_trims_empty_lines() {
    let mut cfg = Config::default();
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "liminal_dialog_{}_{}.txt",
        std::process::id(),
        unique_suffix()
    ));
    let mut file = fs::File::create(&path).expect("create temp file");
    writeln!(file, "first").unwrap();
    writeln!(file).unwrap();
    writeln!(file, " second ").unwrap();
    file.sync_all().unwrap();

    cfg.inputs_path = Some(path.to_string_lossy().to_string());

    let items = load_inputs(&cfg);
    assert_eq!(items, vec!["first", "second"]);

    let _ = fs::remove_file(path);
}

#[test]
fn load_inputs_fallback_to_defaults() {
    let mut cfg = Config::default();
    cfg.cycles = 3;
    cfg.script = None;
    cfg.inputs_path = None;

    let items = load_inputs(&cfg);
    assert_eq!(items.len(), cfg.cycles);
    assert!(items.iter().all(|item| item == default_utterance()));
}

fn unique_suffix() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos()
}
