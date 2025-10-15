use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Session {
    pub id: String,
    #[allow(dead_code)]
    pub cycles: usize,
    pub log_dir: String,
    file: Option<File>,
}

#[derive(Clone)]
pub struct Snapshot {
    pub ts: String,
    pub device: String,
    pub drift: f32,
    pub resonance: f32,
    pub wpm: f32,
    pub articulation: f32,
    pub tone: String,
    pub asr_ms: u128,
    pub tts_ms: u128,
    pub total_ms: u128,
    pub idx: usize,
    pub utterance: String,
    pub guard: Option<String>,
    pub state: Option<String>,
    pub emote_state: Option<String>,
}

pub fn start(cycles: usize, log_dir: &str) -> Session {
    Session {
        id: generate_id(),
        cycles,
        log_dir: log_dir.to_string(),
        file: None,
    }
}

pub fn open_file(sess: &mut Session) -> io::Result<()> {
    let path = session_path(sess);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    sess.file = Some(File::create(path)?);
    Ok(())
}

pub fn write(sess: &mut Session, snap: &Snapshot) -> io::Result<()> {
    let file = sess
        .file
        .as_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "session file not opened"))?;

    let guard_value = match snap.guard.as_ref() {
        Some(value) => format!("\"{}\"", escape_json(value)),
        None => "null".to_string(),
    };
    let state_value = match snap.state.as_ref() {
        Some(value) => format!("\"{}\"", escape_json(value)),
        None => "null".to_string(),
    };
    let emote_value = match snap.emote_state.as_ref() {
        Some(value) => format!("\"{}\"", escape_json(value)),
        None => "null".to_string(),
    };

    let line = format!(
        r#"{{"ts":"{}","device":"{}","drift":{:.3},"resonance":{:.3},"wpm":{:.3},"articulation":{:.3},"tone":"{}","asr_ms":{},"tts_ms":{},"total_ms":{},"idx":{},"utt":"{}","guard":{},"state":{},"emote_state":{}}}"#,
        escape_json(&snap.ts),
        escape_json(&snap.device),
        snap.drift,
        snap.resonance,
        snap.wpm,
        snap.articulation,
        escape_json(&snap.tone),
        snap.asr_ms,
        snap.tts_ms,
        snap.total_ms,
        snap.idx,
        escape_json(&snap.utterance),
        guard_value,
        state_value,
        emote_value
    );

    writeln!(file, "{}", line)
}

pub fn close(mut sess: Session) {
    if let Some(mut file) = sess.file.take() {
        let _ = file.flush();
    }
}

fn session_path(sess: &Session) -> PathBuf {
    Path::new(&sess.log_dir).join(format!("session-{}.jsonl", sess.id))
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn generate_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = now.as_nanos();
    let hex = format!("{:016x}", nanos);
    let len = hex.len();
    hex[len.saturating_sub(8)..].to_string()
}

#[cfg(test)]
mod tests {
    use super::escape_json;

    #[test]
    fn escape_handles_quotes() {
        assert_eq!(escape_json("\"test\\"), "\\\"test\\\\");
    }
}
