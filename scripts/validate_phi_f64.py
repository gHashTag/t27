#!/usr/bin/env python3
"""Cross-check IEEE 754 binary64 parameters for Flocq [B754_finite] (PHI-IDENTITY Phase B).

Run from repo root:  python3 scripts/validate_phi_f64.py
Mantissa = full significand (implicit leading 1 + 52 fraction bits).
Exponent = unbiased exponent minus 52 (Flocq-style, matches typical decode recipes).
"""

from __future__ import annotations

import math
import struct


def f64_params(x: float, name: str = "x") -> tuple[int, int, int]:
    bits = struct.unpack(">Q", struct.pack(">d", x))[0]
    sign = (bits >> 63) & 1
    exp_biased = (bits >> 52) & 0x7FF
    mantissa_bits = bits & 0xFFFFFFFFFFFFF
    mantissa_full = (1 << 52) | mantissa_bits
    exp_flocq = exp_biased - 1023 - 52
    verify = (-1) ** sign * mantissa_full * 2**exp_flocq
    assert verify == x, f"decode mismatch: {verify!r} != {x!r}"
    print(f"--- {name} ---")
    print(f"  mantissa = {mantissa_full}  (Coq positive)")
    print(f"  exponent = {exp_flocq}  (Coq Z)")
    print(f"  hex      = {x.hex()}")
    return sign, mantissa_full, exp_flocq


def main() -> None:
    phi = (1.0 + math.sqrt(5.0)) / 2.0
    f64_params(phi, "phi")
    f64_params(phi * phi, "phi_sq")
    f64_params(phi + 1.0, "phi_plus_one")

    residual = abs(phi * phi - (phi + 1.0))
    tolerance = 5.0 * 2.0**-53 * phi**2
    print()
    print(f"|phi^2 - (phi+1)| = {residual:.20e}")
    print(f"PHI_TOLERANCE     = {tolerance:.20e}")
    print(f"residual < tol    = {residual < tolerance}")
    print(f"phi_sq == phi_po  = {phi * phi == phi + 1.0}")


if __name__ == "__main__":
    main()
