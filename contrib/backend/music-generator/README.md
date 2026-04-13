# T27 Music Generator

AI music generation module for t27 with voice cloning capabilities.

**phi^2 + 1/phi^2 = 3 | TRINITY**

## Overview

This module provides a complete pipeline for generating music tracks using AI, including:
- MusicGen-based instrumental generation
- Voice cloning with RVC interface
- Text-to-vocal synthesis
- Audio effects processing (pedalboard)
- Automated mixing and mastering
- Gradio web interface

**CPU-optimized deployment** with optional GPU support.

## Installation

```bash
cd contrib/backend/music-generator
pip install -r requirements.txt
```

## Quick Start

### Command Line

```bash
# Generate a phonk track with vocals
python pipeline.py --lyrics "Shadows creeping in the night" --style phonk --output track.wav

# Generate instrumental only
python pipeline.py --style trap --duration 30 --output beat.wav --skip-vocals
```

### Python API

```python
from music_generator import MusicPipeline

# Create pipeline
pipeline = MusicPipeline(device="cpu")

# Generate track
result = pipeline.generate(
    lyrics="Riding through the dark",
    style="phonk",
    output_path="output.wav",
    duration=60,
    vocal_style="aggressive",
)

# Access results
instrumental = result["instrumental"]
vocals = result["vocals"]
mastered = result["mastered"]
```

### Web Interface

```bash
python -m web_ui.app --host 0.0.0.0 --port 7860
```

Then open http://localhost:7860 in your browser.

## Supported Genres

| Genre | Description | BPM Range |
|-------|-------------|-----------|
| phonk | Dark drift phonk with cowbells and 808s | 130-150 |
| trap | Hard trap with rolling hi-hats | 130-150 |
| hip_hop | Boom bap hip-hop with samples | 80-100 |
| drill | UK drill with sliding 808s | 130-150 |
| lofi | Chill lofi with vinyl warmth | 70-90 |

## Vocal Styles

- **aggressive**: Punchy, in-your-face delivery
- **eerie**: Mysterious, haunting delivery
- **ethereal**: Dreamy, atmospheric delivery
- **smooth**: Natural, clean delivery
- **choppy**: Staccato, rhythmic delivery

## Configuration

Environment variables:

```bash
export MUSICGEN_MODEL_SIZE=small      # small, medium, large
export MUSICGEN_DEVICE=cpu            # cpu, cuda
export MUSICGEN_SAMPLE_RATE=48000
export MUSICGEN_VOCAL_LEVEL_DB=-4.0
```

Or use `config.yaml` for advanced configuration.

## Module Structure

```
music-generator/
├── music_gen/          # MusicGen integration
├── voice_clone/        # RVC voice cloning
├── vocal_synth/        # Text-to-vocal synthesis
├── effects/            # Audio effects (pedalboard)
├── mixing/             # Automated mixing
├── utils/              # Audio utilities
├── web_ui/             # Gradio interface
├── tests/              # Unit tests
├── pipeline.py         # CLI entry point
├── config.py           # Configuration
└── requirements.txt    # Dependencies
```

## Testing

```bash
# Run all tests
python -m pytest tests/ -v

# Run specific test file
python -m pytest tests/test_musicgen.py -v

# Run integration tests (requires models)
python -m pytest tests/ -v -m integration
```

## CPU Performance

Expected generation times (CPU only):

| Duration | Small Model | Medium Model |
|----------|-------------|--------------|
| 15s      | ~30s        | ~60s         |
| 30s      | ~60s        | ~2min        |
| 60s      | ~2min       | ~4min        |

*Times are approximate and depend on hardware.*

## License

This module is part of the t27 Trinity S3AI project.

**phi^2 + 1/phi^2 = 3 | TRINITY**
