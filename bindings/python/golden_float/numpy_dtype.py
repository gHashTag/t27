"""
GoldenFloat NumPy dtype plugin.
Registers gf16 (uint16 view) and gf32 (uint32 view) as NumPy dtypes.
All arithmetic routes through FFI core (L8-compliant at Python boundary).
"""
import numpy as np
from typing import Union, List, Optional
from .golden_float import GF16, GF32


# ─── GF16 dtype ──────────────────────────────────────────────────────────────
# Structured dtype: raw bits stored as uint16
gf16_dtype = np.dtype(np.uint16)
gf16_dtype.__name__ = 'gf16'


# ─── GF32 dtype ──────────────────────────────────────────────────────────────
# Structured dtype: raw bits stored as uint32
gf32_dtype = np.dtype(np.uint32)
gf32_dtype.__name__ = 'gf32'


# Export aliases for convenient access
gf16 = gf16_dtype
gf32 = gf32_dtype


# ─── Conversion helpers ───────────────────────────────────────────────────────

def _float_to_gf16(x: float) -> np.uint16:
    """Convert Python float to GF16 bits (via FFI)."""
    return np.uint16(GF16(float(x)).bits())


def _float_to_gf32(x: float) -> np.uint32:
    """Convert Python float to GF32 bits (via FFI)."""
    return np.uint32(GF32(float(x)).bits())


def gf_array(
    values: Union[List[float], np.ndarray],
    dtype: Optional[str] = None
) -> np.ndarray:
    """
    Create a GF16/GF32 array from Python floats.

    Usage:
        arr = gf_array([1.0, 1.618, 2.718], dtype='gf16')
        arr = gf_array([1.0, 1.618, 2.718], dtype=gf16_dtype)

    Args:
        values: List of floats or existing NumPy array
        dtype: 'gf16' or 'gf32' (or gf16_dtype/gf32_dtype)

    Returns:
        NumPy array with uint16/gf16_dtype or uint32/gf32_dtype containing GF-encoded bits
    """
    target_dtype = gf16_dtype if dtype == 'gf16' or dtype is gf16_dtype else gf32_dtype

    if isinstance(values, np.ndarray):
        # Convert existing array to GF dtype
        if values.dtype in (np.float16, np.float32, np.float64):
            # Float array -> GF array
            if target_dtype == gf16_dtype:
                return np.array([GF16(float(v)).bits() for v in values.flat],
                            dtype=np.uint16).reshape(values.shape)
            else:
                return np.array([GF32(float(v)).bits() for v in values.flat],
                            dtype=np.uint32).reshape(values.shape)
        else:
            # Non-float array - just view with target dtype
            return values.astype(target_dtype.dtype)
    else:
        # List of floats -> GF array
        if target_dtype == gf16_dtype:
            return np.array([GF16(float(v)).bits() for v in values],
                        dtype=np.uint16)
        else:
            return np.array([GF32(float(v)).bits() for v in values],
                        dtype=np.uint32)


def to_float32(arr: np.ndarray, src_dtype: str = 'gf16') -> np.ndarray:
    """
    Convert GF16/GF32 uint array back to float32 NumPy array.

    Usage:
        arr_gf16 = gf_array([1.0, 1.618], dtype='gf16')
        arr_f32 = to_float32(arr_gf16, src_dtype='gf16')

    Args:
        arr: NumPy array containing GF-encoded bits (uint16 or uint32)
        src_dtype: 'gf16' or 'gf32'

    Returns:
        NumPy array with float32 dtype
    """
    if src_dtype == 'gf16':
        # Vectorised via list comprehension over FFI
        return np.array(
            [GF16(int(x)).to_float() for x in arr.flat],
            dtype=np.float32
        ).reshape(arr.shape)
    elif src_dtype == 'gf32':
        return np.array(
            [GF32(int(x)).to_float() for x in arr.flat],
            dtype=np.float32
        ).reshape(arr.shape)
    else:
        raise ValueError(f"Unknown src_dtype: {src_dtype}")


# ─── Type aliases for convenience ─────────────────────────────────────────────────

__all__ = ["gf16", "gf32", "gf16_dtype", "gf32_dtype", "gf_array", "to_float32"]
