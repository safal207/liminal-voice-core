//! Meta-cognitive awareness layer
//!
//! Tracks the system's own internal state and confidence levels.
//! Implements self-observation capabilities.
//!
//! Padmasambhava teaches: "Recognize the nature of mind, and you are free."
//! This module allows the system to observe its own state - meta-cognition.

use crate::metrics::clamp01;

/// Meta-cognitive state of the system
#[derive(Debug, Clone)]
pub struct MetaCognition {
    /// How unstable is the system itself? (0=stable, 1=chaotic)
    pub self_drift: f32,

    /// How present/aware is the system? (0=absent, 1=fully aware)
    pub self_resonance: f32,

    /// Confidence in current measurements (0=uncertain, 1=certain)
    pub confidence: f32,

    /// Clarity of understanding the situation (0=confused, 1=clear)
    pub clarity: f32,

    /// Level of doubt about actions (0=certain, 1=very doubtful)
    pub doubt: f32,

    /// Number of observations made
    pub observation_count: usize,
}

impl MetaCognition {
    pub fn new() -> Self {
        Self {
            self_drift: 0.0,
            self_resonance: 1.0,
            confidence: 0.5,  // Start neutral
            clarity: 0.5,
            doubt: 0.5,
            observation_count: 0,
        }
    }

    /// Observe the system's own state based on recent metrics
    pub fn observe(&mut self, measured_drift: f32, measured_res: f32,
                   stabilizer_state: &str, sync_corrections: f32) {
        self.observation_count += 1;

        // Self-drift: how much are our own parameters changing?
        // High sync corrections = high self-drift
        self.self_drift = clamp01(sync_corrections.abs() * 5.0);

        // Self-resonance: how stable/present are we?
        // If stabilizer is in Normal and measured_res is high = high self-resonance
        self.self_resonance = match stabilizer_state {
            "Normal" => clamp01(measured_res + 0.1),
            "Warming" => clamp01(measured_res),
            "Overheat" => clamp01(measured_res - 0.2),
            "Cooldown" => clamp01(measured_res - 0.1),
            _ => measured_res,
        };

        // Confidence: how sure are we about our measurements?
        // Low drift + high resonance = high confidence
        // High drift + low resonance = low confidence
        self.confidence = clamp01((1.0 - measured_drift) * measured_res);

        // Clarity: how well do we understand what's happening?
        // Increases with observation count (up to a point)
        let observation_bonus = (self.observation_count as f32 * 0.05).min(0.3);
        self.clarity = clamp01(self.confidence + observation_bonus);

        // Doubt: inverse of confidence with a floor
        self.doubt = clamp01(1.0 - self.confidence).max(0.1);
    }

    /// Should the system express uncertainty?
    pub fn should_express_doubt(&self) -> bool {
        self.doubt > 0.6 && self.confidence < 0.4
    }

    /// Is the system in a clear, stable state?
    pub fn is_clear_and_stable(&self) -> bool {
        self.clarity > 0.7 && self.self_drift < 0.3
    }

    /// Generate a self-assessment message
    pub fn self_assess(&self) -> String {
        let state = if self.is_clear_and_stable() {
            "Clear & Stable"
        } else if self.should_express_doubt() {
            "Uncertain"
        } else if self.self_drift > 0.5 {
            "Self-Adjusting"
        } else {
            "Observing"
        };

        format!(
            "self_state={} conf={:.2} clarity={:.2} doubt={:.2}",
            state, self.confidence, self.clarity, self.doubt
        )
    }
}

impl Default for MetaCognition {
    fn default() -> Self {
        Self::new()
    }
}

/// Meta-stabilizer: stabilizes the meta-cognition layer itself
pub struct MetaStabilizer {
    ema_self_drift: f32,
    ema_confidence: f32,
    alpha: f32,  // EMA smoothing factor
}

impl MetaStabilizer {
    pub fn new(alpha: f32) -> Self {
        Self {
            ema_self_drift: 0.0,
            ema_confidence: 0.5,
            alpha,
        }
    }

    /// Update EMA of meta-cognitive metrics
    pub fn update(&mut self, meta: &MetaCognition) {
        self.ema_self_drift = self.alpha * meta.self_drift
                            + (1.0 - self.alpha) * self.ema_self_drift;
        self.ema_confidence = self.alpha * meta.confidence
                            + (1.0 - self.alpha) * self.ema_confidence;
    }

    /// Get stabilized meta-metrics
    pub fn get_stable_metrics(&self) -> (f32, f32) {
        (self.ema_self_drift, self.ema_confidence)
    }

    /// Should we increase meta-awareness?
    pub fn needs_more_awareness(&self) -> bool {
        self.ema_self_drift > 0.4 || self.ema_confidence < 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metacognition_initialization() {
        let meta = MetaCognition::new();
        assert_eq!(meta.observation_count, 0);
        assert!(meta.confidence > 0.0 && meta.confidence <= 1.0);
    }

    #[test]
    fn test_observe_increases_clarity() {
        let mut meta = MetaCognition::new();
        let initial_clarity = meta.clarity;

        // Multiple observations should increase clarity
        for _ in 0..5 {
            meta.observe(0.2, 0.8, "Normal", 0.01);
        }

        assert!(meta.clarity > initial_clarity);
    }

    #[test]
    fn test_high_drift_low_resonance_increases_doubt() {
        let mut meta = MetaCognition::new();
        meta.observe(0.9, 0.2, "Overheat", 0.5);

        assert!(meta.doubt > 0.5);
        assert!(meta.confidence < 0.5);
    }

    #[test]
    fn test_meta_stabilizer() {
        let mut stabilizer = MetaStabilizer::new(0.3);
        let mut meta = MetaCognition::new();

        meta.observe(0.5, 0.5, "Normal", 0.1);
        stabilizer.update(&meta);

        let (drift, conf) = stabilizer.get_stable_metrics();
        assert!(drift >= 0.0 && drift <= 1.0);
        assert!(conf >= 0.0 && conf <= 1.0);
    }

    #[test]
    fn test_should_express_doubt() {
        let mut meta = MetaCognition::new();

        // High drift, low resonance should trigger doubt
        meta.observe(0.95, 0.1, "Overheat", 0.8);
        assert!(meta.should_express_doubt());
    }

    #[test]
    fn test_is_clear_and_stable() {
        let mut meta = MetaCognition::new();

        // Multiple stable observations
        for _ in 0..10 {
            meta.observe(0.15, 0.85, "Normal", 0.01);
        }

        assert!(meta.is_clear_and_stable());
    }
}
