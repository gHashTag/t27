# repro/numerics -- L4/L5 differential accuracy oracle for GF16

This directory closes the L4 (differential) and L5 (comparative) rungs of the
numeric validation ladder for GoldenFloat **GF16** (E6M9), per
`docs/GF16_BFLOAT16_NMSE_PROTOCOL.md` and `docs/NUMERICS_VALIDATION.md`.

It produces **measured** round-trip accuracy numbers for GF16 versus
**bfloat16** and **IEEE float16** -- the two 16-bit alternatives of the same
memory footprint -- over the protocol's reference distributions.

> **R5-HONEST.** These are **host-measured, unsealed** numbers. Per protocol
> section 8 they are **informational, not a silicon certifying claim**. No
> result here asserts a measured silicon NMSE for any product. `D_PHI` is an
> identity-anchored sanity check (L5), **not** a superiority claim.

## How to reproduce

```
python repro/numerics/nmse_gf16.py            # 2,000,000 samples/distribution
python repro/numerics/nmse_gf16.py --samples 10000000 --seed 2718281
```

- GF16 codec under test: `conformance/gf16_ref.py` (BIAS=31, EXP_BITS=6, MANT_BITS=9).
- bf16: `ml_dtypes.bfloat16`; fp16: `numpy.float16`.
- The run aborts non-zero if the L5 identity witness
  (`|phi^2-(phi+1)|<1e-15`, `|phi^2+phi^-2-3|<1e-15`) fails or any NMSE < 0.
- Output manifests:
  - `nmse_manifest.json` -- the **rich** manifest (fields per protocol section 6:
    all six distributions including `D_WIDE`, ULP-like metric, overflow rates,
    representable-max table). Intentionally NOT schema-bound.
  - `nmse_manifest_protocol_v1.json` -- the **certifying** manifest, strictly
    conforming to `schemas/nmse-protocol-v1.json` (`additionalProperties:false`;
    only the five schema distributions `D_NORM/D_LOG/D_RELU/D_PHI/D_DEEP`; each
    result is `nmse_gf16`/`nmse_bf16`/`ratio`). `D_WIDE` is deliberately absent.

## Sealing and the certifying manifest (R5-HONEST)

The certifying manifest carries a `seal_hash` field. It is set to the 64-hex
SHA-256 in `bootstrap/stage0/FROZEN_HASH` **only** when `--seal` is passed AND
the live seal source (`bootstrap/src/compiler.rs`) hashes to exactly that
digest under a pinned toolchain. In every other case it stays the literal
`"unsealed"`. The script never fabricates a seal: a host run on an unpinned
tree is honestly `unsealed` and informational only.

```
python repro/numerics/nmse_gf16.py --seal          # seals iff source matches FROZEN_HASH
python repro/numerics/validate_manifest.py         # re-validate the committed manifest
make -C repro repro-numerics-certify               # CI-facing validation step
```

`validate_manifest.py` validates the manifest against the schema AND enforces
the honesty rule: a non-`unsealed` `seal_hash` must equal the FROZEN_HASH
digest, or it exits non-zero. As committed, `seal_hash = "unsealed"` because
this manifest was produced on an unpinned host (the live compiler source does
not match the frozen digest) -- exactly the state protocol section 8 calls
informational, not a silicon certifying claim.

## Measured results (seed 2718281, 2,000,000 samples/distribution, unsealed host)

`NMSE(F) = E[(x - Q_F(x))^2] / E[x^2]`. Headline = `NMSE_GF16 / NMSE_BF16`
(< 1 means GF16 is closer to the reference).

| Distribution | NMSE GF16 | NMSE BF16 | NMSE FP16 | GF16/BF16 | GF16/FP16 |
|--------------|-----------|-----------|-----------|-----------|-----------|
| D_NORM  | 1.73e-07 | 2.76e-06 | 4.30e-08 | **0.063** | 4.02 |
| D_LOG   | 1.48e-07 | 2.35e-06 | 3.68e-08 | **0.063** | 4.02 |
| D_RELU  | 1.73e-07 | 2.76e-06 | 4.32e-08 | **0.063** | 4.01 |
| D_PHI   | 1.78e-07 | 2.85e-06 | 4.45e-08 | **0.063** | 4.00 |
| D_DEEP  | 1.48e-07 | 2.37e-06 | 3.65e-08 | **0.062** | 4.04 |
| D_WIDE  | 1.47e-07 | 2.36e-06 | 3.67e-08 | **0.063** | 4.02 |

Saturation / overflow rate (fraction of non-zero samples beyond a format's
finite max: GF16 ~4.29e9, BF16 ~3.39e38, FP16 ~6.55e4):

| Distribution | GF16 | BF16 | FP16 |
|--------------|------|------|------|
| D_WIDE (log2|x| ~ U(-28,28)) | 0.0000 | 0.0000 | **0.2144** |

(all other distributions: 0.0000 for every format)

## Honest interpretation (facts, not claims)

1. **GF16 vs bf16 (mantissa).** GF16 has 9 mantissa bits, bf16 has 7. Across
   every distribution GF16's round-trip NMSE is ~16x lower (ratio ~0.063).
   This is the expected consequence of the bit split, not a surprise.
2. **GF16 vs fp16 (mantissa).** fp16 has 10 mantissa bits, GF16 has 9, so fp16
   is ~4x more accurate near 1.0 (ratio ~4.0). GF16 does **not** beat fp16 on
   near-1.0 precision -- stated plainly.
3. **Dynamic range (exponent).** This is where the tradeoff flips. fp16's
   exponent saturates at ~65504, so on a wide-range distribution **21.4% of
   fp16 samples overflow**, while GF16 (max ~4.29e9, 6-bit exponent) and bf16
   (8-bit exponent) lose none. GF16's wider range is the price fp16 pays for
   its extra mantissa bit.
4. **Net.** GF16 sits between bf16 and fp16: more precise than bf16, wider
   range than fp16. This matches the E6M9 motivation of IBM DLFloat
   (ARITH 2019, DOI 10.1109/ARITH.2019.00023) and Popescu et al.
   (arXiv:2103.15940), which independently pick a 1/6/9 split.

## Cross-links

- Protocol: `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md`
- Validation ladder: `docs/NUMERICS_VALIDATION.md` (this fills L4/L5)
- Codec SSOT: `conformance/gf16_ref.py`, `conformance/FORMAT-SPEC-001.json`
- Preprint context: arXiv:2606.05017

phi^2 + phi^-2 = 3  |  TRINITY
