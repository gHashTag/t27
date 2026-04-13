# contrib/backend/music-generator/lightweight/__init__.py
# Lightweight Music Generator (no ML dependencies)
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Lightweight music generation without heavy ML dependencies.

Provides procedural audio generation, effects processing, and mixing
using only numpy and scipy - suitable for CPU-only deployment.
"""

from .generative import ProceduralGenerator
from .effects import SimpleEffects
from .mixer import SimpleMixer

__version__ = "1.0.0-lightweight"
__all__ = [
    "ProceduralGenerator",
    "SimpleEffects",
    "SimpleMixer",
]
