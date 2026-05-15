"""Tests for golden-float Python bindings

φ² + 1/φ² = 3 | TRINITY
"""

import pytest
import numpy as np
from golden_float import (
    GF16, GF32,
    array_to_gf16, gf16_array_to_float, gf16_dot_product, gf16_normalize,
    phi, phi_gf16, phi_gf32, trinity_identity,
)


class TestGF16:
    """Test GF16 class"""

    def test_creation(self):
        phi = GF16(1.618)
        assert phi.to_float() == pytest.approx(1.618, rel=1e-3)

    def test_zero(self):
        zero = GF16(0.0)
        assert zero.is_zero()

    def test_bits(self):
        gf = GF16(1.0)
        bits = gf.bits()
        assert isinstance(bits, int)
        assert 0 <= bits <= 0xFFFF

    def test_arithmetic_add(self):
        a = GF16(1.0)
        b = GF16(2.0)
        c = a + b
        assert c.to_float() == pytest.approx(3.0, rel=1e-2)

    def test_arithmetic_sub(self):
        a = GF16(5.0)
        b = GF16(2.0)
        c = a - b
        assert c.to_float() == pytest.approx(3.0, rel=1e-2)

    def test_arithmetic_mul(self):
        a = GF16(3.0)
        b = GF16(4.0)
        c = a * b
        assert c.to_float() == pytest.approx(12.0, rel=1e-2)

    def test_arithmetic_div(self):
        a = GF16(10.0)
        b = GF16(2.0)
        c = a / b
        assert c.to_float() == pytest.approx(5.0, rel=1e-2)

    def test_negation(self):
        a = GF16(3.0)
        b = -a
        assert b.to_float() == pytest.approx(-3.0, rel=1e-2)

    def test_comparison(self):
        a = GF16(1.0)
        b = GF16(2.0)
        c = GF16(1.0)

        assert a < b
        assert b > a
        assert a <= b
        assert b >= a
        assert a == c
        assert a != b

    def test_hash(self):
        a = GF16(1.0)
        b = GF16(1.0)
        assert hash(a) == hash(b)

    def test_from_bits(self):
        original = GF16(1.0)
        bits = original.bits()
        reconstructed = GF16.from_bits(bits)
        assert reconstructed.bits() == bits

    def test_to_bytes(self):
        gf = GF16(1.0)
        bytes_val = gf.to_bytes()
        assert isinstance(bytes_val, bytes)
        assert len(bytes_val) == 2


class TestGF32:
    """Test GF32 class"""

    def test_creation(self):
        phi = GF32(1.618)
        assert phi.to_float() == pytest.approx(1.618, rel=1e-6)

    def test_zero(self):
        zero = GF32(0.0)
        assert zero.is_zero()

    def test_bits(self):
        gf = GF32(1.0)
        bits = gf.bits()
        assert isinstance(bits, int)
        assert 0 <= bits <= 0xFFFFFFFF

    def test_arithmetic(self):
        a = GF32(1.0)
        b = GF32(2.0)

        assert (a + b).to_float() == pytest.approx(3.0, rel=1e-6)
        assert (a - b).to_float() == pytest.approx(-1.0, rel=1e-6)
        assert (a * b).to_float() == pytest.approx(2.0, rel=1e-6)
        assert (b / a).to_float() == pytest.approx(2.0, rel=1e-6)

    def test_comparison(self):
        a = GF32(1.0)
        b = GF32(2.0)

        assert a < b
        assert b > a
        assert a <= b
        assert b >= a
        assert a == GF32(1.0)
        assert a != b


class TestNumPyOperations:
    """Test NumPy array operations"""

    def test_array_to_gf16(self):
        arr = np.array([1.0, 2.0, 3.0])
        gf16_arr = array_to_gf16(arr)
        assert gf16_arr.shape == (3,)
        assert all(isinstance(x, int) for x in gf16_arr)

    def test_gf16_array_to_float(self):
        gf16_arr = np.array([GF16(1.0).bits(), GF16(2.0).bits()])
        float_arr = gf16_array_to_float(gf16_arr)
        assert float_arr.shape == (2,)
        assert float_arr[0] == pytest.approx(1.0, rel=1e-2)
        assert float_arr[1] == pytest.approx(2.0, rel=1e-2)

    def test_gf16_dot_product(self):
        a = np.array([GF16(1.0).bits(), GF16(2.0).bits(), GF16(3.0).bits()])
        b = np.array([GF16(1.0).bits(), GF16(2.0).bits(), GF16(3.0).bits()])
        result = gf16_dot_product(a, b)
        # 1*1 + 2*2 + 3*3 = 14
        result_float = GF16.from_bits(result).to_float()
        assert result_float == pytest.approx(14.0, rel=1e-1)

    def test_gf16_normalize(self):
        arr = np.array([GF16(3.0).bits(), GF16(4.0).bits()])
        normalized = gf16_normalize(arr)
        # 3-4-5 triangle
        assert normalized.shape == (2,)

    def test_gf16_quantize_matrix(self):
        mat = np.array([[1.0, 2.0], [3.0, 4.0]])
        quantized = gf16_quantize_matrix(mat)
        assert quantized.shape == (2, 2)


class TestConstants:
    """Test mathematical constants"""

    def test_phi(self):
        assert phi() == pytest.approx(1.618033988749895)

    def test_phi_gf16(self):
        gf = phi_gf16()
        assert isinstance(gf, GF16)
        assert gf.to_float() == pytest.approx(1.618, rel=1e-3)

    def test_phi_gf32(self):
        gf = phi_gf32()
        assert isinstance(gf, GF32)
        assert gf.to_float() == pytest.approx(1.618, rel=1e-6)

    def test_trinity_identity(self):
        # φ² + φ⁻² = 3
        assert trinity_identity()


class TestFormatInfo:
    """Test format information functions"""

    def test_gf16_bias(self):
        from golden_float import gf16_bias
        assert gf16_bias() == 31

    def test_gf16_exp_bits(self):
        from golden_float import gf16_exp_bits
        assert gf16_exp_bits() == 6

    def test_gf16_mant_bits(self):
        from golden_float import gf16_mant_bits
        assert gf16_mant_bits() == 9

    def test_gf32_bias(self):
        from golden_float import gf32_bias
        assert gf32_bias() == 127


def test_version():
    """Test version is available"""
    from golden_float import __version__
    assert __version__ == "1.0.0"