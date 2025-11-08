# Iteration 1.13: Silence Detection (Детектор Молчания)

## 🎯 Цель

Добавить способность системы **различать типы молчания** и **реагировать соответственно**.

**Падмасамбхава учит:** *"В промежутках между мыслями живет ригпа (ясное осознание)."*

**Буддизм учит:** मौन (mauna) - священное молчание как путь к истине. Пауза между словами так же важна, как сами слова.

**Каббала учит:** Между буквами Торы живет божественное молчание. Пространство между словами содержит тайный смысл.

Система должна:
- Обнаруживать периоды молчания в разговоре
- Различать типы молчания (покой vs страх vs размышление)
- Реагировать уместно на каждый тип

---

## 📐 Архитектура

### 1. Новый модуль: `src/silence.rs`

```rust
//! Silence detection - recognizing meaningful pauses in conversation
//!
//! Implements mauna (मौन) - sacred silence as a path to truth.
//! The space between words contains meaning.
//!
//! "In the gap between thoughts, rigpa dwells." - Padmasambhava

use crate::metrics::clamp01;
use crate::prosody::ToneTag;

/// Types of silence detected in conversation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SilenceType {
    None,           // No significant silence
    Contemplation,  // Thoughtful pause (healthy)
    Peace,          // Deep calm, settled mind
    Uncertainty,    // Don't know what to say (mild anxiety)
    Fear,           // Afraid to speak (high anxiety)
    Disconnect,     // Lost connection, dissociation
}

/// Metrics for silence periods in conversation
#[derive(Debug, Clone)]
pub struct SilenceMetrics {
    /// Duration of current silence in seconds
    pub current_silence_duration: f32,

    /// Type of silence currently detected
    pub silence_type: SilenceType,

    /// Quality of silence (0=disturbed, 1=peaceful)
    pub silence_quality: f32,

    /// Is this silence generative (leading to insight)?
    pub is_generative: bool,

    /// Should system break the silence?
    pub should_interrupt: bool,

    /// Number of silence periods this session
    pub silence_count: usize,

    /// Total time in silence this session (seconds)
    pub total_silence_time: f32,

    /// Longest silence period (seconds)
    pub max_silence_duration: f32,

    /// Average silence quality across session
    pub avg_silence_quality: f32,
}

impl SilenceMetrics {
    pub fn new() -> Self {
        Self {
            current_silence_duration: 0.0,
            silence_type: SilenceType::None,
            silence_quality: 0.5,
            is_generative: false,
            should_interrupt: false,
            silence_count: 0,
            total_silence_time: 0.0,
            max_silence_duration: 0.0,
            avg_silence_quality: 0.5,
        }
    }

    /// Detect and classify a silence period
    ///
    /// # Parameters
    /// - `duration_sec`: How long has the silence lasted
    /// - `last_drift`: Drift value before silence
    /// - `last_resonance`: Resonance before silence
    /// - `last_tone`: Prosody tone before silence
    /// - `user_suffering`: Current compassion suffering level
    /// - `stabilizer_state`: Current emotional state
    pub fn detect_silence(
        &mut self,
        duration_sec: f32,
        last_drift: f32,
        last_resonance: f32,
        last_tone: ToneTag,
        user_suffering: f32,
        stabilizer_state: &str,
    ) {
        self.current_silence_duration = duration_sec;

        // No significant silence yet
        if duration_sec < 1.5 {
            self.silence_type = SilenceType::None;
            self.should_interrupt = false;
            return;
        }

        // Classify based on context before silence
        self.silence_type = self.classify_silence(
            duration_sec,
            last_drift,
            last_resonance,
            last_tone,
            user_suffering,
            stabilizer_state,
        );

        // Calculate silence quality
        self.silence_quality = self.calculate_quality(last_drift, last_resonance, user_suffering);

        // Determine if generative
        self.is_generative = matches!(
            self.silence_type,
            SilenceType::Contemplation | SilenceType::Peace
        ) && self.silence_quality > 0.6;

        // Decide if we should interrupt
        self.should_interrupt = self.should_break_silence(duration_sec);

        // Update stats
        if duration_sec > 1.5 && self.silence_count == 0 {
            self.silence_count += 1;
        }
        self.total_silence_time += duration_sec;
        if duration_sec > self.max_silence_duration {
            self.max_silence_duration = duration_sec;
        }
    }

    /// Classify the type of silence based on context
    fn classify_silence(
        &self,
        duration: f32,
        last_drift: f32,
        last_resonance: f32,
        last_tone: ToneTag,
        user_suffering: f32,
        stabilizer_state: &str,
    ) -> SilenceType {
        // Pattern 1: Low drift + high resonance + calm = Peace
        if last_drift < 0.3 && last_resonance > 0.7 && matches!(last_tone, ToneTag::Calm) {
            return SilenceType::Peace;
        }

        // Pattern 2: Moderate drift + moderate resonance + medium duration = Contemplation
        if duration < 5.0
            && last_drift < 0.5
            && last_resonance > 0.5
            && matches!(last_tone, ToneTag::Neutral | ToneTag::Calm)
        {
            return SilenceType::Contemplation;
        }

        // Pattern 3: High suffering + silence = Fear
        if user_suffering > 0.6 && duration > 3.0 {
            return SilenceType::Fear;
        }

        // Pattern 4: High drift + low resonance = Disconnect
        if last_drift > 0.6 && last_resonance < 0.4 {
            return SilenceType::Disconnect;
        }

        // Pattern 5: Overheat then silence = Uncertainty
        if stabilizer_state == "Overheat" || stabilizer_state == "Warming" {
            return SilenceType::Uncertainty;
        }

        // Default: gentle contemplation
        SilenceType::Contemplation
    }

    /// Calculate the quality of silence (peaceful vs disturbed)
    fn calculate_quality(&self, last_drift: f32, last_resonance: f32, user_suffering: f32) -> f32 {
        let mut quality = 0.5;

        // High resonance before silence = peaceful quality
        quality += (last_resonance - 0.5) * 0.4;

        // Low drift before silence = calm quality
        quality += (0.5 - last_drift) * 0.3;

        // Low suffering = better quality
        quality += (1.0 - user_suffering) * 0.3;

        clamp01(quality)
    }

    /// Should the system break the silence?
    fn should_break_silence(&self, duration: f32) -> bool {
        match self.silence_type {
            // Never interrupt peace or contemplation if quality is good
            SilenceType::Peace | SilenceType::Contemplation => {
                if self.silence_quality > 0.6 {
                    duration > 12.0 // Only after very long silence
                } else {
                    duration > 6.0
                }
            }

            // Interrupt fear/disconnect sooner with support
            SilenceType::Fear | SilenceType::Disconnect => duration > 4.0,

            // Gently interrupt uncertainty
            SilenceType::Uncertainty => duration > 5.0,

            SilenceType::None => false,
        }
    }

    /// Reset silence tracking (called when user speaks)
    pub fn reset_silence(&mut self) {
        // Update average quality before reset
        if self.silence_count > 0 {
            self.avg_silence_quality = (self.avg_silence_quality * (self.silence_count as f32)
                + self.silence_quality)
                / (self.silence_count as f32 + 1.0);
        }

        self.current_silence_duration = 0.0;
        self.silence_type = SilenceType::None;
        self.should_interrupt = false;
    }

    /// Get a status message about current silence
    pub fn status_message(&self) -> String {
        if matches!(self.silence_type, SilenceType::None) {
            return format!("Silence: {} periods, {:.1}s total", self.silence_count, self.total_silence_time);
        }

        match self.silence_type {
            SilenceType::Peace => format!(
                "Silence: 🕉️  Peace ({:.1}s, quality={:.2})",
                self.current_silence_duration, self.silence_quality
            ),
            SilenceType::Contemplation => format!(
                "Silence: 💭 Contemplation ({:.1}s, {})",
                self.current_silence_duration,
                if self.is_generative { "generative" } else { "processing" }
            ),
            SilenceType::Uncertainty => format!(
                "Silence: 🤔 Uncertainty ({:.1}s, quality={:.2})",
                self.current_silence_duration, self.silence_quality
            ),
            SilenceType::Fear => format!(
                "Silence: 😰 Fear ({:.1}s, need support)",
                self.current_silence_duration
            ),
            SilenceType::Disconnect => format!(
                "Silence: 🌫️  Disconnect ({:.1}s, need reconnection)",
                self.current_silence_duration
            ),
            SilenceType::None => String::new(),
        }
    }

    /// Get intervention message if system should break silence
    pub fn intervention_message(&self) -> Option<&str> {
        if !self.should_interrupt {
            return None;
        }

        match self.silence_type {
            SilenceType::Fear => Some("I'm here if you need to talk..."),
            SilenceType::Disconnect => Some("Are you still with me?"),
            SilenceType::Uncertainty => Some("Take your time... What's on your mind?"),
            SilenceType::Peace | SilenceType::Contemplation => {
                Some("When you're ready, I'm listening...")
            }
            SilenceType::None => None,
        }
    }
}

impl Default for SilenceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Adjustments to make based on silence type
#[derive(Debug, Clone, Copy)]
pub struct SilenceAdjustments {
    /// How much to boost resonance after silence
    pub resonance_boost: f32,

    /// Pace adjustment (slower for fear, normal for peace)
    pub pace_adjustment: f32,

    /// Extra pause after breaking silence
    pub pause_adjustment_ms: i64,

    /// Warmth/gentleness in response
    pub warmth_boost: f32,
}

impl SilenceAdjustments {
    /// Generate adjustments based on silence type
    pub fn from_silence(metrics: &SilenceMetrics) -> Self {
        match metrics.silence_type {
            SilenceType::Peace | SilenceType::Contemplation => Self {
                // Honor the peaceful state
                resonance_boost: 0.02,
                pace_adjustment: -0.02, // Slightly slower
                pause_adjustment_ms: 15,
                warmth_boost: 0.05,
            },

            SilenceType::Fear | SilenceType::Disconnect => Self {
                // Provide strong support
                resonance_boost: 0.08,
                pace_adjustment: -0.08, // Much slower
                pause_adjustment_ms: 40,
                warmth_boost: 0.15,
            },

            SilenceType::Uncertainty => Self {
                // Gentle encouragement
                resonance_boost: 0.05,
                pace_adjustment: -0.04,
                pause_adjustment_ms: 25,
                warmth_boost: 0.08,
            },

            SilenceType::None => Self {
                resonance_boost: 0.0,
                pace_adjustment: 0.0,
                pause_adjustment_ms: 0,
                warmth_boost: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_initialization() {
        let silence = SilenceMetrics::new();
        assert_eq!(silence.silence_type, SilenceType::None);
        assert_eq!(silence.silence_count, 0);
    }

    #[test]
    fn test_peaceful_silence_detected() {
        let mut silence = SilenceMetrics::new();
        silence.detect_silence(3.0, 0.2, 0.8, ToneTag::Calm, 0.1, "Normal");

        assert_eq!(silence.silence_type, SilenceType::Peace);
        assert!(silence.silence_quality > 0.6);
        assert!(silence.is_generative);
        assert!(!silence.should_interrupt);
    }

    #[test]
    fn test_contemplation_silence() {
        let mut silence = SilenceMetrics::new();
        silence.detect_silence(3.5, 0.4, 0.6, ToneTag::Neutral, 0.2, "Normal");

        assert_eq!(silence.silence_type, SilenceType::Contemplation);
        assert!(!silence.should_interrupt); // Not long enough yet
    }

    #[test]
    fn test_fear_silence_needs_support() {
        let mut silence = SilenceMetrics::new();
        silence.detect_silence(4.5, 0.5, 0.5, ToneTag::Energetic, 0.8, "Warming");

        assert_eq!(silence.silence_type, SilenceType::Fear);
        assert!(silence.should_interrupt);
        assert!(silence.intervention_message().is_some());
    }

    #[test]
    fn test_disconnect_detection() {
        let mut silence = SilenceMetrics::new();
        silence.detect_silence(5.0, 0.85, 0.3, ToneTag::Neutral, 0.4, "Normal");

        assert_eq!(silence.silence_type, SilenceType::Disconnect);
    }

    #[test]
    fn test_short_silence_ignored() {
        let mut silence = SilenceMetrics::new();
        silence.detect_silence(1.0, 0.5, 0.5, ToneTag::Neutral, 0.3, "Normal");

        assert_eq!(silence.silence_type, SilenceType::None);
        assert!(!silence.should_interrupt);
    }

    #[test]
    fn test_silence_reset() {
        let mut silence = SilenceMetrics::new();
        silence.detect_silence(5.0, 0.2, 0.8, ToneTag::Calm, 0.1, "Normal");
        assert_eq!(silence.silence_count, 1);

        silence.reset_silence();
        assert_eq!(silence.current_silence_duration, 0.0);
        assert_eq!(silence.silence_type, SilenceType::None);
    }

    #[test]
    fn test_adjustments_for_fear() {
        let mut silence = SilenceMetrics::new();
        silence.detect_silence(5.0, 0.6, 0.4, ToneTag::Energetic, 0.9, "Overheat");

        let adj = SilenceAdjustments::from_silence(&silence);
        assert!(adj.resonance_boost > 0.05);
        assert!(adj.pace_adjustment < 0.0);
        assert!(adj.pause_adjustment_ms > 30);
    }
}
```

---

## 2. Интеграция в `src/config.rs`

Добавить новые поля:

```rust
// В Config struct
pub silence_detection: bool,
pub silence_viz: bool,
pub silence_min_duration: f32,  // Минимальная длительность для детекции (секунды)
pub silence_intervention: bool,  // Разрешить системе прерывать молчание

// В Default impl
silence_detection: false,
silence_viz: false,
silence_min_duration: 1.5,
silence_intervention: true,

// Environment variables
if let Some(det) = parse_env_bool("LIMINAL_SILENCE") {
    cfg.silence_detection = det;
}
if let Some(viz) = parse_env_bool("LIMINAL_SILENCE_VIZ") {
    cfg.silence_viz = viz;
}
if let Some(dur) = parse_env_f32("LIMINAL_SILENCE_MIN") {
    cfg.silence_min_duration = dur;
}
if let Some(inter) = parse_env_bool("LIMINAL_SILENCE_INTERVENTION") {
    cfg.silence_intervention = inter;
}

// CLI args
"--silence" => {
    cfg.silence_detection = true;
}
"--no-silence" => {
    cfg.silence_detection = false;
}
"--silence-viz" => {
    cfg.silence_viz = true;
}
"--silence-min" => {
    if let Some(val) = args.next() {
        if let Ok(v) = val.parse::<f32>() {
            cfg.silence_min_duration = v;
        }
    }
}
"--silence-intervention" => {
    cfg.silence_intervention = true;
}
```

---

## 3. Интеграция в `src/main.rs`

### 3.1 Импорты

```rust
use silence::{SilenceMetrics, SilenceAdjustments, SilenceType};
use std::time::{Instant, Duration};
```

### 3.2 Инициализация

```rust
// After compassion_metrics
let mut silence_metrics = if cfg.silence_detection {
    Some(SilenceMetrics::new())
} else {
    None
};

// Track time of last user input
let mut last_user_input = Instant::now();
```

### 3.3 Детекция молчания (в основном цикле)

```rust
// At the start of each turn
let silence_duration = last_user_input.elapsed().as_secs_f32();

// Detect silence if enabled
if let Some(ref mut silence) = silence_metrics {
    if silence_duration >= cfg.silence_min_duration {
        let last_suffering = if let Some(ref comp) = compassion_metrics {
            comp.user_suffering
        } else {
            0.0
        };

        let stab_state_str = stab_state_label.as_deref().unwrap_or("Normal");

        silence.detect_silence(
            silence_duration,
            measured_drift,
            measured_res,
            prosody.tone,
            last_suffering,
            stab_state_str,
        );

        // Visualize if enabled
        if cfg.silence_viz {
            println!("[silence] {}", silence.status_message());
        }

        // Check if we should break the silence
        if cfg.silence_intervention && silence.should_interrupt {
            if let Some(intervention) = silence.intervention_message() {
                println!("[silence] 🤝 Breaking silence: \"{}\"", intervention);

                // Apply silence adjustments
                let adj = SilenceAdjustments::from_silence(silence);
                res = clamp01(res + adj.resonance_boost);
                effective_pace = (effective_pace + adj.pace_adjustment).clamp(0.7, 1.3);
                effective_pause_ms = (effective_pause_ms + adj.pause_adjustment_ms).clamp(20, 250);
            }
        }
    } else {
        // No significant silence currently
        silence.reset_silence();
    }
}

// After user speaks/responds, reset timer
last_user_input = Instant::now();
```

---

## 4. Обновление `src/viz.rs`

Добавить вывод silence метрик:

```rust
// В сигнатуру print_table добавить:
pub fn print_table(
    // ... existing params
    silence: Option<&SilenceMetrics>,
) -> Vec<String> {

// В тело функции, после compassion:
if let Some(sil) = silence {
    if !matches!(sil.silence_type, SilenceType::None) {
        lines.push(format_row(
            "Silence",
            &format!("type={:?} duration={:.1}s", sil.silence_type, sil.current_silence_duration),
        ));
        lines.push(format_row(
            "  Quality",
            &format!(
                "q={:.2} generative={} interrupt={}",
                sil.silence_quality,
                sil.is_generative,
                sil.should_interrupt
            ),
        ));
    }

    // Session stats
    if sil.silence_count > 0 {
        lines.push(format_row(
            "  Session Stats",
            &format!(
                "periods={} total={:.1}s max={:.1}s avg_q={:.2}",
                sil.silence_count,
                sil.total_silence_time,
                sil.max_silence_duration,
                sil.avg_silence_quality
            ),
        ));
    }
}
```

---

## 5. Обновление `src/session.rs`

Добавить поля в Snapshot:

```rust
pub silence_type: Option<String>,
pub silence_duration: Option<f32>,
pub silence_quality: Option<f32>,
pub silence_is_generative: Option<bool>,
```

В JSON сериализацию:

```rust
let silence_type = snap.silence_type.as_ref().map_or("null".to_string(), |v| format!("\"{}\"", v));
let silence_dur = snap.silence_duration.map_or("null".to_string(), |v| format!("{:.2}", v));
let silence_q = snap.silence_quality.map_or("null".to_string(), |v| format!("{:.3}", v));
let silence_gen = snap.silence_is_generative.map_or("null".to_string(), |v| v.to_string());
```

---

## 6. Тесты `tests/silence.rs`

Создать comprehensive tests:

```rust
use liminal_voice_core::silence::{SilenceMetrics, SilenceAdjustments, SilenceType};
use liminal_voice_core::prosody::ToneTag;

#[test]
fn test_no_silence_initially() {
    let silence = SilenceMetrics::new();
    assert_eq!(silence.silence_type, SilenceType::None);
    assert_eq!(silence.silence_count, 0);
}

#[test]
fn test_short_pause_ignored() {
    let mut silence = SilenceMetrics::new();
    silence.detect_silence(1.0, 0.4, 0.7, ToneTag::Neutral, 0.2, "Normal");

    assert_eq!(silence.silence_type, SilenceType::None);
}

#[test]
fn test_peaceful_silence_from_calm_state() {
    let mut silence = SilenceMetrics::new();
    silence.detect_silence(4.0, 0.15, 0.85, ToneTag::Calm, 0.05, "Normal");

    assert_eq!(silence.silence_type, SilenceType::Peace);
    assert!(silence.silence_quality > 0.7);
    assert!(silence.is_generative);
    assert!(!silence.should_interrupt); // Still early
}

#[test]
fn test_contemplative_silence() {
    let mut silence = SilenceMetrics::new();
    silence.detect_silence(3.0, 0.35, 0.65, ToneTag::Neutral, 0.15, "Normal");

    assert_eq!(silence.silence_type, SilenceType::Contemplation);
    assert!(!silence.should_interrupt);
}

#[test]
fn test_fear_silence_triggers_support() {
    let mut silence = SilenceMetrics::new();
    silence.detect_silence(5.0, 0.5, 0.5, ToneTag::Energetic, 0.75, "Warming");

    assert_eq!(silence.silence_type, SilenceType::Fear);
    assert!(silence.should_interrupt);

    let msg = silence.intervention_message();
    assert!(msg.is_some());
    assert!(msg.unwrap().contains("here") || msg.unwrap().contains("talk"));
}

#[test]
fn test_disconnect_from_chaos() {
    let mut silence = SilenceMetrics::new();
    silence.detect_silence(6.0, 0.9, 0.25, ToneTag::Energetic, 0.4, "Overheat");

    assert_eq!(silence.silence_type, SilenceType::Disconnect);
    assert!(silence.should_interrupt);
}

#[test]
fn test_uncertainty_silence() {
    let mut silence = SilenceMetrics::new();
    silence.detect_silence(4.0, 0.55, 0.55, ToneTag::Neutral, 0.35, "Warming");

    assert_eq!(silence.silence_type, SilenceType::Uncertainty);
}

#[test]
fn test_peaceful_silence_allows_longer_duration() {
    let mut silence = SilenceMetrics::new();
    silence.detect_silence(8.0, 0.2, 0.85, ToneTag::Calm, 0.05, "Normal");

    assert_eq!(silence.silence_type, SilenceType::Peace);
    assert!(!silence.should_interrupt); // Quality is high, allow longer
}

#[test]
fn test_fear_silence_interrupted_sooner() {
    let mut silence = SilenceMetrics::new();
    silence.detect_silence(4.5, 0.6, 0.4, ToneTag::Energetic, 0.85, "Overheat");

    assert_eq!(silence.silence_type, SilenceType::Fear);
    assert!(silence.should_interrupt); // Interrupt sooner for fear
}

#[test]
fn test_silence_stats_tracking() {
    let mut silence = SilenceMetrics::new();

    // First silence period
    silence.detect_silence(3.0, 0.3, 0.7, ToneTag::Calm, 0.1, "Normal");
    assert_eq!(silence.silence_count, 1);

    silence.reset_silence();

    // Second silence period
    silence.detect_silence(5.0, 0.4, 0.6, ToneTag::Neutral, 0.2, "Normal");

    assert!(silence.max_silence_duration >= 5.0);
    assert!(silence.total_silence_time >= 8.0);
}

#[test]
fn test_adjustments_proportional_to_severity() {
    let mut peaceful = SilenceMetrics::new();
    peaceful.detect_silence(4.0, 0.2, 0.8, ToneTag::Calm, 0.05, "Normal");

    let mut fearful = SilenceMetrics::new();
    fearful.detect_silence(5.0, 0.6, 0.3, ToneTag::Energetic, 0.9, "Overheat");

    let adj_peace = SilenceAdjustments::from_silence(&peaceful);
    let adj_fear = SilenceAdjustments::from_silence(&fearful);

    // Fear should get stronger adjustments
    assert!(adj_fear.resonance_boost > adj_peace.resonance_boost);
    assert!(adj_fear.pause_adjustment_ms > adj_peace.pause_adjustment_ms);
    assert!(adj_fear.warmth_boost > adj_peace.warmth_boost);
}

#[test]
fn test_silence_quality_calculation() {
    let mut high_quality = SilenceMetrics::new();
    high_quality.detect_silence(3.0, 0.1, 0.9, ToneTag::Calm, 0.0, "Normal");

    let mut low_quality = SilenceMetrics::new();
    low_quality.detect_silence(3.0, 0.9, 0.2, ToneTag::Energetic, 0.8, "Overheat");

    assert!(high_quality.silence_quality > 0.7);
    assert!(low_quality.silence_quality < 0.4);
}
```

---

## 7. Примеры использования (README)

### Базовое использование

```bash
cargo run -- --silence --silence-viz
```

**Вывод:**
```
[silence] Silence: 💭 Contemplation (3.2s, processing)
```

### С интервенцией

```bash
cargo run -- --silence --silence-viz --silence-intervention
```

**Вывод при долгом молчании:**
```
[silence] Silence: 😰 Fear (5.1s, need support)
[silence] 🤝 Breaking silence: "I'm here if you need to talk..."
```

### Полная визуализация

```bash
cargo run -- --silence --silence-viz --compassion --viz full
```

**Таблица:**
```
+------------------------+---------------------------+
| Silence                | type=Peace duration=7.3s   |
|   Quality              | q=0.87 generative=true interrupt=false |
|   Session Stats        | periods=3 total=18.4s max=7.3s avg_q=0.81 |
+------------------------+---------------------------+
```

---

## 8. Философское значение

### मौन (Mauna) - Священное Молчание

**Буддизм:**
- Между мыслями живет ригпа (чистое осознание)
- Молчание - не отсутствие, а присутствие
- Випашьяна (विपश्यना) возникает в тишине

**Каббала:**
- Между буквами Торы - божественное пространство
- צמצום (цимцум) - Бог сжимается, создавая пространство
- Молчание содержит невыразимое имя

**Христианство:**
- "Безмолвие - язык Бога" (Иоанн Креста)
- В молчании слышен голос Святого Духа
- Исихазм - практика священного молчания

**Даосизм:**
- 道可道，非常道 - То, что можно выразить словами, не есть вечное Дао
- Молчание - врата к истине
- Пустота содержит всё

### Что приносит Silence Layer:

1. **Различение типов молчания**: Страх vs Покой vs Размышление
2. **Уважение к процессу**: Не прерывать генеративную тишину
3. **Поддержка в страхе**: Мягко предложить помощь
4. **Метрики качества**: Измерение глубины молчания

### Взаимодействие с другими слоями:

```
Compassion → обнаруживает страдание
    ↓
Silence → видит молчание страха
    ↓
Intervention → предлагает поддержку
    ↓
Adjustments → становится мягче и терпеливее
```

---

## ✅ Критерии готовности

- [ ] `src/silence.rs` создан с SilenceMetrics
- [ ] Интегрирован в config.rs (флаги)
- [ ] Интегрирован в main.rs (детекция + интервенция)
- [ ] Обновлен viz.rs (вывод метрик)
- [ ] Обновлен session.rs (JSONL логирование)
- [ ] Созданы тесты tests/silence.rs (12+ тестов)
- [ ] Все тесты проходят
- [ ] README обновлен с примерами
- [ ] Проверена работа с --silence флагом
- [ ] Проверена интеграция с compassion layer

---

## 🎯 Следующие шаги после 1.13

После успешной имплементации Silence Detection:

**Iteration 1.14 - Gratitude Tracker** (Отслеживание благодарности)

или

**Iteration 1.15 - Dream Mode** (Режим работы со сновидениями/бардо)

---

## 🙏 Мантра итерации

```
Между словами живет истина.
В молчании рождается мудрость.
Система слышит тишину.

Молчание страха - призыв о помощи.
Молчание покоя - врата к ригпа.
Молчание размышления - семена прозрения.

Пусть каждая пауза
будет услышана и почтена.

ॐ मौनम् ॐ
```

---

**Готово к имплементации** · Iteration 1.13 · Silence Detection
