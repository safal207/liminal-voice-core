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
