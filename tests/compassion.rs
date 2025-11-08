//! Integration tests for compassion metrics
//!
//! Tests the compassion detection and adjustment system

use liminal_voice_core::compassion::{CompassionAdjustments, CompassionMetrics, SufferingType};
use liminal_voice_core::prosody::ToneTag;

#[test]
fn test_no_suffering_baseline() {
    let mut comp = CompassionMetrics::new();

    // Low drift, high resonance, normal state
    comp.detect_suffering(0.1, 0.8, ToneTag::Calm, 150.0, "Normal", false);

    assert_eq!(comp.suffering_type, SufferingType::None);
    assert!(comp.user_suffering < 0.3);
}

#[test]
fn test_high_drift_triggers_suffering() {
    let mut comp = CompassionMetrics::new();

    // High drift + low resonance = emotional chaos
    comp.detect_suffering(0.95, 0.3, ToneTag::Neutral, 150.0, "Normal", false);

    assert!(comp.user_suffering > 0.5);
    assert_ne!(comp.suffering_type, SufferingType::None);
}

#[test]
fn test_overheat_state_increases_suffering() {
    let mut comp = CompassionMetrics::new();

    // Overheat state should add to suffering score
    comp.detect_suffering(0.3, 0.6, ToneTag::Calm, 150.0, "Overheat", false);

    assert!(comp.user_suffering > 0.2);
}

#[test]
fn test_fast_energetic_speech_signals_anxiety() {
    let mut comp = CompassionMetrics::new();

    // Fast + energetic = anxiety
    comp.detect_suffering(0.4, 0.5, ToneTag::Energetic, 200.0, "Normal", false);

    assert!(comp.user_suffering > 0.15);
}

#[test]
fn test_repeated_theme_increases_suffering() {
    let mut comp = CompassionMetrics::new();

    // First without repeated theme
    comp.detect_suffering(0.3, 0.6, ToneTag::Calm, 150.0, "Normal", false);
    let suffering_without_repeat = comp.user_suffering;

    // Reset and test with repeated theme
    comp = CompassionMetrics::new();
    comp.detect_suffering(0.3, 0.6, ToneTag::Calm, 150.0, "Normal", true);
    let suffering_with_repeat = comp.user_suffering;

    assert!(suffering_with_repeat > suffering_without_repeat);
    assert_eq!(comp.suffering_streak, 1);
}

#[test]
fn test_suffering_streak_accumulates() {
    let mut comp = CompassionMetrics::new();

    // Build up streak with repeated theme
    for _ in 0..5 {
        comp.detect_suffering(0.5, 0.5, ToneTag::Calm, 150.0, "Normal", true);
    }

    assert!(comp.suffering_streak >= 5);
    // Extended streak (>2) should increase suffering further
    assert!(comp.user_suffering > 0.4);
}

#[test]
fn test_suffering_streak_resets_without_repeat() {
    let mut comp = CompassionMetrics::new();

    // Build up streak
    for _ in 0..3 {
        comp.detect_suffering(0.5, 0.5, ToneTag::Calm, 150.0, "Normal", true);
    }
    assert!(comp.suffering_streak > 0);

    // Break the streak
    comp.detect_suffering(0.2, 0.8, ToneTag::Calm, 150.0, "Normal", false);
    assert_eq!(comp.suffering_streak, 0);
}

#[test]
fn test_suffering_type_classification() {
    let mut comp = CompassionMetrics::new();

    // None (< 0.2)
    comp.detect_suffering(0.1, 0.9, ToneTag::Calm, 150.0, "Normal", false);
    assert_eq!(comp.suffering_type, SufferingType::None);

    // Mild (0.2-0.4) - Need drift > 0.5 to trigger, let's use 0.55 with resonance 0.5
    comp.detect_suffering(0.55, 0.5, ToneTag::Calm, 150.0, "Normal", false);
    assert_eq!(comp.suffering_type, SufferingType::Mild);

    // Moderate/Severe (>0.4) - High drift + low resonance + overheat
    comp.detect_suffering(0.8, 0.3, ToneTag::Energetic, 200.0, "Overheat", false);
    assert!(matches!(
        comp.suffering_type,
        SufferingType::Moderate | SufferingType::Severe
    ));
}

#[test]
fn test_calculate_kindness_with_interventions() {
    let mut comp = CompassionMetrics::new();

    // First detect suffering
    comp.detect_suffering(0.8, 0.3, ToneTag::Neutral, 150.0, "Overheat", true);

    // Calculate kindness with some interventions
    comp.calculate_kindness(true, -0.05, 20, 0.1);

    // Kindness should be positive with interventions
    assert!(comp.response_kindness > 0.5);
}

#[test]
fn test_calculate_kindness_without_interventions() {
    let mut comp = CompassionMetrics::new();

    // Detect suffering
    comp.detect_suffering(0.7, 0.4, ToneTag::Neutral, 150.0, "Normal", false);

    // No interventions
    comp.calculate_kindness(false, 0.0, 0, 0.0);

    // Kindness should still be present but lower
    assert!(comp.response_kindness >= 0.5);
}

#[test]
fn test_update_compassion_level() {
    let mut comp = CompassionMetrics::new();

    // Set up some values
    comp.detect_suffering(0.6, 0.4, ToneTag::Neutral, 150.0, "Normal", false);
    comp.calculate_kindness(true, -0.03, 15, 0.05);
    comp.update_compassion_level();

    // Compassion level = suffering * 0.5 + healing * 0.3 + kindness * 0.2
    let expected = (comp.user_suffering * 0.5)
        + (comp.healing_intent * 0.3)
        + (comp.response_kindness * 0.2);
    assert!((comp.compassion_level - expected).abs() < 0.01);
}

#[test]
fn test_should_offer_support() {
    let mut comp = CompassionMetrics::new();

    // Low suffering - no support needed
    comp.detect_suffering(0.1, 0.9, ToneTag::Calm, 150.0, "Normal", false);
    assert!(!comp.should_offer_support());

    // High suffering - support needed
    comp.detect_suffering(0.9, 0.2, ToneTag::Energetic, 200.0, "Overheat", true);
    assert!(comp.should_offer_support());
}

#[test]
fn test_should_activate_compassion() {
    let mut comp = CompassionMetrics::new();

    // No suffering
    comp.detect_suffering(0.1, 0.9, ToneTag::Calm, 150.0, "Normal", false);
    comp.update_compassion_level();
    assert!(!comp.should_activate_compassion());

    // Significant suffering
    comp.detect_suffering(0.8, 0.3, ToneTag::Neutral, 150.0, "Overheat", true);
    comp.calculate_kindness(true, -0.05, 20, 0.1);
    comp.update_compassion_level();
    assert!(comp.should_activate_compassion());
}

#[test]
fn test_get_adjustments_scales_with_suffering() {
    let mut comp = CompassionMetrics::new();

    // Mild suffering
    comp.detect_suffering(0.3, 0.6, ToneTag::Calm, 150.0, "Normal", false);
    comp.update_compassion_level();
    let adj_mild = CompassionAdjustments::from_compassion(&comp);

    // Severe suffering
    comp.detect_suffering(0.9, 0.2, ToneTag::Energetic, 200.0, "Overheat", true);
    comp.calculate_kindness(true, -0.1, 30, 0.15);
    comp.update_compassion_level();
    let adj_severe = CompassionAdjustments::from_compassion(&comp);

    // Severe should have stronger adjustments
    assert!(adj_severe.resonance_boost > adj_mild.resonance_boost);
    assert!(adj_severe.drift_reduction.abs() > adj_mild.drift_reduction.abs());
}

#[test]
fn test_status_message_format() {
    let mut comp = CompassionMetrics::new();
    comp.detect_suffering(0.6, 0.4, ToneTag::Neutral, 150.0, "Normal", true);
    comp.calculate_kindness(false, 0.0, 0, 0.0);
    comp.update_compassion_level();

    let msg = comp.status_message();

    // Should contain "Compassion" and "suffering" at minimum
    assert!(msg.contains("Compassion"));
    assert!(msg.contains("suffering"));
}

#[test]
fn test_healing_intent_increases_with_suffering() {
    let mut comp = CompassionMetrics::new();

    // Low suffering
    comp.detect_suffering(0.1, 0.9, ToneTag::Calm, 150.0, "Normal", false);
    let healing_low = comp.healing_intent;

    // High suffering
    comp.detect_suffering(0.9, 0.2, ToneTag::Energetic, 200.0, "Overheat", true);
    let healing_high = comp.healing_intent;

    assert!(healing_high > healing_low);
}

#[test]
fn test_suffering_count_increments() {
    let mut comp = CompassionMetrics::new();

    assert_eq!(comp.suffering_count, 0);

    // Below threshold (0.2) - should not increment
    comp.detect_suffering(0.1, 0.9, ToneTag::Calm, 150.0, "Normal", false);
    assert_eq!(comp.suffering_count, 0);

    // Above threshold - should increment
    comp.detect_suffering(0.6, 0.4, ToneTag::Neutral, 150.0, "Normal", false);
    assert!(comp.suffering_count > 0);

    // Another above threshold
    comp.detect_suffering(0.7, 0.3, ToneTag::Energetic, 180.0, "Overheat", false);
    assert!(comp.suffering_count > 1);
}
