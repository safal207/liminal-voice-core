use liminal_voice_core::prosody::{self, ToneTag};

#[test]
fn basic_analysis_ranges() {
    let p = prosody::analyze("hello liminal", 1.0, 60);
    assert!((60.0..=220.0).contains(&p.wpm), "unexpected wpm: {}", p.wpm);
    assert!(
        (0.0..=1.0).contains(&p.articulation),
        "unexpected articulation: {}",
        p.articulation
    );
    assert!(matches!(
        p.tone,
        ToneTag::Neutral | ToneTag::Calm | ToneTag::Energetic
    ));
}

#[test]
fn empty_text_safe() {
    let p = prosody::analyze("", 1.0, 60);
    assert!(p.wpm >= 0.0);
    assert!((0.0..=1.0).contains(&p.articulation));
}

#[test]
fn extreme_parameters_clamped() {
    let slow = prosody::analyze("slow", 0.5, 20);
    assert!((0.0..=220.0).contains(&slow.wpm));
    assert!((0.0..=1.0).contains(&slow.articulation));

    let fast = prosody::analyze("fast", 1.5, 200);
    assert!((0.0..=220.0).contains(&fast.wpm));
    assert!((0.0..=1.0).contains(&fast.articulation));
}

#[test]
fn articulation_hint_clamped() {
    let boosted = prosody::apply_articulation_hint(0.9, 0.2);
    assert!((0.0..=1.0).contains(&boosted));
    let reduced = prosody::apply_articulation_hint(0.05, -0.2);
    assert!((0.0..=1.0).contains(&reduced));
}
