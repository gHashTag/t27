#!/usr/bin/env python3
"""Honest accuracy study: GF16 (1/6/9, bias31) vs IEEE fp16 (1/5/10) vs bfloat16 (1/8/7).
Round-trip NMSE + max relative error + dynamic range, over several distributions.
Matches the RTL gf16 format semantics (round-to-nearest-even on the 9-bit mantissa)."""
import numpy as np

BIAS = 31; MBITS = 9; EBITS = 6
EMAX = (1 << EBITS) - 1  # 63

def gf16_round(x):
    """Encode->decode a real array through GF16 (round to nearest). Vectorized."""
    x = np.asarray(x, dtype=np.float64)
    out = np.zeros_like(x)
    nz = x != 0
    s = np.sign(x[nz]); a = np.abs(x[nz])
    e = np.floor(np.log2(a)).astype(np.int64)          # unbiased exponent
    expf = e + BIAS
    frac = a / np.power(2.0, e.astype(np.float64)) - 1.0  # in [0,1)
    mant = np.round(frac * (1 << MBITS)).astype(np.int64)
    carry = mant >> MBITS                               # mantissa rounded up to 2.0
    expf = expf + carry
    mant = np.where(carry > 0, 0, mant)
    val = (1.0 + mant / (1 << MBITS)) * np.power(2.0, (expf - BIAS).astype(np.float64))
    val = np.where(expf <= 0, 0.0, val)                 # underflow -> 0
    val = np.where(expf >= EMAX, np.power(2.0, float(EMAX - BIAS)), val)  # clamp overflow
    res = np.zeros(nz.sum()); res[:] = s * val
    out[nz] = res
    return out

def bf16_round(x):
    x = np.asarray(x, dtype=np.float32)
    u = x.view(np.uint32)
    # round-to-nearest-even truncation to 7 mantissa bits
    rounding = ((u >> 16) & 1) + 0x7FFF
    u = (u + rounding) & 0xFFFF0000
    return u.view(np.float32).astype(np.float64)

def fp16_round(x):
    return np.asarray(x, dtype=np.float16).astype(np.float64)

def nmse(ref, approx):
    ref = np.asarray(ref, np.float64); approx = np.asarray(approx, np.float64)
    return np.mean((approx - ref) ** 2) / np.mean(ref ** 2)

def maxrel(ref, approx):
    m = ref != 0
    return np.max(np.abs((approx[m] - ref[m]) / ref[m]))

rng = np.random.default_rng(0)
dists = {
    "N(0,1) weights":        rng.standard_normal(2_000_000),
    "N(0,1)*8 (wide)":       8.0 * rng.standard_normal(2_000_000),
    "lognormal (acts>0)":    rng.lognormal(0, 1, 2_000_000),
    "uniform[-1,1]":         rng.uniform(-1, 1, 2_000_000),
}
print(f"{'format':10} {'mant':>4} {'exp':>3}  {'rel-prec':>9}  | per-distribution NMSE (dB) / max-rel%")
print(f"{'GF16':10} {9:>4} {6:>3}  {2**-10*100:>8.3f}%")
print(f"{'fp16':10} {10:>4} {5:>3}  {2**-11*100:>8.3f}%")
print(f"{'bf16':10} {7:>4} {8:>3}  {2**-8*100:>8.3f}%")
print("-" * 96)
for name, data in dists.items():
    print(f"\n# {name}  (range |x|: {np.abs(data[data!=0]).min():.2e} .. {np.abs(data).max():.2e})")
    for fmt, fn in [("GF16", gf16_round), ("fp16", fp16_round), ("bf16", bf16_round)]:
        approx = fn(data)
        n = nmse(data, approx); mr = maxrel(data, approx)
        print(f"  {fmt:6}  NMSE={10*np.log10(n):8.2f} dB   max-rel={mr*100:7.3f}%")

print("\n# Dynamic range (normal, no denormal):")
print(f"  GF16: 2^{1-BIAS} .. 2^{EMAX-1-BIAS}  = {2.0**(1-BIAS):.2e} .. {2.0**(EMAX-1-BIAS):.2e}")
print(f"  fp16: 2^-14 .. ~6.5e4   ;  bf16: ~1e-38 .. ~3e38 (fp32 range, 7-bit mant)")
