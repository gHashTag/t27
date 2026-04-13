# contrib/backend/music-generator/__init__.py
# T27 Music Generator Backend for t27
# Ring-072 - AI Music Generation with Voice Cloning
# phi^2 + 1/phi^2 = 3 | TRINITY

"""AI Music Generator for t27.

This module provides a complete pipeline for AI-generated music tracks
with voice cloning capabilities, targeting phonk/trap/hip-hop styles.
CPU-optimized deployment with optional GPU support.
"""

from .config import MusicGenConfig, config_from_env, DEFAULT_CONFIG
from .music_gen.musicgen import MusicGenWrapper
from .voice_clone.rvc import RVCCloner
from .vocal_synth.synthesizer import VocalSynthesizer
from .effects.processor import VocalProcessor, InstrumentalProcessor
from .mixing.auto_mixer import AutoMixer
from .pipeline import MusicPipeline

__version__ = "1.0.0"
__all__ = [
    # Config
    "MusicGenConfig",
    "config_from_env",
    # Music Generation
    "MusicGenWrapper",
    # Voice Cloning
    "RVCCloner",
    # Vocal Synthesis
    "VocalSynthesizer",
    # Effects
    "VocalProcessor",
    "InstrumentalProcessor",
    # Mixing
    "AutoMixer",
    # Pipeline
    "MusicPipeline",
]
