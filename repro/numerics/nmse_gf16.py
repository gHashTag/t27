#!/usr/bin/env python3
"""
L4/L5 differential + comparative numeric-accuracy oracle for GoldenFloat GF16.

Implements docs/GF16_BFLOAT16_NMSE_PROTOCOL.md (host-only, unsealed).
Compares GF16 (E6M9) against bfloat16 and IEEE float16 by round-trip
real -> format -> real over the protocol's five reference distributions,
reporting NMSE, NMSE ratios, max-abs-err and a ULP-like (kappa-approx) metric.

Codec under test: conformance/gf16_ref.py (BIAS=31, EXP_BITS=6, MANT_BITS=9).
bf16: ml_dtypes.bfloat16. fp16: numpy.float16.

R5-HONEST: this is host-measured, unsealed => informational, NOT a silicon
certifying claim (protocol section 8). D_PHI is an identity-anchored sanity
check (L5), NOT a superiority claim.

Run:  python repro/numerics/nmse_gf16.py [--samples N] [--seed S] [--out FILE]
Exit non-zero if the L5 identity witness fails or any NMSE is negative.
"""
import argparse
import json
import math
import os
import platform
import sys
from datetime import datetime, timezone

import numpy as np

# --- repo paths -------------------------------------------------------------
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "conformance"))

import gf16_ref as gf  # noqa: E402  (E6M9 codec under test)

try:
    import ml_dtypes  # noqa: E402
    BF16 = ml_dtypes.bfloat16
except Exception as exc:  # pragma: no cover
    print("ERROR: ml_dtypes with bfloat16 is required:", exc)
    sys.exit(2)

PROTOCOL_VERSION = "1.0.0"
PHI = (1.0 + math.sqrt(5.0)) / 2.0

# --- L5 identity witness (protocol section 5) -------------------------------
def identity_witness():
    w1 = abs(PHI * PHI - (PHI + 1.0))
    w2 = abs(PHI * PHI + 1.0 / (PHI * PHI) - 3.0)
    ok = (w1 < 1e-15) and (w2 < 1e-15)
    return ok, w1, w2


# --- round-trip codecs ------------------------------------------------------
def gf16_roundtrip(x):
    """real -> GF16 -> real, elementwise, using the SSOT codec."""
    out = np.empty_like(x, dtype=np.float64)
    flat = x.reshape(-1)
    o = out.reshape(-1)
    for i in range(flat.shape[0]):
        o[i] = gf.decode(gf.encode(float(flat[i])))
    return out


def bf16_roundtrip(x):
    return x.astype(BF16).astype(np.float64)


def fp16_roundtrip(x):
    return x.astype(np.float16).astype(np.float64)


# --- metrics ----------------------------------------------------------------
def nmse(x, q):
    """E[(x-q)^2] / E[x^2] over finite, non-overflowed samples."""
    mask = np.isfinite(q) & np.isfinite(x)
    x = x[mask]
    q = q[mask]
    denom = np.mean(x * x)
    if denom == 0.0:
        return 0.0, int(mask.sum())
    return float(np.mean((x - q) ** 2) / denom), int(mask.sum())


def max_abs_err(x, q):
    mask = np.isfinite(q) & np.isfinite(x)
    if mask.sum() == 0:
        return float("nan")
    return float(np.max(np.abs(x[mask] - q[mask])))


def ulp_like(x, q):
    """kappa-approx style scale-invariant relative error mean (P3109 idea):
    mean of |x-q| / max(|x|, tiny). Dimensionless, comparable across formats."""
    mask = np.isfinite(q) & np.isfinite(x) & (x != 0.0)
    if mask.sum() == 0:
        return float("nan")
    return float(np.mean(np.abs(x[mask] - q[mask]) / np.abs(x[mask])))


# Representable finite range of each format (for the dynamic-range story).
# GF16 max normal ~ (2-2^-9)*2^31; bf16 ~ 3.39e38; fp16 ~ 65504.
GF16_MAX = (2.0 - 2.0 ** -9) * (2.0 ** 31)
GF16_MIN_NORMAL = 2.0 ** (1 - 31)
BF16_MAX = float(ml_dtypes.finfo(BF16).max)
FP16_MAX = float(np.finfo(np.float16).max)


def overflow_rate(x, fmax):
    """Fraction of |x| that exceeds the format's max finite magnitude
    (i.e. saturates/overflows). The dynamic-range disadvantage that NMSE
    on surviving samples cannot show."""
    nz = x[x != 0.0]
    if nz.size == 0:
        return 0.0
    return float(np.mean(np.abs(nz) > fmax))


# --- reference distributions (protocol section 4) ---------------------------
def sample_distribution(tag, n, rng):
    if tag == "D_NORM":
        return rng.standard_normal(n)
    if tag == "D_LOG":
        e = rng.uniform(-10.0, 10.0, n)
        sign = rng.choice([-1.0, 1.0], n)
        return sign * (2.0 ** e)
    if tag == "D_RELU":
        return np.maximum(0.0, rng.standard_normal(n))
    if tag == "D_PHI":
        return rng.normal(PHI, 1.0 / PHI, n)
    if tag == "D_DEEP":
        k = int(0.7 * n)
        a = rng.standard_normal(k)
        e = rng.uniform(-10.0, 10.0, n - k)
        sign = rng.choice([-1.0, 1.0], n - k)
        b = sign * (2.0 ** e)
        return np.concatenate([a, b])
    if tag == "D_WIDE":
        # Protocol extension (documented): log2|x| ~ U(-28, 28). Probes the wide
        # dynamic range where GF16's 6-bit exponent (max ~2^31) is expected to
        # LOSE to bf16's 8-bit exponent. Included for honesty so the study is
        # not cherry-picked to favour GF16 (see WS-03, protocol section 3.3).
        e = rng.uniform(-28.0, 28.0, n)
        sign = rng.choice([-1.0, 1.0], n)
        return sign * (2.0 ** e)
    raise ValueError(tag)


DISTRIBUTIONS = ["D_NORM", "D_LOG", "D_RELU", "D_PHI", "D_DEEP", "D_WIDE"]


def run(samples, seed):
    ok, w1, w2 = identity_witness()
    if not ok:
        print(f"L5 IDENTITY WITNESS FAILED: w1={w1:.3e} w2={w2:.3e}")
        sys.exit(1)
    print(f"L5 identity witness OK (w1={w1:.2e}, w2={w2:.2e})")

    rng = np.random.default_rng(seed)
    results = {}
    any_negative = False

    print(f"\n{'dist':8} {'NMSE_GF16':>12} {'NMSE_BF16':>12} {'NMSE_FP16':>12} "
          f"{'GF16/BF16':>10} {'GF16/FP16':>10}")
    print("-" * 70)

    for tag in DISTRIBUTIONS:
        x = sample_distribution(tag, samples, rng).astype(np.float64)
        qg = gf16_roundtrip(x)
        qb = bf16_roundtrip(x)
        qf = fp16_roundtrip(x)

        ng, ng_n = nmse(x, qg)
        nb, nb_n = nmse(x, qb)
        nf, nf_n = nmse(x, qf)
        for v in (ng, nb, nf):
            if v < 0:
                any_negative = True

        rg_b = (ng / nb) if nb > 0 else float("inf")
        rg_f = (ng / nf) if nf > 0 else float("inf")

        results[tag] = {
            "samples_finite_gf16": ng_n,
            "samples_finite_bf16": nb_n,
            "samples_finite_fp16": nf_n,
            "NMSE_GF16": ng,
            "NMSE_BF16": nb,
            "NMSE_FP16": nf,
            "ratio_GF16_over_BF16": rg_b,
            "ratio_GF16_over_FP16": rg_f,
            "max_abs_err_GF16": max_abs_err(x, qg),
            "max_abs_err_BF16": max_abs_err(x, qb),
            "max_abs_err_FP16": max_abs_err(x, qf),
            "ulp_like_GF16": ulp_like(x, qg),
            "ulp_like_BF16": ulp_like(x, qb),
            "ulp_like_FP16": ulp_like(x, qf),
            "overflow_rate_GF16": overflow_rate(x, GF16_MAX),
            "overflow_rate_BF16": overflow_rate(x, BF16_MAX),
            "overflow_rate_FP16": overflow_rate(x, FP16_MAX),
        }
        print(f"{tag:8} {ng:12.3e} {nb:12.3e} {nf:12.3e} {rg_b:10.3f} {rg_f:10.3f}")

    if any_negative:
        print("INVARIANT FAILED: a NMSE value is negative")
        sys.exit(1)

    print("\noverflow (saturation) rate per distribution -- dynamic-range story:")
    print(f"{'dist':8} {'GF16':>10} {'BF16':>10} {'FP16':>10}")
    print("-" * 42)
    for tag in DISTRIBUTIONS:
        r = results[tag]
        print(f"{tag:8} {r['overflow_rate_GF16']:10.4f} "
              f"{r['overflow_rate_BF16']:10.4f} {r['overflow_rate_FP16']:10.4f}")

    manifest = {
        "protocol_version": PROTOCOL_VERSION,
        "representable_max": {"GF16": GF16_MAX, "BF16": BF16_MAX, "FP16": FP16_MAX},
        "protocol_doc": "docs/GF16_BFLOAT16_NMSE_PROTOCOL.md",
        "codec_gf16": "conformance/gf16_ref.py (E6M9, BIAS=31, EXP_BITS=6, MANT_BITS=9)",
        "rng_family": "numpy.random.default_rng (PCG64)",
        "rng_seed": seed,
        "samples_per_distribution": samples,
        "bf16_subnormal_policy": "ieee",
        "bf16_impl": f"ml_dtypes.bfloat16 {getattr(ml_dtypes, '__version__', '?')}",
        "fp16_impl": f"numpy.float16 {np.__version__}",
        "seal": "unsealed",
        "seal_note": "host-only measurement; informational, NOT a silicon certifying claim (protocol section 8)",
        "L5_identity_witness": {"phi2_eq_phi_plus_1": w1, "phi2_plus_inv_eq_3": w2, "passed": True},
        "runner": {
            "host_arch": platform.machine(),
            "python": platform.python_version(),
            "platform": platform.platform(),
        },
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "headline": "ratio_GF16_over_BF16 is the protocol headline; <1 means GF16 closer to reference.",
        "results": results,
    }
    return manifest


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--samples", type=int, default=2_000_000,
                    help="samples per distribution (protocol default 10M; host default 2M)")
    ap.add_argument("--seed", type=int, default=2718281)
    ap.add_argument("--out", default=os.path.join(HERE, "nmse_manifest.json"))
    args = ap.parse_args()

    manifest = run(args.samples, args.seed)
    with open(args.out, "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"\nmanifest written: {args.out}")


if __name__ == "__main__":
    main()
