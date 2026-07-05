#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
cross_check_representative.py -- cross-verify the TWO independent gf256 decode paths
(dyadic integer normalizer  vs  Fraction-significand + symbolic shift) over a large
REPRESENTATIVE code set. 2^256 exhaustive is infeasible; this is a falsifiable
representative sweep: 5-class + exponent boundaries + full-mantissa edges +
deep-underflow/overflow edges + deterministic random (seed=256).

Both paths must agree bit-exactly (same canonical dyadic (odd, shift), or same
special class) on every probed code. Any disagreement -> exit 1 (falsified).

Author: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
"""
import os
import random
import sys

# Make both witness modules importable no matter the current working directory:
# witness 2 (gf256_decode_ref) lives beside this file; witness 1
# (gf_wide_independent_witness) lives two levels up in conformance/.
_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)                              # gf256_decode_ref.py
sys.path.insert(0, os.path.abspath(os.path.join(_HERE, "..", "..")))  # conformance/

import gf256_decode_ref as ORACLE                      # Fraction path (witness 2)
from gf_wide_independent_witness import make_decoder, normalize_dyadic  # dyadic path (witness 1)

# gf256: S1 E97 M158, BIAS = 2^(97-1)-1 = 2^96-1
N, E, M = 256, 97, 158
BIAS = (1 << 96) - 1                                   # 79228162514264337593543950335
EXP_MAX = (1 << E) - 1


def path_dyadic(raw):
    """Witness-1 decode -> ('SPECIAL',name) | ('NUM',(odd,shift))."""
    dec, _ = make_decoder(E, M, BIAS)
    r = dec(raw)
    if isinstance(r, str):
        if r.startswith("ZERO"):
            return ("NUM", (0, 0))
        return ("SPECIAL", r)
    return ("NUM", r)  # (odd, shift)


def path_fraction(raw):
    """Witness-2 decode -> ('SPECIAL',name) | ('NUM',(odd,shift))."""
    d = ORACLE.decode_fraction(raw)
    if d[0] == "SPECIAL":
        return ("SPECIAL", d[1])
    return ("NUM", ORACLE.sigshift_to_dyadic(d[1], d[2]))


def make_code(sign, exp, mant):
    return (sign << (E + M)) | ((exp & EXP_MAX) << M) | (mant & ((1 << M) - 1))


def representative_codes():
    codes = set()
    MMAX = (1 << M) - 1
    exps = {0, 1, 2, 3, EXP_MAX, EXP_MAX - 1, EXP_MAX - 2,
            BIAS & EXP_MAX, (BIAS - 1) & EXP_MAX, (BIAS + 1) & EXP_MAX,
            1 << (E - 1), (1 << (E - 1)) - 1, (1 << (E - 1)) + 1}
    # sweep a denser band of exponents too
    for e in range(0, 40):
        exps.add(e)
        exps.add(EXP_MAX - e)
    mants = {0, 1, 2, 3, MMAX, MMAX - 1, MMAX >> 1, (MMAX >> 1) + 1,
             1 << (M - 1), (1 << (M - 1)) - 1, 1 << (M // 2)}
    for s in (0, 1):
        for e in exps:
            for m in mants:
                codes.add(make_code(s, e, m))
    # deterministic random
    rng = random.Random(256)
    for _ in range(200000):
        codes.add(rng.getrandbits(N))
    return codes


def main():
    codes = representative_codes()
    ok = 0
    fails = []
    for raw in codes:
        a = path_dyadic(raw)
        b = path_fraction(raw)
        # NaN: both paths must classify as NaN(+/-) with matching sign; payload irrelevant
        if a[0] == "SPECIAL" and b[0] == "SPECIAL":
            an, bn = a[1], b[1]
            if an.startswith("NAN") and bn.startswith("NAN"):
                match = (an[3:] == bn[3:])  # sign parenthetical
            else:
                match = (an == bn)
        else:
            match = (a == b)
        if match:
            ok += 1
        else:
            if len(fails) < 20:
                fails.append((hex(raw), a, b))
    tot = len(codes)
    print(f"gf256 cross-check (dyadic path == Fraction path): {ok}/{tot} agree "
          f"[e={E} m={M} bias={BIAS}]")
    if fails:
        print("FAILS (first 20):")
        for h, a, b in fails:
            print(f"  {h}: dyadic={a} fraction={b}")
        return 1
    print(f"VERDICT: {tot} representative codes, two independent exact paths agree, "
          f"abs_error=0 (2^256 exhaustive infeasible; falsifiable representative sweep)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
