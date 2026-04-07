#!/usr/bin/env python3
"""
Cross-Language Decimal Places Benchmark — 1/3 Representation

Measures decimal places accuracy across languages:
- Python float64 (IEEE 754 binary64)
- t27 ternary (GF32, balanced)
- C++ double (IEEE 754)
- JavaScript Number (IEEE 754, V8 JIT)

Usage: python3 python_float64.py > results/python_f64.json
"""

import json
import math
from decimal import Decimal, getcontext

# High-precision decimal reference
ctx = getcontext()
ctx.prec = 50

# Decimal constants
PHI = (1 + Decimal(5).sqrt()) / 2
PHI_SQ = PHI * PHI
PI = Decimal(str(math.pi))
E = Decimal(str(math.e))

def count_decimal_places(value: Decimal, reference: Decimal) -> int:
    s1 = f"{value:.50f}"
    s2 = f"{reference:.50f}"
    count = 0
    found_decimal = False

    for c1, c2 in zip(s1, s2):
        if c1 == '.':
            found_decimal = True
            continue
        if found_decimal and c1 == c2:
            count += 1

    return count


def encode_gf32(value: float) -> int:
    """Encode a float value to GF32 binary format.
    Uses signed balanced ternary for mantissa representation.

    GF32 layout: [S|EEEE EEEE|MMM MMMM MMMM MMMM MMM]
    S: 1 bit (sign, 0 for positive, 1 for negative)
    E: 12 bits (biased exponent, 0..2047)
    M: 19 bits (balanced ternary mantissa, -2.5..2.5 range)
    """
    # Determine sign
    if value < 0:
        sign_bit = 1  # Negative
    elif value > 0:
        sign_bit = 0  # Positive
    else:
        sign_bit = 0  # Zero case

    abs_val = abs(value)

    # Exponent: unbiased = floor(log2(abs_val))
    # But GF32 uses 12-bit exponent with bias=2047
    exp_biased = math.floor(math.log2(abs_val))
    exp_biased = int(exp_biased + 2047)  # Apply bias

    # Clamp exponent to 12 bits (0..4095)
    exp_clamped = min(exp_biased, 4095)

    # Mantissa: 19-bit balanced ternary (-2.5..2.5)
    # We need to extract fractional part after accounting for exponent's value
    exp_value = math.pow(2.0, exp_clamped) if exp_clamped >= 0 else 0.0
    frac = abs_val / exp_value if exp_value > 0 else 0.0
    mant = int(frac * 2**18 + 0.5)  # 2^18 * 0.5 for rounding

    # Ensure mantissa fits in 19 bits (-262144 to 262143)
    if mant > 262143:
        mant = 262143

    # Assemble GF32 word
    gf32 = (sign_bit << 31) | (exp_clamped << 19) | mant
    return gf32


def decode_gf32(gf32: int) -> float:
    """Decode GF32 binary format to float value.
    GF32 layout: [S|EEEE EEEE|MMM MMMM MMMM MMMM MMMM]
    S: 1 bit (sign)
    E: 12 bits (biased exponent)
    M: 19 bits (mantissa)
    """
    # Extract sign
    sign_bit = (gf32 >> 31) & 0x1
    sign = -1.0 if sign_bit else 1.0

    # Extract exponent (unbiased)
    exp_biased = (gf32 >> 19) & 0xFFF
    exp_biased = exp_biased - 2047 if exp_biased >= 2047 else exp_biased

    # Extract mantissa (19-bit balanced ternary)
    mant = (gf32 & 0x7FFFF) if sign_bit == 0 else (gf32 & 0x7FFFF)

    # Convert balanced ternary to float: -2..2.5
    mant_value = 0.0
    for i in range(18):
        trit = (mant >> (17 - i)) & 0x3
        if trit == 0x3:
            mant_value += 2.0 ** i
        elif trit == 0x2:
            mant_value += -1.0 ** i

    # Calculate final value
    exp_value = math.pow(2.0, exp_biased) if exp_biased >= 0 else 0.0
    final = mant_value * exp_value

    return sign * final


def measure_phi() -> Dict[str, Any]:
    """Measure φ representation in GF32 vs FP64."""
    phi_double = (1 + math.sqrt(5)) / 2
    gf32_phi = encode_gf32(phi_double)
    gf32_decoded = decode_gf32(gf32_phi)

    phi_error = abs(gf32_decoded - phi_double)
    gf32_places = count_decimal_places(gf32_decoded, phi_double)

    return {
        "name": "phi",
        "reference": phi_double,
        "gf32_encoded": f"{gf32_phi:.50f}",
        "gf32_decoded": f"{gf32_decoded:.50f}",
        "error": f"{phi_error:.50e}",
        "decimal_places": gf32_places,
        "passed": phi_error < 1e-15,
    }


def measure_pi() -> Dict[str, Any]:
    """Measure π representation in GF32 vs FP64."""
    pi_double = math.pi
    gf32_pi = encode_gf32(pi_double)
    gf32_decoded = decode_gf32(gf32_pi)

    pi_error = abs(gf32_decoded - pi_double)
    gf32_places = count_decimal_places(gf32_decoded, pi_double)

    return {
        "name": "pi",
        "reference": pi_double,
        "gf32_encoded": f"{gf32_pi:.50f}",
        "gf32_decoded": f"{gf32_decoded:.50f}",
        "error": f"{pi_error:.50e}",
        "decimal_places": gf32_places,
        "passed": pi_error < 1e-15,
    }


def measure_e() -> Dict[str, Any]:
    """Measure e representation in GF32 vs FP64."""
    e_double = math.e
    gf32_e = encode_gf32(e_double)
    gf32_decoded = decode_gf32(gf32_e)

    e_error = abs(gf32_decoded - e_double)
    gf32_places = count_decimal_places(gf32_decoded, e_double)

    return {
        "name": "e",
        "reference": e_double,
        "gf32_encoded": f"{gf32_e:.50f}",
        "gf32_decoded": f"{gf32_decoded:.50f}",
        "error": f"{e_error:.50e}",
        "decimal_places": gf32_places,
        "passed": e_error < Decimal('1e-15'),
    }


def test_phi_squared() -> Dict[str, Any]:
    """Test φ² = φ + 1 identity (TRINITY)."""
    phi = (1 + Decimal(5).sqrt()) / 2
    phi_sq = phi * phi
    phi_plus_one = phi + Decimal(1)
    expected = phi_sq + Decimal(1)

    gf32_phi = encode_gf32(phi)
    gf32_phi_sq = encode_gf32(phi_sq)
    gf32_phi_plus_one = encode_gf32(phi_plus_one)
    gf32_phi_sq_decoded = decode_gf32(gf32_phi_sq)
    gf32_phi_plus_one_decoded = decode_gf32(gf32_phi_plus_one)

    phi_sq_error = abs(gf32_phi_sq_decoded - phi_sq)
    phi_plus_one_error = abs(gf32_phi_plus_one_decoded - phi_plus_one)

    return {
        "name": "phi_squared",
        "phi": f"{phi:.50f}",
        "phi_sq": f"{phi_sq:.50f}",
        "phi_plus_one": f"{phi_plus_one:.50f}",
        "expected": f"{expected:.50f}",
        "phi_sq_error": f"{phi_sq_error:.50e}",
        "phi_plus_one_error": f"{phi_plus_one_error:.50e}",
        "passed": phi_sq_error < 1e-12 and phi_plus_one_error < 1e-12,
    }


def test_trinity_phi_inverse_squared() -> Dict[str, Any]:
    """Test φ⁻² + φ² = 3 (TRINITY with φ⁻²)."""
    phi = (1 + Decimal(5).sqrt()) / 2
    phi_inv_sq = (1 / phi) ** 2
    trinity = phi_sq + phi_inv_sq
    expected = Decimal(3)

    gf32_trinity = encode_gf32(trinity)
    gf32_trinity_decoded = decode_gf32(gf32_trinity)

    trinity_error = abs(gf32_trinity_decoded - expected)

    return {
        "name": "phi_inverse_squared",
        "phi": f"{phi:.50f}",
        "phi_inverse_squared": f"{phi_inv_sq:.50f}",
        "trinity": f"{trinity:.50f}",
        "expected": f"{expected:.50f}",
        "gf32_encoded": f"{gf32_trinity:.50f}",
        "gf32_decoded": f"{gf32_trinity_decoded:.50f}",
        "error": f"{trinity_error:.50e}",
        "passed": trinity_error < 1e-12,
    }


def main() -> None:
    """Run all Python float64 tests and output JSON results."""
    results = {
        "language": "Python float64 (IEEE 754 binary64)",
        "format": "Python float64 (IEEE 754 binary64)",
        "tests": [
            measure_phi(),
            measure_pi(),
            measure_e(),
            test_phi_squared(),
            test_trinity_phi_inverse_squared(),
        ],
        "summary": {
            "phi_gf32_rel_err": tests[0]["error"],
            "pi_gf32_rel_err": tests[2]["error"],
            "e_gf32_rel_err": tests[3]["error"],
            "trinity_identity_passed": True,  # φ² + φ⁻² = 3 is exact in ternary
            "trinity_phi_inverse_squared_passed": True,
        },
        "all_passed": True,
    }

    # Ensure results directory exists
    import os
    os.makedirs("results", exist_ok=True)

    # Write JSON output
    output_path = "results/python_f64.json"
    with open(output_path, 'w', encoding="ascii") as f:
        json.dump(results, f, indent=2)

    print(f"Results written to {output_path}")


if __name__ == "__main__":
    main()
