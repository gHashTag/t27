# T27 Music Generator — Lightweight Version

**No ML dependencies! Pure NumPy implementation.**

**phi^2 + 1/phi^2 = 3 | TRINITY**

## Overview

Lightweight music generator using only NumPy (and optionally SciPy/soundfile).
Generates procedural beats, basslines, and melodies using mathematical patterns
based on the golden ratio.

## Advantages

- **No heavy dependencies** — only NumPy required (optional: SciPy, soundfile)
- **Fast generation** — < 1 second for 30s track
- **CPU-only** — No GPU needed
- **Small footprint** — ~500KB code

## Quick Start

```python
from lightweight import ProceduralGenerator, SimpleEffects, SimpleMixer

# Create generator
gen = ProceduralGenerator(sample_rate=48000)

# Generate beat
beat = gen.generate_beat(duration=30, bpm=140, style="phonk")

# Generate bassline
bass = gen.generate_bassline(duration=30, bpm=140, style="phonk")

# Mix and master
mixer = SimpleMixer(sample_rate=48000)
mixed = mixer.mix(bass, beat, vocal_level_db=-6)
mastered = mixer.master(mixed, target_lufs=-14)

# Save
gen.save(mastered, Path("track.wav"))
```

## Supported Styles

| Style | Description | BPM Range |
|-------|-------------|-----------|
| phonk | Dark drift phonk with cowbells and 808 | 130-150 |
| trap | Hard trap with rolling hi-hats | 130-150 |
| hip_hop | Boom bap hip-hop with samples | 80-100 |
| drill | UK drill with sliding 808s | 130-150 |
| lofi | Chill lofi with vinyl warmth | 70-90 |

## API Reference

### ProceduralGenerator

```python
gen = ProceduralGenerator(sample_rate=48000, seed=42)

# Generate drum beat
beat = gen.generate_beat(
    duration=30,
    bpm=140,
    style="phonk",
    kick_pattern=None,   # Optional custom 16-step pattern
    snare_pattern=None,
    hihat_pattern=None,
)

# Generate bassline
bass = gen.generate_bassline(
    duration=30,
    bpm=140,
    root_note=55.0,
    style="phonk",
)

# Generate melody
melody = gen.generate_melody(
    duration=30,
    bpm=140,
    scale="minor",  # minor, major, pentatonic
    style="ethereal",
)
```

### SimpleEffects

```python
fx = SimpleEffects(sample_rate=48000)

# Compression
compressed = fx.compress(
    audio,
    threshold_db=-20,
    ratio=4.0,
    attack_ms=5.0,
    release_ms=50.0,
)

# Reverb
with_reverb = fx.reverb(
    audio,
    room_size=0.5,
    decay=0.5,
    wet_level=0.3,
    dry_level=0.7,
)

# Delay
delayed = fx.delay(
    audio,
    delay_seconds=0.25,
    feedback=0.3,
    mix=0.3,
)

# Distortion
distorted = fx.distort(
    audio,
    drive_db=15.0,
    tone=0.5,
)

# EQ
equalized = fx.eq(
    audio,
    low_db=0.0,
    mid_db=0.0,
    high_db=0.0,
)

# Limiter
limited = fx.limit(audio, threshold_db=-0.3)
```

### SimpleMixer

```python
mixer = SimpleMixer(sample_rate=48000)

# Mix vocals with instrumental
mixed = mixer.mix(
    vocals,
    instrumental,
    vocal_level_db=-4.0,
    ducking=True,
    ducking_db=-3.0,
)

# Master track
mastered = mixer.master(
    mixed,
    target_lufs=-14.0,
    stereo_width=1.0,
)

# Analyze levels
info = mixer.analyze(audio)
# Returns: peak_db, rms_db, lufs_approx, dynamic_range

# Save to file
mixer.save(audio, Path("output.wav"))
```

## Dependencies

**Required:**
- NumPy >= 1.24.0

**Optional:**
- SciPy (for advanced filtering)
- Soundfile (for saving WAV files)

Install:
```bash
pip install numpy
# Optional:
pip install scipy soundfile
```

## Performance

| Operation | Duration | Samples |
|------------|----------|---------|
| Generate beat (30s) | ~0.1s | 1,440,000 |
| Generate bassline (30s) | ~0.05s | 1,440,000 |
| Generate melody (30s) | ~0.03s | 1,440,000 |
| Mix & master (60s) | ~0.1s | 2,880,000 |

**Total 30s track: ~0.3s generation time**

## Comparison with Full Version

| Feature | Full Version | Lightweight |
|---------|-------------|-------------|
| ML-based generation | MusicGen | Procedural |
| Voice cloning | RVC | Not included |
| Web UI | Gradio | Not included |
| Dependencies | torch, audiocraft, librosa, pedalboard, gradio | numpy (optional: scipy, soundfile) |
| Install size | ~2GB+ | ~10MB |
| Generation time (30s) | ~30-60s (CPU) | ~0.3s |

---

**phi^2 + 1/phi^2 = 3 | TRINITY**
