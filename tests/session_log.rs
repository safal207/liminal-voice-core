use std::fs;
use std::path::Path;

use liminal_voice_core::session;

#[test]
fn session_writes_jsonl() -> std::io::Result<()> {
    let tmp_dir = std::env::temp_dir().join("liminal_session_test");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }
    fs::create_dir_all(&tmp_dir)?;

    let log_dir = tmp_dir.join("logs_test");
    let log_dir_str = log_dir.to_string_lossy().to_string();

    let mut sess = session::start(2, &log_dir_str);
    session::open_file(&mut sess)?;

    let session_id = sess.id.clone();
    let log_dir_copy = sess.log_dir.clone();

    let snapshot1 = session::Snapshot {
        ts: "2024-01-01T00:00:00.000Z".to_string(),
        device: "test-device".to_string(),
        drift: 0.1,
        resonance: 0.2,
        wpm: 150.0,
        articulation: 0.5,
        tone: "Calm".to_string(),
        asr_ms: 10,
        tts_ms: 20,
        total_ms: 35,
        idx: 0,
        utterance: "hello liminal".to_string(),
    };

    let snapshot2 = session::Snapshot {
        ts: "2024-01-01T00:00:01.000Z".into(),
        tone: "Energetic".into(),
        idx: 1,
        utterance: "second".into(),
        ..snapshot1.clone()
    };

    session::write(&mut sess, &snapshot1)?;
    session::write(&mut sess, &snapshot2)?;

    session::close(sess);

    let log_path = Path::new(&log_dir_copy).join(format!("session-{}.jsonl", session_id));
    assert!(log_path.exists());

    let contents = fs::read_to_string(&log_path)?;
    assert!(!contents.is_empty());
    let lines: Vec<_> = contents.lines().collect();
    assert_eq!(lines.len(), 2);

    Ok(())
}
