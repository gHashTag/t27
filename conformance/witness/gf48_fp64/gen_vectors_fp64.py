#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
gen_vectors_fp64.py -- generate the vector file vectors_gf48.txt for the iverilog
testbench tb_gf_decode_fp64.v. Each line: "<hex_gf48> <hex_fp64_golden>".

The golden binary64 pattern comes from the independent Fraction oracle
gf48_decode_ref.golden_fp64() (NOT from the RTL model -- oracle and DUT are independent).

Set = 5-class + boundary + full-exponent stress sweep + deep-underflow edge +
deterministic random (2^48 full enumeration is infeasible). Same set that
rtl_bit_model_fp64.py checks, so iverilog reproduces exactly the same gate.

Author: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
"""
import sys
from gf48_decode_ref import golden_fp64, N, E, M, BIAS, EXP_MAX

FP64_MIN_NORM_EXP = -1022
FP64_SUB_LSB_EXP = -1074
MASK_M = (1 << M) - 1


def representative_codes():
    import random
    random.seed(27)
    codes = set()
    codes.add(0)
    codes.add(1 << (E + M))
    for sgn in (0, 1):
        codes.add((sgn << (E + M)) | (EXP_MAX << M))            # Inf
        codes.add((sgn << (E + M)) | (EXP_MAX << M) | 1)        # NaN
        codes.add((sgn << (E + M)) | (EXP_MAX << M) | MASK_M)   # NaN max payload
        codes.add((sgn << (E + M)) | 1)                          # smallest sub
        codes.add((sgn << (E + M)) | MASK_M)                     # largest sub
        codes.add((sgn << (E + M)) | (1 << M))                   # smallest normal
        codes.add((sgn << (E + M)) | ((EXP_MAX - 1) << M) | MASK_M)  # largest normal
    for h in (0x3FFFE0000000, 0x400000000000, 0x400010000000,
              0x3FFFF0000000, 0xBFFFE0000000):
        codes.add(h)
    for e in range(0, EXP_MAX + 1, max(1, EXP_MAX // 4000)):
        for mant in (0, MASK_M >> 1, MASK_M):
            for sgn in (0, 1):
                codes.add((sgn << (E + M)) | (e << M) | mant)
    for target in (FP64_MIN_NORM_EXP, FP64_SUB_LSB_EXP, FP64_MIN_NORM_EXP - 1,
                   FP64_SUB_LSB_EXP - 1, FP64_SUB_LSB_EXP - 30):
        e = target + BIAS
        if 1 <= e <= EXP_MAX - 1:
            for mant in (0, 1, MASK_M >> 1, MASK_M):
                for sgn in (0, 1):
                    codes.add((sgn << (E + M)) | (e << M) | mant)
    for _ in range(200000):
        codes.add(random.getrandbits(N))
    return sorted(codes)


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "vectors_gf48.txt"
    codes = representative_codes()
    with open(out, "w") as f:
        for raw in codes:
            fp = golden_fp64(raw)
            f.write(f"{raw & ((1 << N) - 1):012x} {fp:016x}\n")
    print(f"wrote {len(codes)} vectors to {out}")


if __name__ == "__main__":
    main()
