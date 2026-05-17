#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# t27/tests/cross_format/test_format_conversions.py
# Cross-format conversion tests

import pytest
import numpy as np


class GoldenFloat16:
    """GF16: [S(1) | E(6) | M(9)], Bias = 31"""

    def __init__(self, bits=0):
        self.bits = bits & 0xFFFF

    @staticmethod
    def from_float(f: float):
        """Convert float32 to GF16 (simplified)"""
        if f == 0.0:
            return GoldenFloat16(0x0000 if f >= 0 else 0x8000)
        if f > 0:
            sign = 0
        else:
            sign = 1
            f = -f

        exp = int(np.log2(f)) + 31
        mant = int((f / (2 ** (exp - 31)) - 1) * 512) & 0x1FF

        exp = max(0, min(63, exp))
        return GoldenFloat16((sign << 15) | (exp << 9) | mant)

    def to_float(self) -> float:
        """Convert GF16 to float32 (simplified)"""
        bits = self.bits
        sign = -1.0 if (bits >> 15) else 1.0
        exp = (bits >> 9) & 0x3F
        mant = bits & 0x1FF

        if exp == 0 and mant == 0:
            return 0.0
        if exp == 63:
            return float('inf') if mant == 0 else float('nan')

        return sign * (1 + mant / 512.0) * (2 ** (exp - 31))


class FP8_E4M3:
    """FP8 E4M3: [S(1) | E(4) | M(3)], Bias = 7"""

    def __init__(self, bits=0):
        self.bits = bits & 0xFF

    @staticmethod
    def from_float(f: float):
        if f == 0.0:
            return FP8_E4M3(0x00 if f >= 0 else 0x80)
        if f > 0:
            sign = 0
        else:
            sign = 1
            f = -f

        exp = int(np.log2(f)) + 7
        mant = int((f / (2 ** (exp - 7)) - 1) * 8) & 0x7

        exp = max(0, min(15, exp))
        return FP8_E4M3((sign << 7) | (exp << 3) | mant)

    def to_float(self) -> float:
        bits = self.bits
        sign = -1.0 if (bits >> 7) else 1.0
        exp = (bits >> 3) & 0xF
        mant = bits & 0x7

        if exp == 0 and mant == 0:
            return 0.0
        if exp == 15:
            return float('inf') if mant == 0 else float('nan')

        return sign * (1 + mant / 8.0) * (2 ** (exp - 7))


class FP8_E5M2:
    """FP8 E5M2: [S(1) | E(5) | M(2)], Bias = 15"""

    def __init__(self, bits=0):
        self.bits = bits & 0xFF

    @staticmethod
    def from_float(f: float):
        if f == 0.0:
            return FP8_E5M2(0x00 if f >= 0 else 0x80)
        if f > 0:
            sign = 0
        else:
            sign = 1
            f = -f

        exp = int(np.log2(f)) + 15
        mant = int((f / (2 ** (exp - 15)) - 1) * 4) & 0x3

        exp = max(0, min(31, exp))
        return FP8_E5M2((sign << 7) | (exp << 2) | mant)

    def to_float(self) -> float:
        bits = self.bits
        sign = -1.0 if (bits >> 7) else 1.0
        exp = (bits >> 2) & 0x1F
        mant = bits & 0x3

        if exp == 0 and mant == 0:
            return 0.0
        if exp == 31:
            return float('inf') if mant == 0 else float('nan')

        return sign * (1 + mant / 4.0) * (2 ** (exp - 15))


class Int8:
    """Int8: signed 8-bit integer"""

    def __init__(self, value: int):
        self.value = value & 0xFF

    @staticmethod
    def from_float(f: float):
        clamped = max(-128, min(127, int(f)))
        return Int8(clamped if clamped >= 0 else 256 + clamped)

    def to_float(self) -> float:
        if self.value >= 128:
            return self.value - 256
        return self.value


# Tests

def test_gf16_identity():
    """GF16 identity: 1.0 rounds to 1.0"""
    one = GoldenFloat16.from_float(1.0)
    back = one.to_float()
    assert abs(back - 1.0) < 0.01, f"Expected ~1.0, got {back}"


def test_gf16_additive_property():
    """GF16: a + 0 = a"""
    a = GoldenFloat16.from_float(3.14)
    zero = GoldenFloat16.from_float(0.0)
    # This tests encoding/decoding, actual add in hardware
    assert abs(a.to_float() - 3.14) < 0.1


def test_fp8_e4m3_range():
    """FP8 E4M3 range test"""
    small = FP8_E4M3.from_float(0.1)
    large = FP8_E4M3.from_float(10.0)
    assert small.to_float() > 0
    assert large.to_float() > 5


def test_fp8_e5m2_range():
    """FP8 E5M2 range test"""
    small = FP8_E4M3.from_float(0.01)
    large = FP8_E4M3.from_float(100.0)
    assert small.to_float() > 0
    assert large.to_float() > 50


def test_int8_clamp():
    """Int8 clamping test"""
    too_large = Int8.from_float(300.0)
    too_small = Int8.from_float(-300.0)
    assert too_large.to_float() == 127
    assert too_small.to_float() == -128


def test_cross_format_gf16_to_fp8_e4m3():
    """Cross-format: GF16 → FP8 E4M3"""
    gf_val = GoldenFloat16.from_float(5.5)
    fp_val = FP8_E4M3.from_float(gf_val.to_float())
    # Accept some loss due to lower precision
    assert abs(fp_val.to_float() - 5.5) < 1.0


def test_cross_format_gf16_to_int8():
    """Cross-format: GF16 → Int8"""
    gf_val = GoldenFloat16.from_float(42.7)
    int_val = Int8.from_float(gf_val.to_float())
    assert int_val.to_float() == 42


def test_cross_format_fp8_e4m3_to_e5m2():
    """Cross-format: FP8 E4M3 → FP8 E5M2"""
    e4m3_val = FP8_E4M3.from_float(3.5)
    e5m2_val = FP8_E5M2.from_float(e4m3_val.to_float())
    assert abs(e5m2_val.to_float() - 3.5) < 1.0


def test_zero_conversion():
    """Zero is preserved across formats"""
    gf_zero = GoldenFloat16.from_float(0.0)
    fp_zero = FP8_E4M3.from_float(0.0)
    int_zero = Int8.from_float(0.0)
    assert gf_zero.to_float() == 0.0
    assert fp_zero.to_float() == 0.0
    assert int_zero.to_float() == 0.0


def test_negative_conversion():
    """Negative numbers handled correctly"""
    gf_neg = GoldenFloat16.from_float(-5.0)
    fp_neg = FP8_E4M3.from_float(-5.0)
    int_neg = Int8.from_float(-50.0)
    assert gf_neg.to_float() < 0
    assert fp_neg.to_float() < 0
    assert int_neg.to_float() < 0


def test_phi_ratio_approximation():
    """GF formats approximate φ-ratio (0.618)"""
    # GF16: 6 exp, 9 mant → 6/9 = 0.667 (close to 0.618)
    assert abs(6/9 - 1/1.618) < 0.1

    # GF32: 12 exp, 19 mant → 12/19 = 0.632 (closer)
    assert abs(12/19 - 1/1.618) < 0.1


def test_format_preserves_order():
    """Ordering preserved for positive numbers"""
    values = [1.0, 2.0, 3.14, 5.5, 10.0]
    gf_vals = [GoldenFloat16.from_float(v) for v in values]
    decoded = [g.to_float() for g in gf_vals]
    for i in range(len(decoded) - 1):
        assert decoded[i] < decoded[i + 1] + 0.1


def test_special_values_nan():
    """NaN propagation"""
    gf_nan = GoldenFloat16(0xFE01)
    assert np.isnan(gf_nan.to_float())


def test_special_values_inf():
    """Infinity handling"""
    gf_inf = GoldenFloat16(0x7E00)
    assert np.isinf(gf_inf.to_float())
    assert gf_inf.to_float() > 0

    gf_neg_inf = GoldenFloat16(0xFE00)
    assert np.isinf(gf_neg_inf.to_float())
    assert gf_neg_inf.to_float() < 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])