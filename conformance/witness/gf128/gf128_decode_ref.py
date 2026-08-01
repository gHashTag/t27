#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
gf128_decode_ref.py -- GOLDEN software decode oracle for gf128
(GoldenFloat128: S1 E49 M78, BIAS=281474976710655 = 2^35-1); exact-rational target.

Status: [verified SW] -- an INDEPENDENT second decode path built from scratch on
exact rational arithmetic (fractions.Fraction). It does NOT reuse the software
encoder that produced the pack, and it is implemented differently from the
in-repo dyadic witness (gf_wide_independent_witness.py): this oracle carries the
full rational value num/den through Fraction, then canonicalizes to a dyadic pair
only for the final exact comparison. Two structurally different exact paths that
agree => a genuine second witness.

Why NOT an FP-target (contrast with gf48):
  gf48 has M=29 <= 52, so it lowered into IEEE binary64 with RNE on the subnormal
  edge. gf128 has M=59 > 52 (binary64 mantissa), so binary64 CANNOT hold the
  mantissa exactly and a binary64 lowering WOULD round. The pack therefore keeps
  every gf128 value as an EXACT dyadic literal A*2^B (value_encoding=dyadic), and
  the conformance target is the exact rational value itself, not an FP lowering.
  Consequently there is NO rounding in the decode path: every representable gf128
  code maps to an exact dyadic rational, and the oracle reproduces it exactly.
  (See the analytic separation-bound note: SEPARATION_BOUND.md.)

Decode law (5 classes, HAS_INF semantics), parametric in (E,M,BIAS), identical law
to the whole GoldenFloat ladder (gf14/gf16/gf48/...):
  exp==EXP_MAX, mant==0  -> +-Inf
  exp==EXP_MAX, mant!=0  -> quiet NaN
  exp==0,       mant==0  -> +-0
  exp==0,       mant!=0  -> subnormal: (-1)^s * mant/2^M * 2^(1-BIAS)
  else (normal)          -> (-1)^s * (1+mant/2^M) * 2^(exp-BIAS)

Author: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
"""
import json
import re
import os
import sys
from fractions import Fraction

N, E, M, BIAS = 128, 49, 78, 281474976710655
EXP_MAX = (1 << E) - 1


def decode_fraction(raw):
    """raw(int, 96-bit) -> ('SPECIAL', name) | ('FIN', significand_Fraction, shift).

    Independent path. CRITICAL: the exponent range of gf128 is +-2^48, so 2^(exp-BIAS)
    must NEVER be materialized as an integer (2^(2^48) bits -- unmaterializable -> OOM). Instead this
    oracle carries the value as a pair (significand, shift) where
        value = significand * 2^shift,
    significand is a small exact Fraction in [0,2) built with fractions.Fraction
    (mantissa arithmetic only, exponents <= M+1 = 79 -> tiny), and shift is a plain
    Python int (may be ~ -2.8e14). This is a DIFFERENT internal decomposition from
    the in-repo dyadic witness (which normalizes an integer odd*2^shift directly):
    here the fractional significand is a genuine Fraction and the huge power of two
    is kept symbolic in `shift`. The two structurally different exact paths agreeing
    is what makes this a real second witness.
    """
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
            return ("FIN", Fraction(0), 0)
        # subnormal: (mant/2^M) * 2^(1-BIAS)
        sig = Fraction(mant, 1 << M)       # in (0,1), tiny denominator (<= 2^59)
        shift = 1 - BIAS
    else:
        # normal: (1 + mant/2^M) * 2^(exp-BIAS)
        sig = 1 + Fraction(mant, 1 << M)   # in [1,2), tiny denominator
        shift = exp - BIAS
    if sign:
        sig = -sig
    return ("FIN", sig, shift)


def _canon(num, shift):
    """Canonicalize num*2^shift -> (odd_num, shift) with odd_num odd (or 0),
    WITHOUT materializing 2^shift."""
    if num == 0:
        return (0, 0)
    sign = -1 if num < 0 else 1
    num = abs(num)
    tz = (num & -num).bit_length() - 1
    num >>= tz
    shift += tz
    return (sign * num, shift)


def sigshift_to_dyadic(sig, shift):
    """(significand Fraction, int shift) -> canonical (odd_num, shift') = value.
    sig has a power-of-two denominator (dyadic) since it is 1+mant/2^M or mant/2^M.
    value = sig.num/sig.den * 2^shift = sig.num * 2^(shift - log2(sig.den))."""
    if sig == 0:
        return (0, 0)
    num, den = sig.numerator, sig.denominator
    if den & (den - 1) != 0:
        raise ValueError(f"non-dyadic significand: {sig}")
    return _canon(num, shift - ((den).bit_length() - 1))


def fraction_to_dyadic(f):
    """Exact (small) Fraction -> canonical (odd_num, shift). For expected-value
    parsing only (pack literals never carry the huge exponent as a Fraction)."""
    if f == 0:
        return (0, 0)
    num, den = f.numerator, f.denominator
    if den & (den - 1) != 0:
        raise ValueError(f"non-dyadic value: {f}")
    return _canon(num, -((den).bit_length() - 1))


_DYADIC = re.compile(r"^(-?\d+)p(-?\d+)$")


def parse_expected(value):
    """pack value -> ('SPECIAL', name) | ('DYADIC', (odd, shift))."""
    s = str(value).strip()
    if s in ("INF(+)", "INF(-)", "NAN(+)", "NAN(-)"):
        return ("SPECIAL", s)
    m = _DYADIC.match(s)
    if m:
        a, b = int(m.group(1)), int(m.group(2))
        # A*2^B exactly, symbolic shift (B may be ~3.4e10) -- no materialization
        return ("DYADIC", _canon(a, b))
    # decimal (must be dyadic; small)
    return ("DYADIC", fraction_to_dyadic(Fraction(s)))


def check_pack(pack_path):
    pack = json.load(open(pack_path))
    ok, fails = 0, []
    for v in pack["vectors"]:
        raw = int(v["hex"], 16)
        # cross-check bits==hex if present
        if v.get("bits") is not None and v["bits"] != raw:
            fails.append((v["label"], f"bits!=hex {v['bits']} vs {raw}"))
            continue
        d = decode_fraction(raw)
        exp = parse_expected(v["value"])
        if d[0] == "SPECIAL":
            match = (exp[0] == "SPECIAL" and exp[1] == d[1])
            got = d
        else:  # ("FIN", sig, shift)
            got = sigshift_to_dyadic(d[1], d[2])
            match = (exp[0] == "DYADIC" and exp[1] == got)
        if match:
            ok += 1
        else:
            fails.append((v["label"], f"got={got} exp={exp}"))
    return ok, len(pack["vectors"]), fails


if __name__ == "__main__":
    p = sys.argv[1] if len(sys.argv) > 1 else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)),
                     "..", "..", "vectors", "gf128_conformance_v0.json")
    ok, tot, fails = check_pack(p)
    print(f"gf128 golden (Fraction exact oracle) vs pack: {ok}/{tot} exact  "
          f"[e={E} m={M} bias={BIAS}]")
    for lbl, msg in fails:
        print("  FAIL", lbl, msg)
    if ok == tot and not fails:
        print("VERDICT: gf128 Fraction-oracle path agrees with pack, abs_error=0")
    sys.exit(0 if (ok == tot and not fails) else 1)
