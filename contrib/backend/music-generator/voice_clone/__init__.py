# contrib/backend/music-generator/voice_clone/__init__.py
# Voice Cloning Module
# phi^2 + 1/phi^2 = 3 | TRINITY

from .rvc import RVCCloner
from .preprocessing import (
    preprocess_audio,
    reduce_noise,
    segment_by_vocal_activity,
    normalize_audio,
)
from .train import VoiceTrainer

__all__ = [
    "RVCCloner",
    "preprocess_audio",
    "reduce_noise",
    "segment_by_vocal_activity",
    "normalize_audio",
    "VoiceTrainer",
]
