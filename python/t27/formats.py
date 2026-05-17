# SPDX-License-Identifier: Apache-2.0
# t27/python/t27/formats.py
# Number format implementations

import math
from typing import Union, Tuple
import numpy as np


PHI = (1 + math.sqrt(5)) / 2  # Golden ratio φ ≈ 1.618
PHI_INV = 1 / PHI  # 1/φ ≈ 0.618


def phi_ratio(exp_bits: int, mant_bits: int) -> float:
    """Calculate φ-ratio deviation from ideal (1/φ)"""
    if mant_bits == 0:
        return float('inf')
    return abs(exp_bits / mant_bits - PHI_INV)


class GF16:
    """GoldenFloat16: [S(1) | E(6) | M(9)], Bias = 31, φ-dist = 0.049"""

    BIAS = 31
    EXP_BITS = 6
    MANT_BITS = 9
    TOTAL_BITS = 16
    PHI_DISTANCE = phi_ratio(EXP_BITS, MANT_BITS)

    def __init__(self, bits: int = 0):
        self.bits = bits & 0xFFFF

    @property
    def sign(self) -> int:
        return (self.bits >> 15) & 0x1

    @property
    def exp(self) -> int:
        return (self.bits >> 9) & 0x3F

    @property
    def mant(self) -> int:
        return self.bits & 0x1FF

    def is_zero(self) -> bool:
        return self.exp == 0 and self.mant == 0

    def is_inf(self) -> bool:
        return self.exp == 63 and self.mant == 0

    def is_nan(self) -> bool:
        return self.exp == 63 and self.mant != 0

    @classmethod
    def from_float(cls, f: float) -> 'GF16':
        """Convert Python float to GF16"""
        if math.isnan(f):
            return cls(0xFE01)
        if f == 0.0:
            return cls(0x0000 if f >= 0 else 0x8000)
        if math.isinf(f):
            return cls(0x7E00 if f > 0 else 0xFE00)

        sign = 0 if f >= 0 else 1
        f = abs(f)

        # Calculate exponent
        exp = int(math.log2(f)) + cls.BIAS
        exp = max(1, min(62, exp))

        # Calculate mantissa
        mant = int((f / (2 ** (exp - cls.BIAS)) - 1) * (2 ** cls.MANT_BITS))
        mant = mant & ((1 << cls.MANT_BITS) - 1)

        return cls((sign << 15) | (exp << 9) | mant)

    def to_float(self) -> float:
        """Convert GF16 to Python float"""
        if self.is_zero():
            return 0.0
        if self.is_nan():
            return float('nan')
        if self.is_inf():
            return float('inf') if self.sign == 0 else float('-inf')

        exp_val = self.exp - self.BIAS
        mant_val = 1 + self.mant / (2 ** self.MANT_BITS)

        value = mant_val * (2 ** exp_val)
        return -value if self.sign else value

    def __repr__(self):
        if self.is_nan():
            return f"GF16(NaN)"
        if self.is_inf():
            return f"GF16({'+Inf' if self.sign == 0 else '-Inf'})"
        if self.is_zero():
            return f"GF16(0x{self.bits:04X})"
        return f"GF16(0x{self.bits:04X} ~ {self.to_float():.4g})"


class GF32:
    """GoldenFloat32: [S(1) | E(12) | M(19)], Bias = 2047, φ-dist = 0.013"""

    BIAS = 2047
    EXP_BITS = 12
    MANT_BITS = 19
    TOTAL_BITS = 32
    PHI_DISTANCE = phi_ratio(EXP_BITS, MANT_BITS)

    def __init__(self, bits: int = 0):
        self.bits = bits & 0xFFFFFFFF

    @property
    def sign(self) -> int:
        return (self.bits >> 31) & 0x1

    @property
    def exp(self) -> int:
        return (self.bits >> 19) & 0xFFF

    @property
    def mant(self) -> int:
        return self.bits & 0x7FFFF

    def is_zero(self) -> bool:
        return self.exp == 0 and self.mant == 0

    def is_inf(self) -> bool:
        return self.exp == 4095 and self.mant == 0

    def is_nan(self) -> bool:
        return self.exp == 4095 and self.mant != 0

    @classmethod
    def from_float(cls, f: float) -> 'GF32':
        if math.isnan(f):
            return cls(0xFFFFFE01)
        if f == 0.0:
            return cls(0x00000000 if f >= 0 else 0x80000000)
        if math.isinf(f):
            return cls(0x7FFF8000 if f > 0 else 0xFFFF8000)

        sign = 0 if f >= 0 else 1
        f = abs(f)

        exp = int(math.log2(f)) + cls.BIAS
        exp = max(1, min(4094, exp))

        mant = int((f / (2 ** (exp - cls.BIAS)) - 1) * (2 ** cls.MANT_BITS))
        mant = mant & ((1 << cls.MANT_BITS) - 1)

        return cls((sign << 31) | (exp << 19) | mant)

    def to_float(self) -> float:
        if self.is_zero():
            return 0.0
        if self.is_nan():
            return float('nan')
        if self.is_inf():
            return float('inf') if self.sign == 0 else float('-inf')

        exp_val = self.exp - self.BIAS
        mant_val = 1 + self.mant / (2 ** self.MANT_BITS)

        value = mant_val * (2 ** exp_val)
        return -value if self.sign else value


class GF64:
    """GoldenFloat64: [S(1) | E(24) | M(39)], Bias = 8388607, φ-dist = 0.013"""

    BIAS = 2 ** 23 - 1
    EXP_BITS = 24
    MANT_BITS = 39
    TOTAL_BITS = 64
    PHI_DISTANCE = phi_ratio(EXP_BITS, MANT_BITS)

    def __init__(self, bits: int = 0):
        self.bits = bits & 0xFFFFFFFFFFFFFFFF

    @classmethod
    def from_float(cls, f: float) -> 'GF64':
        # Simplified conversion
        if math.isnan(f):
            return cls(0xFFFFFE01FFFFFFFF)
        if f == 0.0:
            return cls(0 if f >= 0 else (1 << 63))
        # For full implementation, need 64-bit precision handling
        return cls(0)  # Placeholder

    def to_float(self) -> float:
        # Placeholder
        return 0.0


class GF128:
    """GoldenFloat128: [S(1) | E(48) | M(79)], Bias = 140737488355327, φ-dist = 0.010"""

    BIAS = 2 ** 47 - 1
    EXP_BITS = 48
    MANT_BITS = 79
    TOTAL_BITS = 128
    PHI_DISTANCE = phi_ratio(EXP_BITS, MANT_BITS)


class GF256:
    """GoldenFloat256: [S(1) | E(97) | M(158)], Bias = 7.92e37, φ-dist = 0.004"""

    BIAS = 2 ** 96 - 1
    EXP_BITS = 97
    MANT_BITS = 158
    TOTAL_BITS = 256
    PHI_DISTANCE = phi_ratio(EXP_BITS, MANT_BITS)


class FP8_E4M3:
    """FP8 E4M3 (OCP): [S(1) | E(4) | M(3)], Bias = 7"""

    BIAS = 7
    EXP_BITS = 4
    MANT_BITS = 3
    TOTAL_BITS = 8

    def __init__(self, bits: int = 0):
        self.bits = bits & 0xFF

    @classmethod
    def from_float(cls, f: float) -> 'FP8_E4M3':
        if math.isnan(f):
            return cls(0x7F)
        if f == 0.0:
            return cls(0x00 if f >= 0 else 0x80)
        if math.isinf(f):
            return cls(0x7C if f > 0 else 0xFC)

        sign = 0 if f >= 0 else 1
        f = abs(f)

        exp = int(math.log2(f)) + cls.BIAS
        exp = max(0, min(14, exp))

        mant = int((f / (2 ** (exp - cls.BIAS)) - 1) * 8) & 0x7

        return cls((sign << 7) | (exp << 3) | mant)

    def to_float(self) -> float:
        sign = -1 if (self.bits >> 7) else 1
        exp = (self.bits >> 3) & 0xF
        mant = self.bits & 0x7

        if exp == 0 and mant == 0:
            return 0.0
        if exp == 15:
            return float('inf') if mant == 0 else float('nan')

        return sign * (1 + mant / 8.0) * (2 ** (exp - 7))


class FP8_E5M2:
    """FP8 E5M2 (OCP): [S(1) | E(5) | M(2)], Bias = 15"""

    BIAS = 15
    EXP_BITS = 5
    MANT_BITS = 2
    TOTAL_BITS = 8

    def __init__(self, bits: int = 0):
        self.bits = bits & 0xFF

    @classmethod
    def from_float(cls, f: float) -> 'FP8_E5M2':
        if math.isnan(f):
            return cls(0x7F)
        if f == 0.0:
            return cls(0x00 if f >= 0 else 0x80)
        if math.isinf(f):
            return cls(0x7C if f > 0 else 0xFC)

        sign = 0 if f >= 0 else 1
        f = abs(f)

        exp = int(math.log2(f)) + cls.BIAS
        exp = max(0, min(30, exp))

        mant = int((f / (2 ** (exp - cls.BIAS)) - 1) * 4) & 0x3

        return cls((sign << 7) | (exp << 2) | mant)

    def to_float(self) -> float:
        sign = -1 if (self.bits >> 7) else 1
        exp = (self.bits >> 2) & 0x1F
        mant = self.bits & 0x3

        if exp == 0 and mant == 0:
            return 0.0
        if exp == 31:
            return float('inf') if mant == 0 else float('nan')

        return sign * (1 + mant / 4.0) * (2 ** (exp - 15))


class Int8:
    """Int8: signed 8-bit integer"""

    MIN = -128
    MAX = 127
    TOTAL_BITS = 8

    @classmethod
    def from_float(cls, f: float) -> int:
        return int(np.clip(f, cls.MIN, cls.MAX))

    @classmethod
    def to_float(cls, bits: int) -> float:
        if bits >= 128:
            return bits - 256
        return bits


class Int4:
    """Int4: signed 4-bit integer"""

    MIN = -8
    MAX = 7
    TOTAL_BITS = 4

    @classmethod
    def from_float(cls, f: float) -> int:
        return int(np.clip(f, cls.MIN, cls.MAX))

    @classmethod
    def to_float(cls, bits: int) -> float:
        if bits >= 8:
            return bits - 16
        return bits


class NF4:
    """NormalFloat4: 4-bit quantization for QLoRA"""

    # NF4 uses 16 discrete values symmetric around 0
    # Values are: -∞, ..., ∞ with specific quantization points
    TOTAL_BITS = 4
    LEVELS = 16

    @classmethod
    def from_float(cls, f: float, scale: float = 1.0) -> int:
        """Quantize float to NF4 with given scale"""
        # Symmetric quantization with 16 levels
        scaled = f / scale
        # Clamp to valid range
        scaled = np.clip(scaled, -7.5, 7.5)
        # Round to nearest integer
        return int(np.round(scaled)) & 0xF

    @classmethod
    def to_float(cls, bits: int, scale: float = 1.0) -> float:
        """Dequantize NF4 to float"""
        if bits >= 8:
            return (bits - 16) * scale
        return bits * scale


class Posit16:
    """Posit16: posit type 16, unum 1.0"""

    TOTAL_BITS = 16
    ES = 1  # exponent size (unum)
    USEED = 2  # useed = 2^ES

    @classmethod
    def from_float(cls, f: float) -> int:
        """Convert float to Posit16 (simplified)"""
        if math.isnan(f):
            return 0x7FFF  # NaR
        if f == 0.0:
            return 0x0000
        if math.isinf(f):
            return 0x7FFF if f > 0 else 0x7FFF

        sign = 0 if f >= 0 else 1
        f = abs(f)

        # Find regime
        exp_val = int(math.log2(f))
        regime = abs(exp_val) // 2  # Simplified

        # Build posit (very simplified)
        bits = sign << 15
        if exp_val >= 0:
            bits |= ((1 << (14 - min(regime, 7))) - 1) << (14 - min(regime, 7))
        else:
            bits |= ((1 << min(regime + 1, 8)) - 1) << (14 - min(regime + 1, 8))

        return bits

    @classmethod
    def to_float(cls, bits: int) -> float:
        """Convert Posit16 to float (simplified)"""
        if bits == 0:
            return 0.0
        if bits == 0x7FFF:
            return float('nan')

        sign = -1 if (bits >> 15) else 1
        # Simplified decoding
        magnitude = (bits & 0x7FFF) / 16384.0 * 128.0
        return sign * magnitude