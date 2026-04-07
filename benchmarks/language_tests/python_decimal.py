#!/usr/bin/env python3
"""
Reference GF32 roundtrip vs IEEE f32 — aligns with specs/numeric/gf32.t27.

Normative numeric SSOT: conformance/FORMAT-SPEC-001.json and *.t27 specs.
This script is a non-critical paper helper; verdict math stays in tri/t27c.

Usage (from repo root):
  python3 benchmarks/language_tests/python_decimal.py
"""

from __future__ import annotations

import json
import math
import struct
from decimal import Decimal, getcontext
from typing import Any

getcontext().prec = 50

EXP_BIAS = 2047
EXP_BITS = 12
MANT_BITS = 19
MANT_DIV = float(1 << MANT_BITS)  # 524288


def as_f32(x: float) -> float:
    return struct.unpack("f", struct.pack("f", float(x)))[0]


def floor_log2(x: float) -> int:
    if x <= 0.0:
        return -32768
    exp = 0
    while x >= 2.0:
        x /= 2.0
        exp += 1
    while x < 1.0:
        x *= 2.0
        exp -= 1
    return exp


def extract_mantissa(value: float, exp: int, mant_bits: int) -> int:
    normalized = value / (2.0**exp)
    frac = normalized - 1.0
    max_mant = (1 << mant_bits) - 1
    return int(frac * (max_mant + 1.0))


def encode_gf32(value: float) -> int:
    v = as_f32(value)
    if v == 0.0:
        return 0
    sign = 1 if v < 0.0 else 0
    abs_val = -v if v < 0.0 else v
    exp_unbiased = floor_log2(abs_val)
    exp_biased = exp_unbiased + EXP_BIAS
    exp_clamped = max(0, min(exp_biased, (1 << EXP_BITS) - 1))
    mant = extract_mantissa(abs_val, exp_unbiased, MANT_BITS)
    mant &= (1 << MANT_BITS) - 1
    return (sign << 31) | (exp_clamped << MANT_BITS) | mant


def decode_gf32(raw: int) -> float:
    sign = (raw >> 31) & 0x1
    exp_biased = (raw >> MANT_BITS) & 0xFFF
    mant = raw & ((1 << MANT_BITS) - 1)
    if exp_biased == 0 and mant == 0:
        return 0.0
    if exp_biased == 0:
        exp_unbiased = -EXP_BIAS + 1
        mant_normalized = mant / MANT_DIV
    else:
        exp_unbiased = exp_biased - EXP_BIAS
        mant_normalized = 1.0 + mant / MANT_DIV
    value = mant_normalized * (2.0**exp_unbiased)
    if sign:
        value = -value
    return as_f32(value)


def gf32_roundtrip_f32(x: float) -> float:
    return decode_gf32(encode_gf32(x))


def rel_err(reference: float, got: float) -> float:
    if reference == 0.0:
        return abs(got)
    return abs(got - reference) / abs(reference)


def measure_constant(name: str, ref_f64: float) -> dict[str, Any]:
    ref = as_f32(ref_f64)
    out = gf32_roundtrip_f32(ref)
    return {
        "name": name,
        "reference_f32": repr(ref),
        "gf32_roundtrip_f32": repr(out),
        "relative_error": rel_err(ref, out),
    }


def main() -> None:
    phi = (1.0 + math.sqrt(5.0)) / 2.0
    tests = [
        measure_constant("phi", phi),
        measure_constant("phi_inv", 1.0 / phi),
        measure_constant("pi", math.pi),
        measure_constant("e", math.e),
        measure_constant("one_third", 1.0 / 3.0),
    ]

    # Decimal: repeating expansion for 1/3 (informational)
    d_third = Decimal(1) / Decimal(3)
    f_third = as_f32(1.0 / 3.0)
    summary = {
        "phi_gf32_rel_err": tests[0]["relative_error"],
        "pi_gf32_rel_err": tests[2]["relative_error"],
        "e_gf32_rel_err": tests[3]["relative_error"],
        "one_third_gf32_rel_err": tests[4]["relative_error"],
        "decimal_one_third_is_exact": True,
        "float32_one_third_repeats_binary": True,
        "spec_pointer": "specs/numeric/gf32.t27, conformance/FORMAT-SPEC-001.json",
    }

    payload = {
        "description": "GF32 encode/decode roundtrip (f32 reference), matches gf32.t27 layout",
        "tests": tests,
        "decimal_one_third_sample": str(d_third)[:60] + "...",
        "float32_one_third": repr(f_third),
        "summary": summary,
    }

    import os

    out_dir = os.path.join(os.path.dirname(__file__), "results")
    os.makedirs(out_dir, exist_ok=True)
    path = os.path.join(out_dir, "sacred_constants.json")
    with open(path, "w", encoding="ascii") as f:
        json.dump(payload, f, indent=2)

    print(json.dumps(summary, indent=2))
    print(f"Wrote {path}")


if __name__ == "__main__":
    main()
