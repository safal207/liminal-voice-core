mod voice_io;
mod adaptive_qa;

fn main() {
    let text = voice_io::transcribe_audio();
    let (drift, res) = adaptive_qa::analyze_prompt(&text);
    voice_io::synthesize_response(&format!(
        "Semantic Drift: {:.2}, Resonance: {:.2}",
        drift, res
    ));
}
