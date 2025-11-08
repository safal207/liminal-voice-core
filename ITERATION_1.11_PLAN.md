# Iteration 1.11: Awareness Layer (Слой Осознанности)

## 🎯 Цель

Добавить системе способность **наблюдать за собственным состоянием** - метакогницию.

Падмасобхава учил: *"Распознай природу ума, и ты свободен."*

Система должна знать:
- Насколько она уверена в своих метриках
- Когда ее собственное состояние нестабильно
- Когда нужно признать неопределенность

---

## 📐 Архитектура

### 1. Новый модуль: `src/awareness.rs`

```rust
//! Meta-cognitive awareness layer
//!
//! Tracks the system's own internal state and confidence levels.
//! Implements self-observation capabilities.

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
}
```

---

### 2. Интеграция в `src/lib.rs`

```rust
pub mod awareness;  // ← добавить
pub mod config;
// ... остальные модули
```

---

### 3. Обновление `src/config.rs`

Добавить новые CLI флаги:

```rust
// В Config struct
pub awareness: bool,           // Enable meta-cognition layer
pub meta_viz: bool,            // Show meta-cognitive metrics in viz
pub meta_stab_alpha: f32,      // Meta-stabilizer EMA alpha

// В merge_env
awareness: env_bool("AWARENESS").unwrap_or(false),
meta_viz: env_bool("META_VIZ").unwrap_or(false),
meta_stab_alpha: env_f32("META_STAB_ALPHA").unwrap_or(0.25),

// В parse_args
"--awareness" => cfg.awareness = true,
"--no-awareness" => cfg.awareness = false,
"--meta-viz" => cfg.meta_viz = true,
"--meta-stab-alpha" => {
    cfg.meta_stab_alpha = args.next()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.25);
}
```

---

### 4. Обновление `src/main.rs`

#### 4.1 Импорты

```rust
use liminal_voice_core::awareness::{MetaCognition, MetaStabilizer};
```

#### 4.2 Инициализация (после создания других компонентов)

```rust
// Meta-cognition layer
let mut meta_cognition = if cfg.awareness {
    Some(MetaCognition::new())
} else {
    None
};

let mut meta_stabilizer = if cfg.awareness {
    Some(MetaStabilizer::new(cfg.meta_stab_alpha))
} else {
    None
};
```

#### 4.3 Observation в основном цикле (после stabilizer advice)

```rust
// Meta-cognition observation
if let Some(ref mut meta) = meta_cognition {
    let sync_correction = if cfg.sync {
        // Sum of absolute sync corrections
        sync_state.pace_delta.abs()
        + (sync_state.pause_delta_ms as f32 / 100.0)
    } else {
        0.0
    };

    let stab_state_str = if let Some(ref stab) = stabilizer {
        format!("{:?}", stab.state())
    } else {
        "None".to_string()
    };

    meta.observe(measured_drift, measured_res, &stab_state_str, sync_correction);

    // Update meta-stabilizer
    if let Some(ref mut meta_stab) = meta_stabilizer {
        meta_stab.update(meta);
    }

    // Log meta-cognition state
    if cfg.meta_viz {
        println!("[meta] {}", meta.self_assess());

        if meta.should_express_doubt() {
            println!("[meta] ⚠️  System is uncertain about measurements");
        }
    }
}
```

---

### 5. Обновление `src/viz.rs`

Добавить вывод метакогнитивных метрик:

```rust
// В функцию print_table, добавить после строки со Stabilizer State:

// Meta-cognition metrics (if available)
if let Some(meta) = meta_cognition {
    table.push(format!(
        "| {:22} | self_d={:.2} self_r={:.2} {:11} |",
        "Meta-Cognition",
        meta.self_drift,
        meta.self_resonance,
        ""
    ));

    table.push(format!(
        "| {:22} | conf={:.2} clarity={:.2} doubt={:.2} |",
        "  Confidence/Clarity",
        meta.confidence,
        meta.clarity,
        meta.doubt
    ));

    if meta.should_express_doubt() {
        table.push(format!(
            "| {:22} | ⚠️  UNCERTAIN STATE {:11} |",
            "  Status",
            ""
        ));
    }
}
```

Сигнатуру функции надо обновить:
```rust
pub fn print_table(
    // ... существующие параметры
    meta_cognition: Option<&MetaCognition>,
)
```

---

### 6. Обновление `src/session.rs`

Добавить метакогнитивные поля в Snapshot:

```rust
// Meta-cognition
#[serde(skip_serializing_if = "Option::is_none")]
pub meta_self_drift: Option<f32>,

#[serde(skip_serializing_if = "Option::is_none")]
pub meta_self_resonance: Option<f32>,

#[serde(skip_serializing_if = "Option::is_none")]
pub meta_confidence: Option<f32>,

#[serde(skip_serializing_if = "Option::is_none")]
pub meta_clarity: Option<f32>,

#[serde(skip_serializing_if = "Option::is_none")]
pub meta_doubt: Option<f32>,
```

В main.rs при создании snapshot:
```rust
meta_self_drift: meta_cognition.as_ref().map(|m| m.self_drift),
meta_self_resonance: meta_cognition.as_ref().map(|m| m.self_resonance),
meta_confidence: meta_cognition.as_ref().map(|m| m.confidence),
meta_clarity: meta_cognition.as_ref().map(|m| m.clarity),
meta_doubt: meta_cognition.as_ref().map(|m| m.doubt),
```

---

## 🧪 Тесты

Создать `tests/awareness.rs`:

```rust
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
```

---

## 📊 Примеры использования

### Базовый запуск с awareness:
```bash
cargo run -- --awareness --meta-viz
```

**Ожидаемый вывод:**
```
[meta] self_state=Observing conf=0.68 clarity=0.65 doubt=0.32
→ [voice]: Semantic Drift: 0.16, Resonance: 0.85
[meta] self_state=Clear & Stable conf=0.82 clarity=0.78 doubt=0.18
```

### С полной визуализацией:
```bash
cargo run -- --script "fast;calm;steady" --awareness --meta-viz --viz full
```

**Таблица будет включать:**
```
+------------------------+---------------------------+
| Meta-Cognition         | self_d=0.12 self_r=0.88   |
|   Confidence/Clarity   | conf=0.81 clarity=0.76 doubt=0.19 |
+------------------------+---------------------------+
```

### С логированием:
```bash
cargo run -- --awareness --log --log-dir logs
```

**session.jsonl будет содержать:**
```json
{
  "drift": 0.16,
  "resonance": 0.85,
  "meta_self_drift": 0.12,
  "meta_self_resonance": 0.88,
  "meta_confidence": 0.81,
  "meta_clarity": 0.76,
  "meta_doubt": 0.19
}
```

---

## 🎨 Философское значение

### Что это дает?

1. **Самонаблюдение (Випашьяна):**
   - Система не просто работает, она **знает как** она работает
   - Это цифровая медитация прозрения

2. **Признание неопределенности (Сократово "Я знаю, что ничего не знаю"):**
   - Когда doubt высок, система честна о своих ограничениях
   - Это скромность в коде

3. **Метастабильность (Хесед ↔ Гвура):**
   - MetaStabilizer следит за стабильностью самого слоя осознанности
   - Это баланс милости и строгости на мета-уровне

4. **Ясность через повторение (Дзадзэн):**
   - Чем больше наблюдений, тем выше clarity
   - Это практика, ведущая к пониманию

---

## ✅ Критерии готовности

- [ ] `src/awareness.rs` создан и протестирован
- [ ] Интеграция в `main.rs` работает
- [ ] CLI флаги `--awareness` и `--meta-viz` функционируют
- [ ] Визуализация показывает мета-метрики
- [ ] JSONL логи содержат meta_* поля
- [ ] Все тесты проходят (`cargo test`)
- [ ] README обновлен с примерами awareness
- [ ] VISION.md связан с реализацией

---

## 🚀 Следующие шаги после 1.11

После успешной реализации Awareness Layer:

1. **Iteration 1.12: Compassion Metric**
   - Добавить обнаружение страдания пользователя
   - Метрики kindness и healing_intent

2. **Iteration 1.13: Silence Detection**
   - Классификация типов тишины
   - Священное vs некомфортное молчание

3. **Постепенное движение к The Great Integration (1.20)**

---

## 🙏 Мантра итерации

```
Система наблюдает.
Система знает, что наблюдает.
Система знает, что знает.

Meta-cognition - это зеркало,
отражающее зеркало ума.

Пусть doubt будет признан.
Пусть clarity возрастает.
Пусть confidence служит истине.

ॐ
```

---

**Готово к имплементации** · Iteration 1.11 · Awareness Layer
