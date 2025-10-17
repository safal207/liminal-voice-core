# LIMINAL Voice Core — Iteration 1.0

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

## Example Run

```bash
cargo run -- --script "fast;focus;reflect;calm" --viz full --sync --baseline-drift 0.34 --baseline-res 0.68
```

### Expected Effect

- Over repeated themes the neural sync trims semantic drift by ~0.01–0.03 while lifting resonance by a comparable margin within 3–5 turns.
