use liminal_voice_core::spark;

#[test]
fn sparkline_handles_empty() {
    assert_eq!(spark::sparkline(&[]), "");
}

#[test]
fn sparkline_uses_defined_glyphs() {
    let values = [0.0_f32, 0.5_f32, 1.0_f32];
    let line = spark::sparkline(&values);
    assert_eq!(line.chars().count(), values.len());
    for ch in line.chars() {
        assert!(spark::GLYPHS.contains(&ch));
    }
}
