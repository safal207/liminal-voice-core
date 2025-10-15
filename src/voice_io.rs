#[allow(dead_code)]
pub fn record_audio() -> &'static str {
    "recorded.wav"
}

pub fn transcribe_audio() -> String {
    "hello liminal".to_string()
}

pub fn synthesize_response(text: &str) {
    println!("→ [voice]: {}", text);
}
