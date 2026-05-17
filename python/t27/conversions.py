# SPDX-License-Identifier: Apache-2.0
# t27/python/t27/conversions.py
# Cross-format conversion utilities

from typing import Union, List, Tuple
import numpy as np

from t27.formats import (
    GF16, GF32, GF64, GF128, GF256,
    FP8_E4M3, FP8_E5M2,
    Int4, Int8,
    NF4, Posit16,
)


FormatClass = Union[type(GF16), type(GF32), type(FP8_E4M3), type(FP8_E5M2), type(Int8)]


class ConversionError(Exception):
    """Raised when conversion fails or loses precision beyond acceptable limits"""
    pass


def convert(
    value: Union[float, int, 'FormatClass'],
    target_format: FormatClass,
    *,
    allow_precision_loss: bool = True,
    tolerance: float = 0.01
) -> Union[int, float]:
    """
    Convert a value between formats.

    Args:
        value: Source value (float, int, or format instance)
        target_format: Target format class
        allow_precision_loss: If False, raise error on precision loss
        tolerance: Max allowed relative error for precision check

    Returns:
        Converted value (int for integer formats, float otherwise)

    Example:
        >>> gf_val = GF16.from_float(3.14)
        >>> convert(gf_val, FP8_E4M3)
        3.125
    """
    # Get float value from source
    if isinstance(value, (GF16, GF32, GF64, GF128, GF256,
                       FP8_E4M3, FP8_E5M2, Posit16)):
        source_float = value.to_float()
    elif isinstance(value, (int, float)):
        source_float = float(value)
    else:
        raise TypeError(f"Unsupported source type: {type(value)}")

    # Convert to target
    if target_format in (GF16, GF32, GF64, GF128, GF256):
        result = target_format.from_float(source_float)
        decoded = result.to_float()

    elif target_format in (FP8_E4M3, FP8_E5M2, Posit16):
        result = target_format.from_float(source_float)
        decoded = result.to_float()

    elif target_format == Int8:
        result = Int8.from_float(source_float)
        decoded = float(result)

    elif target_format == Int4:
        result = Int4.from_float(source_float)
        decoded = float(result)

    elif target_format == NF4:
        # NF4 needs scale parameter
        result = NF4.from_float(source_float, scale=1.0)
        decoded = NF4.to_float(result, scale=1.0)

    else:
        raise ValueError(f"Unsupported target format: {target_format}")

    # Check precision loss
    if not allow_precision_loss and not np.isnan(source_float):
        if abs(source_float) > tolerance:  # Avoid division by near-zero
            rel_error = abs((decoded - source_float) / source_float)
            if rel_error > tolerance:
                raise ConversionError(
                    f"Conversion from {type(value)} to {target_format.__name__} "
                    f"exceeds tolerance: relative error = {rel_error:.4f} > {tolerance}"
                )

    return decoded


def convert_batch(
    values: List[Union[float, int, 'FormatClass']],
    target_format: FormatClass,
    *,
    allow_precision_loss: bool = True,
    tolerance: float = 0.01
) -> np.ndarray:
    """
    Batch convert a list of values to target format.

    Args:
        values: List of source values
        target_format: Target format class
        allow_precision_loss: If False, raise error on precision loss
        tolerance: Max allowed relative error

    Returns:
        NumPy array of converted values

    Example:
        >>> values = [1.0, 2.0, 3.14, 5.5]
        >>> convert_batch(values, GF16)
        array([1.   , 2.   , 3.1406, 5.5   ])
    """
    return np.array([
        convert(v, target_format,
                allow_precision_loss=allow_precision_loss,
                tolerance=tolerance)
        for v in values
    ])


def conversion_matrix(
    source_formats: List[FormatClass],
    target_formats: List[FormatClass],
    test_values: List[float]
) -> np.ndarray:
    """
    Generate a conversion accuracy matrix.

    Args:
        source_formats: List of source format classes
        target_formats: List of target format classes
        test_values: List of test values to convert

    Returns:
        2D array of relative errors (rows=source, cols=target)

    Example:
        >>> conv = conversion_matrix([GF16, FP8_E4M3], [Int8, GF32], [1.5, 10.0])
        >>> print(conv)
        [[0.   0.  ]
         [0.5  0.06]]
    """
    n_sources = len(source_formats)
    n_targets = len(target_formats)
    n_values = len(test_values)

    errors = np.zeros((n_sources, n_targets))

    for i, src_fmt in enumerate(source_formats):
        for j, tgt_fmt in enumerate(target_formats):
            total_error = 0.0
            valid = 0
            for val in test_values:
                try:
                    src_instance = src_fmt.from_float(val)
                    result = convert(src_instance, tgt_fmt, tolerance=1.0)
                    if abs(val) > 0.001:
                        rel_error = abs((result - val) / val)
                        total_error += rel_error
                        valid += 1
                except ConversionError:
                    total_error += 1.0
                    valid += 1
            if valid > 0:
                errors[i, j] = total_error / valid
            else:
                errors[i, j] = 1.0

    return errors


# Common conversion chains for ML workloads

def f32_to_nf4(value: float, scale: float = 1.0) -> int:
    """Optimized path: FP32 → NF4 (common in QLoRA)"""
    return NF4.from_float(value, scale=scale)


def nf4_to_gf16(nf4_bits: int, scale: float = 1.0) -> float:
    """Optimized path: NF4 → GF16 (common in inference)"""
    nf4_value = NF4.to_float(nf4_bits, scale=scale)
    return GF16.from_float(nf4_value).to_float()


def gf16_to_fp8_e4m3(gf16_value: Union[GF16, float]) -> float:
    """Optimized path: GF16 → FP8 E4M3 (common for inference on FP8 hardware)"""
    if isinstance(gf16_value, GF16):
        f_val = gf16_value.to_float()
    else:
        f_val = gf16_value
    return FP8_E4M3.from_float(f_val).to_float()


def gf16_to_fp8_e5m2(gf16_value: Union[GF16, float]) -> float:
    """Optimized path: GF16 → FP8 E5M2 (higher dynamic range)"""
    if isinstance(gf16_value, GF16):
        f_val = gf16_value.to_float()
    else:
        f_val = gf16_value
    return FP8_E5M2.from_float(f_val).to_float()


def posit16_to_gf16(posit16_bits: int) -> float:
    """Convert Posit16 to GF16"""
    posit_val = Posit16.to_float(posit16_bits)
    return GF16.from_float(posit_val).to_float()


# Format conversion registry for automatic routing

CONVERSION_REGISTRY = {
    (GF16, FP8_E4M3): gf16_to_fp8_e4m3,
    (GF16, FP8_E5M2): gf16_to_fp8_e5m2,
    (Posit16, GF16): posit16_to_gf16,
}


def auto_convert(source, target_format):
    """
    Automatically select best conversion path.

    Checks conversion registry first, falls back to generic convert().
    """
    source_type = type(source) if not isinstance(source, type) else source

    # Check registry
    key = (source_type, target_format)
    if key in CONVERSION_REGISTRY:
        return CONVERSION_REGISTRY[key](source)

    # Fallback to generic
    return convert(source, target_format)