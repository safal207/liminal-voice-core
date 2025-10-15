use liminal_voice_core::alerts::{self, AlertStats};

#[test]
fn update_counts_breaches() {
    let mut stats = AlertStats::default();
    let samples = vec![(0.20, 0.70), (0.40, 0.60), (0.36, 0.80)];
    for (drift, res) in samples {
        alerts::update(&mut stats, drift, res, 0.35, 0.65);
    }

    assert_eq!(stats.total, 3);
    assert_eq!(stats.drift_breaches, 2);
    assert_eq!(stats.res_breaches, 1);
    assert!((stats.max_drift - 0.40).abs() < f32::EPSILON);
    assert!((stats.min_res - 0.60).abs() < f32::EPSILON);
}

#[test]
fn print_summary_includes_status() {
    let mut stats = AlertStats::default();
    alerts::update(&mut stats, 0.5, 0.7, 0.35, 0.65);

    alerts::print_summary(&stats, 0.35, 0.65);
    let lines = alerts::summary_lines(&stats, 0.35, 0.65);
    assert!(lines.iter().any(|line| line.contains("status:")));
}
