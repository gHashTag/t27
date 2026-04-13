# contrib/backend/music-generator/effects/__init__.py
# Audio Effects Module
# phi^2 + 1/phi^2 = 3 | TRINITY

from .processor import (
    VocalProcessor,
    InstrumentalProcessor,
    EffectChain,
)

__all__ = [
    "VocalProcessor",
    "InstrumentalProcessor",
    "EffectChain",
]
