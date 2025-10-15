use liminal_voice_core::device::{self, DeviceMode};

#[test]
fn detect_modes() {
    assert_eq!(device::detect("phone"), DeviceMode::Phone);
    assert_eq!(device::detect("headset"), DeviceMode::Headset);
    assert_eq!(device::detect("terminal"), DeviceMode::Terminal);
    assert_eq!(device::detect("UNKNOWN"), DeviceMode::Phone);
}

#[test]
fn profile_defaults_match() {
    let phone = device::profile(&DeviceMode::Phone);
    assert!((phone.gain_db + 2.0).abs() < f32::EPSILON);
    assert!((phone.pace_factor - 1.05).abs() < f32::EPSILON);
    assert_eq!(phone.pause_ms, 60);

    let headset = device::profile(&DeviceMode::Headset);
    assert!(headset.gain_db.abs() < f32::EPSILON);
    assert!((headset.pace_factor - 1.0).abs() < f32::EPSILON);
    assert_eq!(headset.pause_ms, 40);

    let terminal = device::profile(&DeviceMode::Terminal);
    assert!((terminal.gain_db - 1.5).abs() < f32::EPSILON);
    assert!((terminal.pace_factor - 0.95).abs() < f32::EPSILON);
    assert_eq!(terminal.pause_ms, 80);
}

#[test]
fn profile_ranges() {
    for mode in [DeviceMode::Phone, DeviceMode::Headset, DeviceMode::Terminal] {
        let prof = device::profile(&mode);
        assert!((0.8..=1.2).contains(&prof.pace_factor));
        assert!((20..=200).contains(&(prof.pause_ms as i32)));
    }
}
