import os
#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
gf48_decode_ref.py -- GOLDEN software decode oracle for gf48
(GoldenFloat48: S1 E18 M29, BIAS=131071=2^17-1); output = IEEE binary64.

Status: [modeled] -- SW oracle on exact rational arithmetic (fractions.Fraction),
NOT hardware. HW-decode = [REQUIRES USER ACTION] (4/4 Tier-E chain on AX7203).

Key difference from Phase-A (gf4..gf32 -> FP32): gf48 mantissa M=29 does NOT fit
in FP32 (23 bits) but fits EXACTLY in FP64 (52 bits) -- so the target type is
binary64, and the entire wide normal range decodes WITHOUT mantissa rounding
(M=29<=52). Rounding appears only on the FP64-subnormal path (deep underflow,
true_exp < -1022), which gf48 (BIAS=131071 -> true_exp in [-131070, +131071])
reaches only at the subnormal edge -- handled by gradual underflow + RNE+sticky.

Decode law (5 classes, HAS_INF semantics), parametric in (E,M,BIAS):
  exp==EXP_MAX, mant==0  -> +-Inf
  exp==EXP_MAX, mant!=0  -> quiet NaN
  exp==0,       mant==0  -> +-0
  exp==0,       mant!=0  -> subnormal: (-1)^s * mant/2^M * 2^(1-BIAS)
  else (normal)          -> (-1)^s * (1+mant/2^M) * 2^(exp-BIAS)

Author: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
"""
import struct
from fractions import Fraction

N, E, M, BIAS = 48, 18, 29, 131071
EXP_MAX = (1 << E) - 1

# FP64 constants
FP64_EBIAS = 1023
FP64_MANT = 52
FP64_MIN_NORM_EXP = -1022
FP64_SUB_LSB_EXP = -1074  # exponent of FP64 subnormal LSB (2^-1074)


def decode_fraction(raw):
    """raw(int, 48-bit) -> ('SPECIAL', name) | Fraction (exact value)."""
    raw &= (1 << N) - 1
    sign = (raw >> (E + M)) & 1
    exp = (raw >> M) & EXP_MAX
    mant = raw & ((1 << M) - 1)
    if exp == EXP_MAX:
        if mant == 0:
            return ("SPECIAL", "INF(-)" if sign else "INF(+)")
        return ("SPECIAL", "NAN(-)" if sign else "NAN(+)")
    if exp == 0:
        if mant == 0:
            return Fraction(0)
        val = Fraction(mant, 1 << M) * pow2(1 - BIAS)
    else:
        val = (1 + Fraction(mant, 1 << M)) * pow2(exp - BIAS)
    return -val if sign else val


def pow2(k):
    return Fraction(1 << k, 1) if k >= 0 else Fraction(1, 1 << (-k))


def frac_to_fp64_bits(val):
    """Exact Fraction -> IEEE binary64 bit pattern (int) with RNE.
    Returns a 64-bit int. Handles 0, normals, subnormals, overflow->Inf."""
    if val == 0:
        return 0  # +0 (caller applies sign by context; magnitude here)
    sign = 1 if val < 0 else 0
    a = abs(val)
    # find the binary order: 2^exp <= a < 2^(exp+1)
    # a = num/den
    num, den = a.numerator, a.denominator
    exp = num.bit_length() - den.bit_length()
    # refine: want 1 <= a / 2^exp < 2
    if a < pow2(exp):
        exp -= 1
    if a >= pow2(exp + 1):
        exp += 1
    # now mantissa_frac = a/2^exp - 1  in [0,1)  (for normal)
    if exp >= FP64_MIN_NORM_EXP:
        # normal: significand = a / 2^exp in [1,2); want 52 fraction bits + RNE
        scaled = a / pow2(exp) * pow2(FP64_MANT)  # in [2^52, 2^53)
        mant_int = round_half_even(scaled)
        # mant_int in [2^52, 2^53]; subtract implicit -> [0, 2^52]
        biased_exp = exp + FP64_EBIAS
        if mant_int == (1 << (FP64_MANT + 1)):  # rounded up -> 2.0
            mant_int >>= 1
            biased_exp += 1
        frac = mant_int - (1 << FP64_MANT)
        if biased_exp >= 0x7FF:
            return (sign << 63) | (0x7FF << 52)  # Inf
        return (sign << 63) | (biased_exp << 52) | frac
    else:
        # subnormal: value = mant52_int * 2^FP64_SUB_LSB_EXP, RNE
        scaled = a / pow2(FP64_SUB_LSB_EXP)
        mant_int = round_half_even(scaled)
        if mant_int >= (1 << FP64_MANT):  # rounded up to a normal
            # promotion to the smallest normal
            return (sign << 63) | (1 << 52) | 0
        return (sign << 63) | mant_int


def round_half_even(frac):
    """round-nearest-ties-even of an exact Fraction to an integer."""
    fl = frac.numerator // frac.denominator
    rem = frac - fl
    if rem < Fraction(1, 2):
        return fl
    if rem > Fraction(1, 2):
        return fl + 1
    # tie
    return fl if (fl % 2 == 0) else fl + 1


def fp64_bits_from_value(val):
    """Full FP64 pattern with correct signed-zero/special class is not needed here:
    called only for finite nonzero values; the sign is already in val."""
    return frac_to_fp64_bits(val)


def golden_fp64(raw):
    """raw -> 64-bit IEEE binary64 pattern (int), reference for DUT comparison."""
    d = decode_fraction(raw)
    sign = (raw >> (E + M)) & 1
    if isinstance(d, tuple):
        name = d[1]
        if name.startswith("INF"):
            return (sign << 63) | (0x7FF << 52)
        # NaN: quiet, payload=1
        return (sign << 63) | (0x7FF << 52) | 1
    if d == 0:
        return sign << 63  # signed zero
    return frac_to_fp64_bits(d)


def python_check_pack(pack_path):
    """Run golden against the conformance pack (value_encoding decimal|dyadic)."""
    import json
    import re
    dy = re.compile(r"^(-?\d+)p(-?\d+)$")
    pack = json.load(open(pack_path))
    ok, fails = 0, []
    for v in pack["vectors"]:
        raw = int(v["hex"], 16)
        d = decode_fraction(raw)
        s = str(v["value"]).strip()
        if s in ("INF(+)", "INF(-)", "NAN(+)", "NAN(-)"):
            exp = ("SPECIAL", s)
        else:
            mo = dy.match(s)
            exp = (int(mo.group(1)) * pow2(int(mo.group(2)))) if mo else Fraction(s)
        if isinstance(d, tuple):
            match = isinstance(exp, tuple) and exp == d
        else:
            match = (not isinstance(exp, tuple)) and d == exp
        ok += 1 if match else 0
        if not match:
            fails.append((v["label"], f"got={d} exp={exp}"))
    return ok, len(pack["vectors"]), fails


if __name__ == "__main__":
    import sys
    p = sys.argv[1] if len(sys.argv) > 1 else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)),
                     "..", "..", "vectors", "gf48_conformance_v0.json")
    ok, tot, fails = python_check_pack(p)
    print(f"gf48 golden (Fraction, FP64-target) vs pack: {ok}/{tot} exact")
    for lbl, msg in fails:
        print("  FAIL", lbl, msg)
    sys.exit(0 if ok == tot else 1)
