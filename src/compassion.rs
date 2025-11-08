//! Compassion metrics - detecting user suffering and responding with kindness
//!
//! Implements karuṇā (करुणा) - compassion as the wish to alleviate suffering.
//! Part of the Tikkun Olam framework - healing through conversation.
//!
//! "Tikkun olam begins with compassion for one's neighbor." - Hasidic wisdom

use crate::metrics::clamp01;
use crate::prosody::ToneTag;

/// Types of detected suffering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SufferingType {
    None,      // No suffering detected
    Mild,      // Slight distress
    Moderate,  // Clear distress signals
    Severe,    // High distress, needs support
}

/// Compassion metrics for the system
#[derive(Debug, Clone)]
pub struct CompassionMetrics {
    /// Detected level of user suffering (0=none, 1=severe)
    pub user_suffering: f32,

    /// Type of suffering detected
    pub suffering_type: SufferingType,

    /// How kind/gentle is the system's response? (0=harsh, 1=very kind)
    pub response_kindness: f32,

    /// Does the system intend to help/heal? (0=no, 1=strong healing intent)
    pub healing_intent: f32,

    /// Compassion activation level (0=inactive, 1=fully compassionate mode)
    pub compassion_level: f32,

    /// Number of times suffering was detected
    pub suffering_count: usize,

    /// Consecutive turns with suffering
    pub suffering_streak: usize,
}

impl CompassionMetrics {
    pub fn new() -> Self {
        Self {
            user_suffering: 0.0,
            suffering_type: SufferingType::None,
            response_kindness: 0.5, // Start neutral
            healing_intent: 0.3,    // Some baseline care
            compassion_level: 0.0,
            suffering_count: 0,
            suffering_streak: 0,
        }
    }

    /// Detect user suffering from conversational metrics
    pub fn detect_suffering(
        &mut self,
        drift: f32,
        resonance: f32,
        tone: ToneTag,
        wpm: f32,
        stabilizer_state: &str,
        repeated_theme: bool,
    ) {
        let mut suffering_score = 0.0;

        // Pattern 1: High drift + low resonance = emotional chaos
        if drift > 0.5 && resonance < 0.6 {
            suffering_score += (drift - 0.5) * 2.0; // Amplify signal
            suffering_score += (0.6 - resonance) * 1.5;
        }

        // Pattern 2: Overheat state = overwhelmed
        if stabilizer_state == "Overheat" {
            suffering_score += 0.3;
        }

        // Pattern 3: Fast chaotic speech (anxiety)
        if matches!(tone, ToneTag::Energetic) && wpm > 180.0 {
            suffering_score += 0.2;
        }

        // Pattern 4: Repeated theme without progress (stuck)
        if repeated_theme {
            suffering_score += 0.25;
            self.suffering_streak += 1;
        } else {
            self.suffering_streak = 0;
        }

        // Pattern 5: Extended suffering streak
        if self.suffering_streak > 2 {
            suffering_score += 0.3;
        }

        self.user_suffering = clamp01(suffering_score);

        // Classify suffering type
        self.suffering_type = if self.user_suffering < 0.2 {
            SufferingType::None
        } else if self.user_suffering < 0.4 {
            SufferingType::Mild
        } else if self.user_suffering < 0.7 {
            SufferingType::Moderate
        } else {
            SufferingType::Severe
        };

        if self.user_suffering > 0.2 {
            self.suffering_count += 1;
        }

        // Update healing intent based on suffering
        self.healing_intent = clamp01(0.3 + self.user_suffering * 0.7);
    }

    /// Calculate response kindness based on system behavior
    pub fn calculate_kindness(
        &mut self,
        was_rephrased: bool,
        pace_delta: f32,
        pause_delta_ms: i64,
        resonance_boost: f32,
    ) {
        let mut kindness = 0.5;

        // Rephrasing to help = kind
        if was_rephrased {
            kindness += 0.2;
        }

        // Slowing down pace = gentle
        if pace_delta < 0.0 {
            kindness += pace_delta.abs() * 0.5;
        }

        // Adding pauses = giving space
        if pause_delta_ms > 0 {
            kindness += (pause_delta_ms as f32 / 100.0).min(0.2);
        }

        // Boosting resonance = caring
        if resonance_boost > 0.0 {
            kindness += resonance_boost * 2.0;
        }

        self.response_kindness = clamp01(kindness);
    }

    /// Update overall compassion activation level
    pub fn update_compassion_level(&mut self) {
        // Compassion activates proportionally to suffering
        // But also considers healing intent and kindness
        let activation = (self.user_suffering * 0.5)
            + (self.healing_intent * 0.3)
            + (self.response_kindness * 0.2);

        self.compassion_level = clamp01(activation);
    }

    /// Should the system activate compassionate mode?
    pub fn should_activate_compassion(&self) -> bool {
        self.compassion_level > 0.5
    }

    /// Should the system offer explicit support?
    pub fn should_offer_support(&self) -> bool {
        matches!(
            self.suffering_type,
            SufferingType::Moderate | SufferingType::Severe
        )
    }

    /// Get a compassion status message
    pub fn status_message(&self) -> String {
        match self.suffering_type {
            SufferingType::None => {
                format!("Compassion: Observing (suffering={:.2})", self.user_suffering)
            }
            SufferingType::Mild => {
                format!(
                    "Compassion: Gentle Care (suffering={:.2}, healing={:.2})",
                    self.user_suffering, self.healing_intent
                )
            }
            SufferingType::Moderate => {
                format!(
                    "Compassion: Active Support (suffering={:.2}, kindness={:.2})",
                    self.user_suffering, self.response_kindness
                )
            }
            SufferingType::Severe => {
                format!(
                    "Compassion: ❤️  Deep Care (suffering={:.2}, streak={})",
                    self.user_suffering, self.suffering_streak
                )
            }
        }
    }
}

impl Default for CompassionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Compassion adjustments to apply to the system
#[derive(Debug, Clone, Copy)]
pub struct CompassionAdjustments {
    /// Extra resonance boost for compassionate mode
    pub resonance_boost: f32,

    /// Pace adjustment (slower = gentler)
    pub pace_adjustment: f32,

    /// Extra pause time (more space)
    pub pause_adjustment_ms: i64,

    /// Drift reduction (calming)
    pub drift_reduction: f32,
}

impl CompassionAdjustments {
    /// Generate adjustments based on compassion level
    pub fn from_compassion(metrics: &CompassionMetrics) -> Self {
        let level = metrics.compassion_level;

        Self {
            // Higher compassion = more resonance
            resonance_boost: level * 0.1,

            // Slow down to be gentle
            pace_adjustment: -level * 0.05,

            // Add pauses to give space
            pause_adjustment_ms: (level * 30.0) as i64,

            // Reduce drift to calm
            drift_reduction: level * 0.08,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compassion_initialization() {
        let comp = CompassionMetrics::new();
        assert_eq!(comp.user_suffering, 0.0);
        assert_eq!(comp.suffering_type, SufferingType::None);
        assert!(comp.response_kindness > 0.0);
    }

    #[test]
    fn test_detect_high_drift_low_resonance_suffering() {
        let mut comp = CompassionMetrics::new();
        comp.detect_suffering(0.9, 0.3, ToneTag::Energetic, 150.0, "Overheat", false);

        assert!(comp.user_suffering > 0.5);
        assert!(matches!(
            comp.suffering_type,
            SufferingType::Moderate | SufferingType::Severe
        ));
    }

    #[test]
    fn test_suffering_streak_increases_score() {
        let mut comp = CompassionMetrics::new();

        // First detection with repeated theme
        comp.detect_suffering(0.6, 0.5, ToneTag::Neutral, 150.0, "Normal", true);
        let first_score = comp.user_suffering;

        // Second detection with repeated theme
        comp.detect_suffering(0.6, 0.5, ToneTag::Neutral, 150.0, "Normal", true);
        let second_score = comp.user_suffering;

        // Streak should increase suffering
        assert!(second_score >= first_score);
        assert_eq!(comp.suffering_streak, 2);
    }

    #[test]
    fn test_kindness_calculation() {
        let mut comp = CompassionMetrics::new();

        // Compassionate actions: rephrased, slowed down, added pauses, boosted resonance
        comp.calculate_kindness(true, -0.05, 25, 0.03);

        assert!(comp.response_kindness > 0.7);
    }

    #[test]
    fn test_compassion_activation() {
        let mut comp = CompassionMetrics::new();

        // Detect severe suffering
        comp.detect_suffering(0.95, 0.2, ToneTag::Energetic, 200.0, "Overheat", true);
        comp.calculate_kindness(true, -0.1, 50, 0.05);
        comp.update_compassion_level();

        assert!(comp.should_activate_compassion());
        assert!(comp.should_offer_support());
    }

    #[test]
    fn test_compassion_adjustments() {
        let mut comp = CompassionMetrics::new();
        comp.compassion_level = 0.8;

        let adj = CompassionAdjustments::from_compassion(&comp);

        assert!(adj.resonance_boost > 0.0);
        assert!(adj.pace_adjustment < 0.0); // Slower
        assert!(adj.pause_adjustment_ms > 0); // More pauses
        assert!(adj.drift_reduction > 0.0);
    }
}
