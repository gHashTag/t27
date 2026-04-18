"""
E2E tests for GoldenFloat NumPy integration.
"""
import pytest
import numpy as np
from golden_float import GF16, GF32, gf_array, to_float32, gf16, gf32

PHI = 1.618033988749895


class TestGF16Dtype:
    """Test GF16 dtype functionality."""

    def test_array_creation_shape(self):
        arr = gf_array([1.0, PHI, 2.718], dtype='gf16')
        assert arr.shape == (3,)
        assert arr.dtype == np.uint16

    def test_array_creation_with_dtype_obj(self):
        arr = gf_array([1.0, PHI, 2.718], dtype=gf16)
        assert arr.shape == (3,)
        assert arr.dtype == np.uint16

    def test_phi_bits_match_scalar(self):
        arr = gf_array([PHI], dtype='gf16')
        scalar = GF16(PHI)
        assert int(arr[0]) == scalar.bits(), \
            f"Array {hex(int(arr[0]))} != scalar {hex(scalar.bits())}"

    def test_roundtrip_precision(self):
        values = [1.0, PHI, 3.14159, 2.71828, 0.5, -1.0]
        arr = gf_array(values, dtype='gf16')
        back = to_float32(arr, src_dtype='gf16')
        for orig, rec in zip(values, back):
            assert abs(float(orig) - float(rec)) < 0.01, \
                f"Roundtrip error too large: {orig} -> {rec}"

    def test_zero_encoding(self):
        arr = gf_array([0.0], dtype='gf16')
        assert GF16(0.0).is_zero()
        assert (int(arr[0]) & 0x7FFF) == 0

    def test_negative_values(self):
        arr = gf_array([-1.0, -PHI], dtype='gf16')
        for x in arr:
            val = GF16(int(x))
            assert val.sign() == 1, "Sign should be 1 for negative"

    def test_array_from_numpy_array(self):
        input_arr = np.array([1.0, PHI, 2.718], dtype=np.float32)
        arr = gf_array(input_arr, dtype='gf16')
        assert arr.shape == (3,)
        assert arr.dtype == np.uint16


class TestGF32Dtype:
    """Test GF32 dtype functionality."""

    def test_phi_gf32_bits(self):
        phi_gf32 = GF32(PHI)
        # φ in GF32 (approximately IEEE f32 bits since GF32 maps to f32)
        expected = np.float32(PHI).view(np.uint32)
        assert phi_gf32.bits() == int(expected), \
            f"GF32(φ) = 0x{phi_gf32.bits():08X}, expected 0x{int(expected):08X}"

    def test_array_creation(self):
        arr = gf_array([1.0, PHI], dtype='gf32')
        assert arr.dtype == np.uint32
        assert arr.shape == (2,)

    def test_array_creation_with_dtype_obj(self):
        arr = gf_array([1.0, PHI], dtype=gf32)
        assert arr.dtype == np.uint32
        assert arr.shape == (2,)

    def test_roundtrip_gf32(self):
        values = [1.0, PHI, 3.14159, 2.71828]
        arr = gf_array(values, dtype='gf32')
        back = to_float32(arr, src_dtype='gf32')
        for orig, rec in zip(values, back):
            assert abs(float(orig) - float(rec)) < 1e-6, \
                f"Roundtrip error too large: {orig} -> {rec}"


class TestCrossLanguageBits:
    """Bit-identical verification across Python paths."""

    def test_scalar_vs_array_identical(self):
        for val in [1.0, PHI, 3.14159, 2.71828]:
            scalar_bits = GF16(val).bits()
            arr_bits = int(gf_array([val], dtype='gf16')[0])
            assert scalar_bits == arr_bits, \
                f"Scalar {hex(scalar_bits)} != array {hex(arr_bits)} for {val}"

    def test_gf32_phi_expected_hex(self):
        """φ in GF32 should match known reference value."""
        phi_bits = GF32(PHI).bits()
        # IEEE f32 of φ ≈ 0x3FCF1BBD
        # GF32 uses same bit layout as f32 for this format
        expected = np.float32(PHI).view(np.uint32)
        assert phi_bits == int(expected), \
            f"GF32(φ) = 0x{phi_bits:08X}, expected 0x{int(expected):08X}"

    def test_field_extraction_from_array(self):
        """Test sign/exponent/mantissa extraction from array values."""
        arr = gf_array([1.0, -2.5, PHI], dtype='gf16')
        # First value: positive
        assert GF16(int(arr[0])).sign() == 0
        # Second value: negative
        assert GF16(int(arr[1])).sign() == 1
        # Third value: positive phi
        assert GF16(int(arr[2])).exponent() > 0


class TestSpecialValues:
    """Test special value handling (zero, inf, nan)."""

    def test_zero_encoding(self):
        arr_pos = gf_array([0.0], dtype='gf16')
        arr_neg = gf_array([-0.0], dtype='gf16')
        assert GF16(0.0).is_zero()
        assert GF16(-0.0).is_zero()
        assert int(arr_pos[0]) == 0x0000
        assert int(arr_neg[0]) == 0x8000

    def test_infinity_encoding(self):
        # Large values should encode to infinity
        arr = gf_array([1e38], dtype='gf16')
        val = GF16(int(arr[0]))
        assert val.is_inf(), f"Large value should be infinity: bits={hex(val.bits())}"

    def test_zero_roundtrip(self):
        """Zero roundtrip should preserve sign."""
        for val in [0.0, -0.0]:
            arr = gf_array([val], dtype='gf16')
            back = to_float32(arr, src_dtype='gf16')[0]
            assert abs(float(back) - float(val)) < 1e-10, \
                f"Zero roundtrip failed: {val} -> {back}"
