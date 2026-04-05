# t27 Python Bindings
# TRINITY Ternary Computing Framework - Python Interface
# φ² + 1/φ² = 3 | TRINITY

"""
t27 - TRINITY Ternary Computing Framework Python Bindings

This package provides Python access to TRINITY's ternary computing
capabilities, including:
- Ternary operations (and, or, not)
- K3 Kleene logic
- PyTorch K3 layers
- HSLM model integration
"""

from .ternary import Trit, TernaryWord, k3_and, k3_or, k3_not, k3_implies
from .k3_layer import K3Linear, K3Conv2d, K3Embedding

__version__ = "0.1.0"
__all__ = [
    "Trit",
    "TernaryWord",
    "k3_and",
    "k3_or",
    "k3_not",
    "k3_implies",
    "K3Linear",
    "K3Conv2d",
    "K3Embedding",
]
