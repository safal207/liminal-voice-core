use liminal_voice_core::viz;

#[test]
fn bar_zero_empty() {
    assert!(viz::bar(0.0, 10).is_empty());
}

#[test]
fn bar_full_length() {
    assert_eq!(viz::bar(1.0, 10).len(), 10);
}

#[test]
fn print_table_outputs_lines() {
    let lines = viz::print_table(0.12, 0.88, 162.0, 0.74, "Neutral", 45, 32, 90);
    assert!(!lines.is_empty());
    assert!(lines.iter().any(|line| line.contains("Semantic Drift")));
}
