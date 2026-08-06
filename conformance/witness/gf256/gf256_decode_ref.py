#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
gf256_decode_ref.py -- GOLDEN software decode oracle for gf256
(GoldenFloat256: S1 E97 M158, BIAS = 2^96-1); exact-rational target.

Status: [verified SW] -- an INDEPENDENT second decode path built from scratch on
exact rational arithmetic (fractions.Fraction). It does NOT reuse the software
encoder/generator that produced the pack, and it is implemented DIFFERENTLY from the
in-repo dyadic witness (gf_wide_independent_witness.py): that path normalizes an
integer (2^M+mant) into (odd, shift) by factoring twos; THIS oracle instead carries
the significand as a genuine fractions.Fraction in [0,2) and keeps the large power
of two as a separate symbolic integer shift, canonicalizing to a dyadic pair only
for the final exact comparison. Two structurally different exact paths that agree
=> a genuine second witness.

Why NOT an FP-target (contrast with gf48):
  gf48 has M=29 <= 52, so it lowered into IEEE binary64 with RNE on the subnormal
  edge. gf256 has M=158 >> 52, so binary64 CANNOT hold the mantissa exactly and a
  binary64 lowering WOULD round. The pack therefore keeps every gf256 value as an
  EXACT dyadic literal A*2^B (value_encoding=dyadic), and the conformance target is
  the exact rational value itself, not an FP lowering. Consequently there is NO
  rounding in the decode path: every representable gf256 code maps to an exact
  dyadic rational, and the oracle reproduces it exactly.
  (See the analytic separation-bound note: SEPARATION_BOUND.md.)

Decode law (5 classes, HAS_INF semantics), parametric in (E,M,BIAS), identical law
to the whole GoldenFloat ladder (gf14/gf16/gf48/gf128/gf512/gf1024):
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

N, E, M = 256, 97, 158
BIAS = (1 << 96) - 1          # 2^(E-1)-1 closed form; matches spec BIAS_EXPR "2^(97-1)-1"
EXP_MAX = (1 << E) - 1


def decode_fraction(raw):
    """raw(int, 256-bit) -> ('SPECIAL', name) | ('FIN', significand_Fraction, shift).

    Independent path. The exponent range of gf256 is +-2^96, so 2^(exp-BIAS) must
    NEVER be materialized as an integer (unmaterializable -> OOM). Instead this
    oracle carries the value as a pair (significand, shift) where
        value = significand * 2^shift,
    significand is a small exact Fraction in [0,2) built with fractions.Fraction
    (mantissa arithmetic only, denominator <= 2^M -> tiny), and shift is a plain
    Python int (may be ~ -8e28). This is a DIFFERENT internal decomposition from the
    in-repo dyadic witness (which normalizes an integer odd*2^shift directly): here
    the fractional significand is a genuine Fraction and the huge power of two is
    kept symbolic in `shift`. The two structurally different exact paths agreeing is
    what makes this a real second witness.
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
        sig = Fraction(mant, 1 << M)       # subnormal, in (0,1)
        shift = 1 - BIAS
    else:
        sig = 1 + Fraction(mant, 1 << M)   # normal, in [1,2)
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
    """(Fraction significand, int shift) -> canonical (odd_num, shift') = value.
    sig has a power-of-two denominator (dyadic): 1+mant/2^M or mant/2^M."""
    if sig == 0:
        return (0, 0)
    num, den = sig.numerator, sig.denominator
    if den & (den - 1) != 0:
        raise ValueError(f"non-dyadic significand: {sig}")
    return _canon(num, shift - ((den).bit_length() - 1))


def fraction_to_dyadic(f):
    if f == 0:
        return (0, 0)
    num, den = f.numerator, f.denominator
    if den & (den - 1) != 0:
        raise ValueError(f"non-dyadic value: {f}")
    return _canon(num, -((den).bit_length() - 1))


_DYADIC = re.compile(r"^(-?\d+)p(-?\d+)$")


def parse_expected(value):
    s = str(value).strip()
    if s in ("INF(+)", "INF(-)", "NAN(+)", "NAN(-)"):
        return ("SPECIAL", s)
    m = _DYADIC.match(s)
    if m:
        a, b = int(m.group(1)), int(m.group(2))
        return ("DYADIC", _canon(a, b))
    return ("DYADIC", fraction_to_dyadic(Fraction(s)))


def check_pack(pack_path):
    pack = json.load(open(pack_path))
    ok, fails = 0, []
    for v in pack["vectors"]:
        raw = int(v["hex"], 16)
        if v.get("bits") is not None and v["bits"] != raw:
            fails.append((v["label"], f"bits!=hex {v['bits']} vs {raw}"))
            continue
        d = decode_fraction(raw)
        exp = parse_expected(v["value"])
        if d[0] == "SPECIAL":
            match = (exp[0] == "SPECIAL" and exp[1] == d[1])
            got = d
        else:
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
                     "..", "..", "vectors", "gf256_conformance_v0.json")
    ok, tot, fails = check_pack(p)
    print(f"gf256 golden (Fraction exact oracle) vs pack: {ok}/{tot} exact  "
          f"[e={E} m={M} bias={BIAS}]")
    for lbl, msg in fails:
        print("  FAIL", lbl, msg)
    if ok == tot and not fails:
        print("VERDICT: gf256 Fraction-oracle path agrees with pack, abs_error=0")
    sys.exit(0 if (ok == tot and not fails) else 1)
