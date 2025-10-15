#[path = "../src/adaptive_qa.rs"]
mod adaptive_qa;

#[test]
fn drift_and_resonance_within_range() {
    let (drift, res) = adaptive_qa::analyze_prompt("hello liminal");
    assert!((0.0..=1.0).contains(&drift), "drift out of range: {}", drift);
    assert!((0.0..=1.0).contains(&res), "resonance out of range: {}", res);
}
