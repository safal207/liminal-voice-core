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
