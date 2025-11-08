use crate::awareness::MetaCognition;
use crate::metrics;
use crate::stabilizer::EmoState;

const LABEL_WIDTH: usize = 22;
const VALUE_WIDTH: usize = 25;
const BAR_WIDTH: usize = 19;

pub fn bar(value_0_1: f32, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let clamped = metrics::clamp01(value_0_1);
    if clamped <= 0.0 {
        return String::new();
    }
    let filled = (clamped * width as f32).round() as usize;
    let filled = filled.clamp(0, width);
    "#".repeat(filled)
}

pub fn print_table(
    drift: f32,
    res: f32,
    wpm: f32,
    articulation: f32,
    tone: &str,
    asr_ms: u128,
    tts_ms: u128,
    total_ms: u128,
    stab_state: Option<&str>,
    emote_seed: Option<&str>,
    meta_cognition: Option<&MetaCognition>,
) -> Vec<String> {
    let mut lines = Vec::new();
    let border = format!(
        "+{}+{}+",
        "-".repeat(LABEL_WIDTH + 2),
        "-".repeat(VALUE_WIDTH + 2)
    );
    let header = format!(
        "| {:<label$} | {:<value$} |",
        "Metric",
        "Value",
        label = LABEL_WIDTH,
        value = VALUE_WIDTH
    );

    lines.push(border.clone());
    lines.push(header);
    lines.push(border.clone());

    let drift_bar = format_bar_entry(drift);
    let res_bar = format_bar_entry(res);
    let articulation_bar = format_bar_entry(articulation);

    lines.push(format_row("Semantic Drift", &drift_bar));
    lines.push(format_row("Resonance", &res_bar));
    lines.push(format_row("WPM", &format!("{:.1}", wpm)));
    lines.push(format_row("Articulation", &articulation_bar));
    lines.push(format_row("Tone", tone));
    lines.push(format_row(
        "Latency (ASR/TTS/T)",
        &format!("{}ms / {}ms / {}ms", asr_ms, tts_ms, total_ms),
    ));
    if let Some(state) = stab_state {
        lines.push(format_row("Stabilizer State", state));
    }
    if let Some(seed) = emote_seed {
        lines.push(format_row("Emotive Seed", seed));
    }

    // Meta-cognition metrics (if available)
    if let Some(meta) = meta_cognition {
        lines.push(format_row(
            "Meta-Cognition",
            &format!("self_d={:.2} self_r={:.2}", meta.self_drift, meta.self_resonance),
        ));
        lines.push(format_row(
            "  Confidence/Clarity",
            &format!(
                "conf={:.2} clarity={:.2} doubt={:.2}",
                meta.confidence, meta.clarity, meta.doubt
            ),
        ));

        if meta.should_express_doubt() {
            lines.push(format_row("  Status", "⚠️  UNCERTAIN STATE"));
        }
    }

    lines.push(border);

    for line in &lines {
        println!("{}", line);
    }

    lines
}

pub fn print_compact_stabilizer(state: EmoState, ema_drift: f32, ema_res: f32) {
    println!(
        "[stab] {:?} d={:.2} r={:.2}",
        state,
        ema_drift.clamp(0.0, 1.0),
        ema_res.clamp(0.0, 1.0)
    );
}

fn format_bar_entry(value: f32) -> String {
    let bar = bar(value, BAR_WIDTH);
    if bar.is_empty() {
        format!("{:.2}", value)
    } else {
        format!("{:.2}  {:<width$}", value, bar, width = BAR_WIDTH)
    }
}

fn format_row(label: &str, value: &str) -> String {
    format!(
        "| {:<label$} | {:<value$} |",
        label,
        value,
        label = LABEL_WIDTH,
        value = VALUE_WIDTH
    )
}
