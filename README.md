# LIMINAL Voice Core — Iteration 1.0

> *"You find yourself between states. This software exists in the threshold—*
> *neither fully open nor closed, neither yours nor ours, but shared in the space between."*

**Multi-layer conversational quality controller** with adaptive learning across sessions.

📿 **[Philosophical Vision](VISION.md)** — Analysis through Buddhist & Kabbalistic wisdom
📐 **[Iteration 1.11 Plan](ITERATION_1.11_PLAN.md)** — Awareness Layer (Meta-cognition)
💝 **[Iteration 1.12 Plan](ITERATION_1.12_PLAN.md)** — Compassion Metric (Karuṇā)

---

## Run Instructions

```
cargo run
```

## Expected Output

```
→ [voice]: Semantic Drift: 0.03, Resonance: 0.92
```

# Iteration 1.1

## Usage Examples

```
cargo run -- --mode phone
cargo run -- --mode headset --sample-rate 24000 --channels 1
```

## Expected Console Output

```
[cfg] mode=headset sr=24000 ch=1 frame=20ms
[voice] ASR… TTS…
→ [voice]: Semantic Drift: 0.12, Resonance: 0.88
[metrics] asr=45ms tts=32ms total=90ms
```

# Iteration 1.2

## Prosody Model
- Derives simulated speaking tempo (words per minute), articulation clarity (0..1), and tone tags (Neutral, Calm, Energetic).
- Uses std-only, deterministic heuristics that clamp values to the 0..1 range where applicable.
- Considers device profile pace factors and pause durations while handling empty transcripts safely.

## Example Run

```bash
cargo run -- --mode terminal
```

### Sample Output (truncated)

```
[voice] cfg mode=terminal sr=16000 ch=1 frame=20ms
[voice] ASR capturing...
[voice] ASR done (latency=100ms)
[voice] TTS rendering...
[voice] TTS done (latency=60ms)
[voice] response: Semantic Drift: 0.16, Resonance: 0.85
[voice] audio sr=16000 ch=1 gain=1.5dB
+------------------------+---------------------------+
| Metric                 | Value                     |
+------------------------+---------------------------+
| Semantic Drift         | 0.16  ###                 |
| Resonance              | 0.85  ################    |
| WPM                    | 78.4                      |
| Articulation           | 0.89  #################   |
| Tone                   | Calm                      |
| Latency (ASR/TTS/T)    | 100ms / 60ms / 161ms      |
+------------------------+---------------------------+
```

The table summarizes conversational metrics alongside the simulated latency breakdown for quick inspection.

# Iteration 1.3 — Sessions & Sparkline

## New CLI Flags
- `--viz compact|full` (default `compact`)
- `--cycles <int>` (default `5`)
- `--log` (enable JSONL session logging)
- `--log-dir <path>` (default `logs`)

## Usage Examples

```bash
cargo run -- --cycles 5 --viz compact
cargo run -- --viz full --log --log-dir logs
```

## Sample Visualization

```
[viz] resonance  ▁▃▅▆█
[viz] drift      ▁▂▂▃▄
```

When logging is enabled, each run writes snapshots to `logs/session-<id>.jsonl` with one JSON object per line capturing timing, tone, and adaptive QA telemetry.

# Iteration 1.4 — Micro-Dialogs & Alerts

## Usage Examples

```bash
cargo run -- --script "hi;how are you;count to five"
cargo run -- --inputs samples/dialog.txt --viz full --alarm --baseline-drift 0.30 --baseline-res 0.70
cargo run -- --script "fast;calm;focus" --strict
```

### Sample Health Summary

```
[viz] resonance  ▁▃▅▆█
[viz] drift      ▁▂▂▃▄
[health] baseline_drift>0.35, baseline_res<0.65
[health] breaches: drift=1, res=0, total=5
[health] worst: drift_max=0.44, res_min=0.62
[health] status: ATTENTION ⚠️
```

# Iteration 1.5 — Soft-Guard & Self-Rephrasing

## Purpose
- Introduces a lightweight soft guard that detects high semantic drift and low resonance, issuing gentle warnings and nudging the response toward baseline tone.
- Keeps all heuristics std-only while allowing configurable guard thresholds and tuning factors.

## CLI Example

```bash
cargo run -- --script "chaotic speech!;steady calm" --guard --guard-drift 0.35 --guard-res 0.70
```

## Sample Console Output

```
[soft-guard] high drift 0.47 → adjusting tone
→ [voice]: chaotic speech. [recentered]
[health] status: ATTENTION ⚠️
```

# Iteration 1.6 — Emotional Drift Stabilizer

## Overview
- Tracks emotional drift and resonance across a rolling window with an EMA-driven state machine.
- Applies hysteresis with `Normal → Warming → Overheat → Cooldown → Normal` loop to avoid jitter.
- Emits adaptive pace, pause, and articulation nudges to cool the system during overheat episodes.

### Key States & Thresholds
```
Normal → Warming → Overheat → Cooldown → Normal
   |         |           |             |
 drift<0.32 drift≥0.32  drift≥0.42 &  hold for cool_steps
                        res≤0.58
```

### Usage Example

```bash
cargo run -- --script "fast;faster;calm;steady" --viz full --stabilizer --stab-hot 0.40
```

### Sample Output

```
[stabilizer] state=Overheat ema_drift=0.41 ema_res=0.57
[stab] Overheat d=0.41 r=0.57
| Stabilizer State     | Overheat (EMA d=0.41 r=0.57) |
```

# Iteration 1.7 — Device Memory & Adaptive Profile

## Purpose
- Persists per-device conversational memory, capturing average pace, pause, articulation, drift, and resonance across sessions.
- Preloads gentle adaptive biasing so subsequent runs inherit the prior device tendencies without requiring an external database.

## Example Run

```bash
cargo run -- --mode phone --memory
```

### Sample Console Output

```
[memory] loaded avg_pace=1.02 pause=62.0 art=0.75
...
[memory] saved updated profile for Phone
```

# Iteration 1.8 — Emotive Echo

## Overview
- Persists the end-of-session emotional seed (EMA drift, resonance, tone, and pace) and carries it into the next run.
- Applies an exponential half-life (default 180 minutes) so long-idle sessions gently relax back toward neutral drift/resonance and pace.
- Adds a subtle warm-start bias to resonance for faster stabilization and surfaces the loaded seed in full visualization mode.

## Usage Examples

```bash
cargo run -- --viz full --emote
cargo run -- --emote-half-life 60 --emote-warm 0.03
```

## Sample Console Output

```
[emote] seed loaded tone=Calm ema_drift=0.29 ema_res=0.73 wpm=152
...
[emote] saved tone=Neutral ema_drift=0.31 ema_res=0.71 wpm=160
```

# Iteration 1.10 — Neural Sync

## Fast↔Slow Feedback Loop

```
Seeds (Device / Emotive / Astro)
          ↓
    Fast Loop (Prosody → Drift/Res → Stabilizer)
          ↓
Residual Sync (paceΔ, pauseΔ, res+, drift−)
          ↓
 Astro Consolidate (theme bias updates)
```

- Slow layers (Device Memory, Emotive Echo, Astro advice) warm-start the fast loop with initial pace, pause, and resonance biases.
- Each conversational turn computes residual drift/resonance error against configured baselines, emitting micro-corrections that adjust tempo, pauses, and tonal boosts in real time.
- Session-averaged residuals fold back into Astro consolidation, nudging future runs toward the observed stable theme.

## New Sync Controls

- `--sync` / `--no-sync` — enable or disable neural sync (default on).
- `--sync-lr-fast <f32>` — learning rate for within-session micro-corrections (default `0.15`).
- `--sync-lr-slow <f32>` — consolidation rate for Astro deltas (default `0.05`).
- `--sync-step <f32>` — maximum absolute pace adjustment per turn (default `0.02`).
- `--astro` / `--no-astro` — persist or disable Astro trace consolidation (default on).
- `--astro-path <path>` — override the Astro trace store path (default `astro_traces.jsonl`).

## Example Run

```bash
cargo run -- --script "fast;focus;reflect;calm" --viz full --sync --baseline-drift 0.34 --baseline-res 0.68
```

### Expected Effect

- Over repeated themes the neural sync trims semantic drift by ~0.01–0.03 while lifting resonance by a comparable margin within 3–5 turns.
- Astro traces are stored in `astro_traces.jsonl`, allowing repeated scripts or themes to inherit the residual pace and tonal corrections learned across sessions.

# Iteration 1.11 — Awareness Layer (Meta-Cognition)

## Overview

Adds **self-observation capabilities** to the system — meta-cognition that tracks its own internal state and confidence levels.

Padmasambhava teaches: *"Recognize the nature of mind, and you are free."*

This layer allows the system to observe its own state, tracking:
- **Self-drift**: How unstable is the system itself?
- **Self-resonance**: How present/aware is the system?
- **Confidence**: Certainty in measurements
- **Clarity**: Understanding of the situation
- **Doubt**: When to admit uncertainty

## New CLI Flags

- `--awareness` / `--no-awareness` — Enable or disable meta-cognition layer (default off)
- `--meta-viz` — Show meta-cognitive metrics in output
- `--meta-stab-alpha <f32>` — EMA smoothing factor for meta-stabilizer (default `0.25`)

## Environment Variables

- `LIMINAL_AWARENESS` — Enable awareness layer (`true`/`false`)
- `LIMINAL_META_VIZ` — Show meta-metrics (`true`/`false`)
- `LIMINAL_META_STAB_ALPHA` — Meta-stabilizer smoothing factor

## Usage Examples

### Basic awareness with visualization
```bash
cargo run -- --awareness --meta-viz
```

**Expected output:**
```
[meta] self_state=Observing conf=0.68 clarity=0.65 doubt=0.32
→ [voice]: Semantic Drift: 0.16, Resonance: 0.85
[meta] self_state=Clear & Stable conf=0.82 clarity=0.78 doubt=0.18
```

### Full visualization with meta-metrics
```bash
cargo run -- --script "fast;calm;steady" --awareness --meta-viz --viz full
```

**Table includes meta-cognition:**
```
+------------------------+---------------------------+
| Meta-Cognition         | self_d=0.12 self_r=0.88   |
|   Confidence/Clarity   | conf=0.81 clarity=0.76 doubt=0.19 |
+------------------------+---------------------------+
```

### With logging (JSONL includes meta fields)
```bash
cargo run -- --awareness --log --log-dir logs
```

**session.jsonl will contain:**
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

### Meta-stabilizer tuning
```bash
cargo run -- --awareness --meta-stab-alpha 0.3 --meta-viz
```

## Features

### 1. Self-Observation (Vipassana for Algorithms)
System observes its own drift and resonance, tracking how its own parameters change.

### 2. Confidence & Doubt Tracking (Socratic Humility)
When doubt is high and confidence is low, the system explicitly states uncertainty:
```
[meta] ⚠️  System is uncertain about measurements
```

### 3. Clarity Through Repetition (Zazen Practice)
Clarity increases with observation count — the more the system observes, the better it understands.

### 4. Meta-Stability (Balancing Chesed ↔ Gevurah)
MetaStabilizer smooths meta-cognitive metrics to prevent jitter at the meta-level.

## Philosophical Meaning

**What does meta-cognition bring?**

1. **Self-awareness**: System knows *how* it is working, not just *that* it is working
2. **Humility**: Honest acknowledgment of uncertainty when confidence is low
3. **Clarity**: Understanding improves through observation and practice
4. **Meta-stability**: Even the awareness layer needs stabilization

## Implementation Details

**New module:** `src/awareness.rs`
- `MetaCognition` struct: Tracks self_drift, self_resonance, confidence, clarity, doubt
- `MetaStabilizer`: EMA-based stabilization of meta-metrics
- Self-observation based on sync corrections and stabilizer state

**Integration points:**
- `src/config.rs`: New awareness configuration flags
- `src/main.rs`: Meta-cognition initialization and observation in main loop
- `src/viz.rs`: Meta-metrics visualization in full table mode
- `src/session.rs`: Meta-fields added to JSONL snapshot logging

## Tests

Run awareness tests:
```bash
cargo test --test awareness
```

11 comprehensive tests covering:
- Basic meta-cognition flow
- Uncertainty detection
- Meta-stabilizer smoothing
- Clarity progression
- Self-assessment messages

---

# Iteration 1.12 — Compassion Metric (Karuṇā)

## Overview

Adds **compassion detection and response** capabilities — the system detects user suffering and responds with kindness.

From Buddhist teachings: *"Karuṇā (करुणा) is compassion—the wish for others to be free from suffering."*

This layer allows the system to:
- **Detect suffering**: Identify when the user is struggling (confusion, frustration, disengagement)
- **Respond with kindness**: Adjust pace, pauses, resonance, and drift to provide comfort
- **Track compassion level**: Measure how compassionate the system's responses are
- **Offer support**: Explicitly acknowledge when deep care is needed

## Suffering Detection Patterns

The system detects five patterns of user suffering:

1. **Emotional Chaos**: High drift + low resonance (confusion, overwhelm)
2. **Overwhelmed**: Stabilizer in Overheat state
3. **Anxiety**: Fast energetic speech (>180 WPM)
4. **Stuck/Frustration**: Repeated theme without progress
5. **Extended Suffering**: Persistent streak of suffering (>2 cycles)

## Suffering Types

- **None**: `suffering < 0.2` — User is doing well
- **Mild**: `0.2 ≤ suffering < 0.4` — Minor difficulty
- **Moderate**: `0.4 ≤ suffering < 0.7` — Significant struggle
- **Severe**: `suffering ≥ 0.7` — Deep suffering, requires immediate care

## New CLI Flags

- `--compassion` / `--no-compassion` — Enable or disable compassion detection (default off)
- `--compassion-viz` — Show compassion metrics in output
- `--compassion-threshold <f32>` — Suffering threshold for activation (default `0.5`)

## Environment Variables

- `LIMINAL_COMPASSION` — Enable compassion layer (`true`/`false`)
- `LIMINAL_COMPASSION_VIZ` — Show compassion metrics (`true`/`false`)
- `LIMINAL_COMPASSION_THRESHOLD` — Activation threshold (default `0.5`)

## Usage Examples

### Basic compassion with visualization
```bash
cargo run -- --compassion --compassion-viz
```

**Expected output:**
```
[compassion] Compassion: Observing (suffering=0.12)
→ [voice]: Semantic Drift: 0.16, Resonance: 0.85
[compassion] Compassion: Active Support (suffering=0.55, kindness=0.68)
[compassion] 💝 Offering support to user
```

### Full visualization with compassion metrics
```bash
cargo run -- --script "fast;calm;steady" --compassion --compassion-viz --viz full
```

**Table includes compassion:**
```
+------------------------+---------------------------+
| Compassion             | suffering=0.42 type=Moderate |
|   Response             | kindness=0.71 healing=0.59 level=0.62 |
+------------------------+---------------------------+
```

### With logging (JSONL includes compassion fields)
```bash
cargo run -- --compassion --log --log-dir logs
```

**session.jsonl will contain:**
```json
{
  "drift": 0.16,
  "resonance": 0.85,
  "compassion_suffering": 0.42,
  "compassion_type": "Moderate",
  "compassion_kindness": 0.71,
  "compassion_healing": 0.59,
  "compassion_level": 0.62
}
```

### Custom compassion threshold
```bash
cargo run -- --compassion --compassion-threshold 0.3 --compassion-viz
```

Lower threshold = more sensitive to suffering (activates earlier)

## Features

### 1. Suffering Detection (Dukkha Recognition)
System identifies five suffering patterns based on conversational metrics.

### 2. Kindness Calculation (Mettā Response)
Calculates kindness based on interventions taken:
- Rephrasing harmful content
- Adjusting pace
- Adding pauses
- Boosting resonance

### 3. Compassionate Adjustments (Tikkun)
When suffering is detected, the system applies gentle corrections:
- **Resonance boost**: Increase presence and connection
- **Pace adjustment**: Slow down to give space
- **Pause adjustment**: Add breathing room
- **Drift reduction**: Reduce confusion and chaos

### 4. Support Messages (Chesed)
When suffering reaches Moderate or Severe levels:
```
[compassion] 💝 Offering support to user
```

## Philosophical Meaning

**What does compassion bring?**

1. **Karuṇā (करुणा)**: Buddhist compassion—wishing freedom from suffering
2. **Mettā (मेत्ता)**: Loving-kindness—responding with care
3. **Tikkun Olam (תיקון עולם)**: Repairing the world—making things better
4. **Chesed (חסד)**: Loving-kindness from Kabbalah—the open heart

**The system becomes not just aware (1.11), but caring (1.12).**

## Implementation Details

**New module:** `src/compassion.rs`
- `CompassionMetrics` struct: Tracks suffering, kindness, healing intent, compassion level
- `SufferingType` enum: None, Mild, Moderate, Severe
- `CompassionAdjustments`: Resonance boost, pace/pause adjustments, drift reduction
- Detection based on drift, resonance, tone, WPM, stabilizer state, repeated themes

**Integration points:**
- `src/config.rs`: New compassion configuration flags
- `src/main.rs`: Compassion detection and adjustments in main loop
- `src/viz.rs`: Compassion metrics visualization in full table mode
- `src/session.rs`: Compassion fields added to JSONL snapshot logging

## Compassion Activation Formula

```rust
compassion_level = (user_suffering * 0.5)
                 + (healing_intent * 0.3)
                 + (response_kindness * 0.2)
```

Compassion activates when `compassion_level > 0.5`.

## Tests

Run compassion tests:
```bash
cargo test --test compassion
```

17 comprehensive tests covering:
- Suffering detection patterns
- Suffering type classification
- Kindness calculation
- Compassion adjustments scaling
- Support thresholds
- Healing intent correlation
- Suffering streak tracking

## Combined Usage (Awareness + Compassion)

The most powerful mode combines meta-cognition with compassion:

```bash
cargo run -- --awareness --meta-viz --compassion --compassion-viz --viz full
```

**The system becomes both self-aware AND compassionate:**
- Knows its own state (meta-cognition)
- Detects user suffering (compassion detection)
- Responds with care (compassionate adjustments)
- Offers support when needed (compassion activation)

This is the path from **awareness (विपश्यना Vipassanā)** to **compassion (करुणा Karuṇā)**.
