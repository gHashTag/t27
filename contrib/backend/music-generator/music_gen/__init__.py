# contrib/backend/music-generator/music_gen/__init__.py
# Music Generation Module
# phi^2 + 1/phi^2 = 3 | TRINITY

from .musicgen import MusicGenWrapper
from .stable_audio import StableAudioGenerator
from .bark import BarkGenerator
from .prompts import (
    PHONK_PROMPTS,
    TRAP_PROMPTS,
    HIP_HOP_PROMPTS,
    DRILL_PROMPTS,
    LOFI_PROMPTS,
    enhance_prompt_for_genre,
)

__all__ = [
    "MusicGenWrapper",
    "StableAudioGenerator",
    "BarkGenerator",
    "PHONK_PROMPTS",
    "TRAP_PROMPTS",
    "HIP_HOP_PROMPTS",
    "DRILL_PROMPTS",
    "LOFI_PROMPTS",
    "enhance_prompt_for_genre",
]
