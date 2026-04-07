"""
GoldenFloat: Phi-structured floating-point formats for ML and scientific computing.

GoldenFloat formats are phi-optimal floating-point representations that
provide better precision for constants like φ (golden ratio) compared to
standard IEEE 754 formats.

Available formats:
- GF4: 4-bit, ultra-compact
- GF8: 8-bit, minimal precision
- GF12: 12-bit, embedded systems
- GF16: 16-bit, primary format (replaces bfloat16)
- GF20: 20-bit, middle ground
- GF24: 24-bit, high precision
- GF32: 32-bit, full precision (replaces float32)
"""

from .golden_float import GF16, GF32  # noqa: F401

__all__ = ["GF16", "GF32"]
__version__ = "0.1.0"
