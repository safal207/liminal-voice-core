use liminal_voice_core::awareness::{MetaCognition, MetaStabilizer};

#[test]
fn test_awareness_basic_flow() {
    let mut meta = MetaCognition::new();

    // Simulate stable conversation
    for _ in 0..5 {
        meta.observe(0.15, 0.85, "Normal", 0.01);
    }

    assert!(meta.confidence > 0.7);
    assert!(meta.clarity > 0.6);
    assert!(meta.doubt < 0.4);
    assert!(!meta.should_express_doubt());
}

#[test]
fn test_awareness_uncertain_state() {
    let mut meta = MetaCognition::new();

    // Simulate chaotic conversation
    meta.observe(0.9, 0.2, "Overheat", 0.8);

    assert!(meta.doubt > 0.5);
    assert!(meta.confidence < 0.5);
    assert!(meta.should_express_doubt());
}

#[test]
fn test_meta_stabilizer_smoothing() {
    let mut stabilizer = MetaStabilizer::new(0.3);
    let mut meta = MetaCognition::new();

    // Spike in self-drift
    meta.observe(0.8, 0.3, "Overheat", 0.7);
    stabilizer.update(&meta);

    let (drift, _) = stabilizer.get_stable_metrics();

    // Should be smoothed (less than raw value)
    assert!(drift < meta.self_drift);
}

#[test]
fn test_clarity_increases_with_observations() {
    let mut meta = MetaCognition::new();
    let initial_clarity = meta.clarity;

    // Many stable observations
    for _ in 0..10 {
        meta.observe(0.2, 0.8, "Normal", 0.01);
    }

    assert!(meta.clarity > initial_clarity);
    assert!(meta.is_clear_and_stable());
}

#[test]
fn test_metacognition_initialization() {
    let meta = MetaCognition::new();
    assert_eq!(meta.observation_count, 0);
    assert!(meta.confidence > 0.0 && meta.confidence <= 1.0);
    assert_eq!(meta.self_drift, 0.0);
    assert_eq!(meta.self_resonance, 1.0);
}

#[test]
fn test_observe_increases_count() {
    let mut meta = MetaCognition::new();
    meta.observe(0.2, 0.8, "Normal", 0.01);
    assert_eq!(meta.observation_count, 1);

    meta.observe(0.3, 0.7, "Normal", 0.02);
    assert_eq!(meta.observation_count, 2);
}

#[test]
fn test_self_resonance_varies_by_stabilizer_state() {
    let mut meta = MetaCognition::new();

    meta.observe(0.2, 0.8, "Normal", 0.01);
    let normal_resonance = meta.self_resonance;

    meta.observe(0.2, 0.8, "Overheat", 0.01);
    let overheat_resonance = meta.self_resonance;

    // Overheat state should reduce self_resonance
    assert!(overheat_resonance < normal_resonance);
}

#[test]
fn test_high_sync_corrections_increase_self_drift() {
    let mut meta = MetaCognition::new();

    meta.observe(0.2, 0.8, "Normal", 0.01);
    let low_correction_drift = meta.self_drift;

    meta.observe(0.2, 0.8, "Normal", 0.5);
    let high_correction_drift = meta.self_drift;

    // High sync corrections should increase self_drift
    assert!(high_correction_drift > low_correction_drift);
}

#[test]
fn test_meta_stabilizer_needs_more_awareness() {
    let mut stabilizer = MetaStabilizer::new(0.3);
    let mut meta = MetaCognition::new();

    // Start with good state
    meta.observe(0.2, 0.8, "Normal", 0.01);
    stabilizer.update(&meta);
    assert!(!stabilizer.needs_more_awareness());

    // Chaotic state
    for _ in 0..5 {
        meta.observe(0.9, 0.2, "Overheat", 0.7);
        stabilizer.update(&meta);
    }
    assert!(stabilizer.needs_more_awareness());
}

#[test]
fn test_self_assess_message() {
    let mut meta = MetaCognition::new();

    // Clear and stable state
    for _ in 0..10 {
        meta.observe(0.15, 0.85, "Normal", 0.01);
    }
    let message = meta.self_assess();
    assert!(message.contains("Clear & Stable") || message.contains("Observing"));

    // Uncertain state
    let mut meta2 = MetaCognition::new();
    meta2.observe(0.95, 0.1, "Overheat", 0.9);
    let message2 = meta2.self_assess();
    assert!(message2.contains("Uncertain"));
}

#[test]
fn test_default_implementation() {
    let meta1 = MetaCognition::new();
    let meta2 = MetaCognition::default();

    assert_eq!(meta1.observation_count, meta2.observation_count);
    assert_eq!(meta1.confidence, meta2.confidence);
}
