use liminal_voice_core::adaptive_qa;
use liminal_voice_core::device::{self, DeviceMode};

#[test]
fn drift_and_resonance_within_range() {
    let profile = device::profile(&DeviceMode::Phone);
    let (drift, res) = adaptive_qa::analyze_prompt("hello liminal", profile.pace_factor);
    assert!(
        (0.0..=1.0).contains(&drift),
        "drift out of range: {}",
        drift
    );
    assert!(
        (0.0..=1.0).contains(&res),
        "resonance out of range: {}",
        res
    );
}
