#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
rtl_bit_model_fp64.py -- BIT MODEL of RTL gf_decode_param_fp64.v (Python equivalent
of the exact combinational Verilog logic for gf48 -> IEEE binary64).

PURPOSE: reproduce the fixed-width datapath of the target RTL one-to-one, in order
to prove the semantics BEFORE the iverilog run (no iverilog in sandbox). NOTE
(lesson 04.07): Python arbitrary-width int does NOT catch fixed-width bugs (truncation/
OOB-read) -- so this model emulates widths explicitly via masks, and the final
witness = independent iverilog at the local agent. Model = target specification.

Difference from the Phase-A (FP32) model: target is binary64 (52-bit mantissa).
For gf48 M=29 <= 52 => normal path = pure zero-pad left (widen-before-shift),
WITHOUT mantissa rounding. FP64-subnormal path (true_exp < -1022) = gradual underflow
+ guard/round/sticky against LSB 2^-1074.

Author: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
"""
from gf48_decode_ref import decode_fraction, golden_fp64, N, E, M, BIAS, EXP_MAX

FP64_EBIAS = 1023
FP64_MANT = 52
FP64_MIN_NORM_EXP = -1022
FP64_SUB_LSB_EXP = -1074

MASK_N = (1 << N) - 1
MASK_M = (1 << M) - 1
MASK52 = (1 << 52) - 1


def clz(v, width):
    """count leading zeros in a width-bit value (v!=0)."""
    for i in range(width):
        if (v >> (width - 1 - i)) & 1:
            return i
    return width


def rtl_decode_fp64(raw):
    """Exact fixed-width bit-model. Returns a 64-bit int (IEEE binary64)."""
    raw &= MASK_N
    sign = (raw >> (E + M)) & 1
    exp_in = (raw >> M) & EXP_MAX
    mant_in = raw & MASK_M

    is_exp_zero = (exp_in == 0)
    is_exp_max = (exp_in == EXP_MAX)
    is_mant_zero = (mant_in == 0)

    cls_zero = is_exp_zero and is_mant_zero
    cls_subnormal = is_exp_zero and not is_mant_zero
    cls_inf = is_exp_max and is_mant_zero
    cls_nan = is_exp_max and not is_mant_zero

    if cls_nan:
        return (sign << 63) | (0x7FF << 52) | 1
    if cls_inf:
        return (sign << 63) | (0x7FF << 52)
    if cls_zero:
        return sign << 63

    # ---- renormalize GF-subnormal, else GF-normal ----
    if cls_subnormal:
        lz = clz(mant_in, M)
        true_exp = (1 - BIAS) - (lz + 1)
        frac_bits = (mant_in << (lz + 1)) & MASK_M  # M-bit truncated field
    else:
        true_exp = exp_in - BIAS
        frac_bits = mant_in

    # ---- Attempt 1: FP64 NORMAL packer (widen-before-shift, M<=52 => no round) ----
    # widen frac_bits (M bits) into 53-bit result field, shift left by (52-M).
    WIDE = FP64_MANT if M <= FP64_MANT else M
    pf_wide = frac_bits & ((1 << (WIDE + 1)) - 1)  # zero-extended
    norm_widen = (pf_wide << (FP64_MANT - M)) & ((1 << (WIDE + 1)) - 1)
    norm_carry = (norm_widen >> FP64_MANT) & 1     # stays 0 for pure widen
    norm_mant52 = norm_widen & MASK52
    norm_exp_final = true_exp + norm_carry + FP64_EBIAS

    is_normal_candidate = (true_exp >= FP64_MIN_NORM_EXP)
    norm_overflow = is_normal_candidate and (norm_exp_final >= 0x7FF)
    takes_normal = is_normal_candidate and (not norm_overflow) and (norm_exp_final >= 1)
    corrected_true_exp = true_exp + norm_carry

    if norm_overflow:
        return (sign << 63) | (0x7FF << 52)
    if takes_normal:
        return (sign << 63) | ((norm_exp_final & 0x7FF) << 52) | norm_mant52

    # ---- Attempt 2: FP64 SUBNORMAL packer (gradual underflow, RNE+sticky) ----
    # Finite nonzero value = full_sig * 2^(eff_true_exp - M), where
    # full_sig = implicit-1 . frac (M+1 bits). Target FP64-subnormal LSB = 2^-1074.
    # We want to represent value ~ q * 2^-1074 with RNE. shift = number of fraction bits
    # to drop (>=1 in the subnormal domain), while WIDTH_FULL is fixed.
    eff_true_exp = corrected_true_exp if is_normal_candidate else true_exp
    full_sig = (1 << M) | frac_bits            # (M+1) bits: implicit1 + frac
    WIDTH_FULL = M + 1
    # LSB exponent of full_sig in the value scale = 2^(eff_true_exp - M)
    # to make the LSB scale become 2^-1074 => shift right by:
    shift = FP64_SUB_LSB_EXP - (eff_true_exp - M)   # >=1 in this domain

    if shift <= 0:
        # subnormal domain, but value >= smallest-normal border => promotion
        # (normally unreachable; safety guard)
        sub_shifted = (full_sig << (-shift))
        sub_guard = 0
        sub_sticky = 0
    elif shift >= WIDTH_FULL + 2:
        # whole significand fell past the sticky window => underflow to 0/subnormal-LSB
        sub_shifted = 0
        sub_guard = (full_sig >> (WIDTH_FULL - 1)) & 1 if shift == WIDTH_FULL + 1 else 0
        sub_sticky = 1 if (full_sig & ((1 << WIDTH_FULL) - 1)) else 0
    else:
        sub_shifted = full_sig >> shift
        sub_guard = (full_sig >> (shift - 1)) & 1
        sticky_mask = (1 << (shift - 1)) - 1
        sub_sticky = 1 if (full_sig & sticky_mask) else 0

    sub_mant_pre = sub_shifted & MASK52
    sub_round_up = sub_guard and (sub_sticky or (sub_shifted & 1))
    sub_mant_rounded = sub_mant_pre + (1 if sub_round_up else 0)
    sub_carry_to_normal = (sub_mant_rounded >> 52) & 1
    sub_mant52 = sub_mant_rounded & MASK52

    if sub_carry_to_normal:
        return (sign << 63) | (1 << 52) | 0  # smallest FP64 normal
    return (sign << 63) | sub_mant52


def sweep(codes):
    fails = []
    for raw in codes:
        got = rtl_decode_fp64(raw)
        exp = golden_fp64(raw)
        # NaN payload-agnostic
        def is_nan(w):
            return ((w >> 52) & 0x7FF) == 0x7FF and (w & MASK52) != 0
        if is_nan(exp):
            ok = is_nan(got)
        else:
            ok = (got == exp)
        if not ok:
            fails.append((raw, got, exp))
    return fails


def representative_codes():
    """5-class + boundary + random representative set (2^48 exhaustive infeasible)."""
    import random
    random.seed(27)
    codes = set()
    # zero / neg-zero
    codes.add(0)
    codes.add(1 << (E + M))
    # Inf / NaN both signs
    for sgn in (0, 1):
        codes.add((sgn << (E + M)) | (EXP_MAX << M))            # Inf
        codes.add((sgn << (E + M)) | (EXP_MAX << M) | 1)        # NaN
        codes.add((sgn << (E + M)) | (EXP_MAX << M) | MASK_M)   # NaN max payload
    # smallest/largest subnormal
    for sgn in (0, 1):
        codes.add((sgn << (E + M)) | 1)                          # smallest sub
        codes.add((sgn << (E + M)) | MASK_M)                     # largest sub
    # smallest/largest normal + one/two/three/1.5
    for sgn in (0, 1):
        codes.add((sgn << (E + M)) | (1 << M))                   # smallest normal
        codes.add((sgn << (E + M)) | ((EXP_MAX - 1) << M) | MASK_M)  # largest normal
    # exact 1.0, 2.0, 3.0, 1.5, -1.0 (from known pack)
    for h in (0x3FFFE0000000, 0x400000000000, 0x400010000000,
              0x3FFFF0000000, 0xBFFFE0000000):
        codes.add(h)
    # full exponent stress: sweep every exponent value with mant=0, mid, max
    for e in range(0, EXP_MAX + 1, max(1, EXP_MAX // 4000)):
        for mant in (0, MASK_M >> 1, MASK_M):
            for sgn in (0, 1):
                codes.add((sgn << (E + M)) | (e << M) | mant)
    # deep-underflow boundary: exponents where true_exp crosses -1022 and -1074
    for target in (FP64_MIN_NORM_EXP, FP64_SUB_LSB_EXP, FP64_MIN_NORM_EXP - 1,
                   FP64_SUB_LSB_EXP - 1, FP64_SUB_LSB_EXP - 30):
        e = target + BIAS
        if 1 <= e <= EXP_MAX - 1:
            for mant in (0, 1, MASK_M >> 1, MASK_M):
                for sgn in (0, 1):
                    codes.add((sgn << (E + M)) | (e << M) | mant)
    # random fill
    for _ in range(200000):
        codes.add(random.getrandbits(N))
    return sorted(codes)


if __name__ == "__main__":
    codes = representative_codes()
    fails = sweep(codes)
    print(f"gf48 FP64 RTL bit-model vs golden: {len(codes)-len(fails)}/{len(codes)} bit-exact "
          f"(fails={len(fails)})")
    for raw, got, exp in fails[:20]:
        print(f"  raw=0x{raw:012X} got=0x{got:016X} exp=0x{exp:016X}")
    import sys
    sys.exit(0 if not fails else 1)
