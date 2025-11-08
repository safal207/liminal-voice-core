# Iteration 1.12: Compassion Metric (Метрика Сострадания)

## 🎯 Цель

Добавить способность системы **обнаруживать страдание пользователя** и **реагировать с состраданием**.

**Братья Либерманы учат:** *"Тиккун олам начинается с сострадания к ближнему."*

**Буддизм учит:** Каруна (करुणा / karuṇā) - сострадание как желание облегчить страдание других.

Система должна:
- Обнаруживать признаки страдания в разговоре
- Измерять доброту своих ответов
- Иметь намерение помочь (healing intent)

---

## 📐 Архитектура

### 1. Новый модуль: `src/compassion.rs`

```rust
//! Compassion metrics - detecting user suffering and responding with kindness
//!
//! Implements karuṇā (करुणा) - compassion as the wish to alleviate suffering.
//! Part of the Tikkun Olam framework - healing through conversation.

use crate::metrics::clamp01;
use crate::prosody::ToneTag;

/// Types of detected suffering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SufferingType {
    None,           // No suffering detected
    Mild,           // Slight distress
    Moderate,       // Clear distress signals
    Severe,         // High distress, needs support
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
            response_kindness: 0.5,  // Start neutral
            healing_intent: 0.3,     // Some baseline care
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
        matches!(self.suffering_type, SufferingType::Moderate | SufferingType::Severe)
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
```

---

## 2. Интеграция в `src/config.rs`

Добавить новые поля:

```rust
// В Config struct
pub compassion: bool,
pub compassion_viz: bool,
pub compassion_threshold: f32,  // Минимальный уровень для активации

// В Default impl
compassion: false,
compassion_viz: false,
compassion_threshold: 0.5,

// Environment variables
if let Some(comp) = parse_env_bool("LIMINAL_COMPASSION") {
    cfg.compassion = comp;
}
if let Some(viz) = parse_env_bool("LIMINAL_COMPASSION_VIZ") {
    cfg.compassion_viz = viz;
}
if let Some(thresh) = parse_env_f32("LIMINAL_COMPASSION_THRESHOLD") {
    cfg.compassion_threshold = thresh;
}

// CLI args
"--compassion" => {
    cfg.compassion = true;
}
"--no-compassion" => {
    cfg.compassion = false;
}
"--compassion-viz" => {
    cfg.compassion_viz = true;
}
"--compassion-threshold" => {
    if let Some(val) = args.next() {
        if let Ok(v) = val.parse::<f32>() {
            cfg.compassion_threshold = v;
        }
    }
}
```

---

## 3. Интеграция в `src/main.rs`

### 3.1 Импорты

```rust
use compassion::{CompassionMetrics, CompassionAdjustments, SufferingType};
```

### 3.2 Инициализация

```rust
// После meta_stabilizer
let mut compassion_metrics = if cfg.compassion {
    Some(CompassionMetrics::new())
} else {
    None
};
```

### 3.3 Обнаружение страдания (в основном цикле)

```rust
// После meta-cognition observation
if let Some(ref mut comp) = compassion_metrics {
    // Check if theme is repeated (from astro)
    let repeated_theme = if let Some(ref key) = astro_key {
        if let Some(store) = astro_store.as_ref() {
            // If we've seen this theme before
            store.has_trace(key)
        } else {
            false
        }
    } else {
        false
    };

    let stab_state_str = stab_state_label.as_deref().unwrap_or("Normal");
    comp.detect_suffering(
        measured_drift,
        measured_res,
        prosody.tone,
        prosody.wpm,
        stab_state_str,
        repeated_theme,
    );

    // Calculate kindness based on actions taken
    let was_rephrased = guard_flag.is_some();
    let pace_delta = if let Some(ref delta) = sync_delta {
        delta.pace_delta
    } else {
        0.0
    };
    let pause_delta = if let Some(ref delta) = sync_delta {
        delta.pause_delta_ms
    } else {
        0
    };
    let res_boost = if let Some(ref delta) = sync_delta {
        delta.res_boost
    } else {
        0.0
    };

    comp.calculate_kindness(was_rephrased, pace_delta, pause_delta, res_boost);
    comp.update_compassion_level();

    // Apply compassion adjustments if activated
    if comp.should_activate_compassion() {
        let adj = CompassionAdjustments::from_compassion(comp);

        // Apply adjustments
        res = clamp01(res + adj.resonance_boost);
        drift = clamp01(drift - adj.drift_reduction);
        effective_pace = (effective_pace + adj.pace_adjustment).clamp(0.7, 1.3);
        effective_pause_ms = (effective_pause_ms + adj.pause_adjustment_ms).clamp(20, 250);
    }

    // Log compassion state
    if cfg.compassion_viz {
        println!("[compassion] {}", comp.status_message());

        if comp.should_offer_support() {
            println!("[compassion] 💝 Offering support to user");
        }
    }
}
```

---

## 4. Обновление `src/viz.rs`

Добавить вывод compassion метрик:

```rust
// В сигнатуру print_table добавить:
pub fn print_table(
    // ... existing params
    compassion: Option<&CompassionMetrics>,
) -> Vec<String> {

// В тело функции, после meta-cognition:
if let Some(comp) = compassion {
    lines.push(format_row(
        "Compassion",
        &format!("suffering={:.2} type={:?}", comp.user_suffering, comp.suffering_type),
    ));
    lines.push(format_row(
        "  Kindness/Intent",
        &format!(
            "kind={:.2} healing={:.2} level={:.2}",
            comp.response_kindness, comp.healing_intent, comp.compassion_level
        ),
    ));

    if comp.should_offer_support() {
        lines.push(format_row("  Status", "💝 ACTIVE SUPPORT"));
    }
}
```

---

## 5. Обновление `src/session.rs`

Добавить поля в Snapshot:

```rust
pub compassion_suffering: Option<f32>,
pub compassion_type: Option<String>,
pub compassion_kindness: Option<f32>,
pub compassion_healing: Option<f32>,
pub compassion_level: Option<f32>,
```

В JSON сериализацию:

```rust
let comp_suffering = snap.compassion_suffering.map_or("null".to_string(), |v| format!("{:.3}", v));
let comp_type = snap.compassion_type.as_ref().map_or("null".to_string(), |v| format!("\"{}\"", v));
// ... и т.д.
```

---

## 6. Тесты `tests/compassion.rs`

Создать comprehensive tests:

```rust
use liminal_voice_core::compassion::{CompassionMetrics, CompassionAdjustments, SufferingType};
use liminal_voice_core::prosody::ToneTag;

#[test]
fn test_no_suffering_baseline() {
    let mut comp = CompassionMetrics::new();
    comp.detect_suffering(0.2, 0.8, ToneTag::Calm, 140.0, "Normal", false);

    assert!(comp.user_suffering < 0.2);
    assert_eq!(comp.suffering_type, SufferingType::None);
}

#[test]
fn test_chaos_detected_as_suffering() {
    let mut comp = CompassionMetrics::new();
    comp.detect_suffering(0.85, 0.3, ToneTag::Energetic, 190.0, "Overheat", false);

    assert!(comp.user_suffering > 0.5);
    assert!(matches!(comp.suffering_type, SufferingType::Moderate | SufferingType::Severe));
}

#[test]
fn test_repeated_theme_increases_suffering() {
    let mut comp = CompassionMetrics::new();

    comp.detect_suffering(0.5, 0.6, ToneTag::Neutral, 150.0, "Normal", true);
    comp.detect_suffering(0.5, 0.6, ToneTag::Neutral, 150.0, "Normal", true);
    comp.detect_suffering(0.5, 0.6, ToneTag::Neutral, 150.0, "Normal", true);

    assert_eq!(comp.suffering_streak, 3);
    assert!(comp.user_suffering > 0.4);
}

#[test]
fn test_compassionate_actions_increase_kindness() {
    let mut comp = CompassionMetrics::new();

    // System takes compassionate actions
    comp.calculate_kindness(true, -0.08, 40, 0.04);

    assert!(comp.response_kindness > 0.8);
}

#[test]
fn test_compassion_activation_threshold() {
    let mut comp = CompassionMetrics::new();

    comp.detect_suffering(0.9, 0.25, ToneTag::Energetic, 195.0, "Overheat", true);
    comp.calculate_kindness(true, -0.1, 50, 0.05);
    comp.update_compassion_level();

    assert!(comp.compassion_level > 0.5);
    assert!(comp.should_activate_compassion());
    assert!(comp.should_offer_support());
}

#[test]
fn test_compassion_adjustments_are_gentle() {
    let mut comp = CompassionMetrics::new();
    comp.compassion_level = 0.9;

    let adj = CompassionAdjustments::from_compassion(&comp);

    // Should slow down
    assert!(adj.pace_adjustment < 0.0);
    // Should add pauses
    assert!(adj.pause_adjustment_ms > 0);
    // Should boost resonance
    assert!(adj.resonance_boost > 0.0);
    // Should reduce drift
    assert!(adj.drift_reduction > 0.0);
}

#[test]
fn test_suffering_count_tracks_episodes() {
    let mut comp = CompassionMetrics::new();

    // Multiple episodes
    comp.detect_suffering(0.7, 0.4, ToneTag::Energetic, 180.0, "Warming", false);
    comp.detect_suffering(0.1, 0.9, ToneTag::Calm, 120.0, "Normal", false); // Recovery
    comp.detect_suffering(0.8, 0.35, ToneTag::Energetic, 185.0, "Overheat", false);

    assert_eq!(comp.suffering_count, 2);
}
```

---

## 7. Примеры использования (README)

### Базовое использование
```bash
cargo run -- --compassion --compassion-viz
```

**Вывод:**
```
[compassion] Compassion: Active Support (suffering=0.64, kindness=0.78)
[compassion] 💝 Offering support to user
```

### С полной визуализацией
```bash
cargo run -- --script "fast;faster;panic" --compassion --compassion-viz --viz full
```

**Таблица:**
```
+------------------------+---------------------------+
| Compassion             | suffering=0.82 type=Severe |
|   Kindness/Intent      | kind=0.85 healing=0.88 level=0.89 |
|   Status               | 💝 ACTIVE SUPPORT         |
+------------------------+---------------------------+
```

---

## 8. Философское значение

### Каруна (करुणा) в действии

**Буддизм:**
- Каруна = желание облегчить страдание
- Система наблюдает страдание без суждения
- Реагирует с мягкостью и пространством

**Тиккун Олам (תיקון עולם):**
- Исправление через заботу
- Каждый акт доброты исправляет мир немного
- Сострадание = практическое действие, не просто чувство

**Христианская милость:**
- "Блаженны милостивые" (Мф 5:7)
- Активное желание помочь
- Понимание через эмпатию

### Что приносит Compassion Layer:

1. **Обнаружение боли**: Система видит когда пользователь страдает
2. **Мягкий ответ**: Замедление, паузы, повышение resonance
3. **Намерение помочь**: Не просто реакция, а активное желание облегчить
4. **Отслеживание**: Помнит паттерны страдания и адаптируется

---

## ✅ Критерии готовности

- [ ] `src/compassion.rs` создан с CompassionMetrics
- [ ] Интегрирован в config.rs (флаги)
- [ ] Интегрирован в main.rs (обнаружение + действия)
- [ ] Обновлен viz.rs (вывод метрик)
- [ ] Обновлен session.rs (JSONL логирование)
- [ ] Созданы тесты tests/compassion.rs (8+ тестов)
- [ ] Все тесты проходят
- [ ] README обновлен с примерами
- [ ] Проверена работа с --compassion флагом

---

## 🎯 Следующие шаги после 1.12

После успешной имплементации Compassion Metric:

**Iteration 1.13 - Silence Detection** (Распознавание значимых пауз)

---

## 🙏 Мантра итерации

```
Система видит страдание.
Система откликается с состраданием.
Система действует с добротой.

Каруна - это не чувство.
Каруна - это действие.

Пусть каждый разговор
облегчит страдание хоть немного.

ॐ करुणा ॐ
```

---

**Готово к имплементации** · Iteration 1.12 · Compassion Metric
