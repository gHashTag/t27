# gf1024 strict SW-bitexact witness chain (exact-dyadic target)

Promotion of `gf1024` (GoldenFloat1024: S1 E391 M632, BIAS=281474976710655=2^390-1) from
`bitexact_selfconsistent` to strict SW-`bitexact` in
`conformance/vectors/INDEX_all_formats.json`.

Status tag: **[verified SW]**. This is a software chain (an analytic
separation-bound + two structurally independent exact decode paths). It is NOT an
on-silicon Tier-E claim. HW-decode / HW-compute remain [REQUIRES USER ACTION]
(a 4/4 Tier-E chain on AX7203, tracked on issue #199 of the trinity-fpga repo).

## Why NO FP-target (contrast with gf48)

Phase-A formats (gf4..gf32) decode into IEEE binary32 (23-bit mantissa). gf48 has
M=29 and lowers EXACTLY into binary64 (52 bits) with RNE only on the FP64-subnormal
edge, so its proof used a fixed-width FP64 RTL bit-model + an iverilog witness.

gf1024 has **M=78 > 52**, so binary64 CANNOT hold the mantissa exactly and a binary64
lowering WOULD round. The pack therefore keeps every gf1024 value as an EXACT dyadic
literal `A*2^B` (`value_encoding=dyadic`), and the conformance target is the exact
rational value itself. Consequently the decode path has NO rounding: every
representable gf1024 code maps to an exact dyadic rational (see `SEPARATION_BOUND.md`,
Lemma sec. 3). The iverilog-FP64 witness is not applicable, and no RTL bit-model is
required, because there is no rounding to model.

## Two independent witnesses (both pass in-sandbox)

1. **Dyadic independent decoder** -- `../../gf_wide_independent_witness.py`
   run with the pack path:
   ```
   python3 conformance/gf_wide_independent_witness.py \
     conformance/vectors/gf1024_conformance_v0.json
   ```
   Expected: `15/15 bit-exact (abs_error=0)`.
   Internal representation: integer `(2^M+mant)` normalized to `(odd, shift)`.

2. **Golden Fraction oracle** -- `gf1024_decode_ref.py`
   Exact-rational decode (fractions.Fraction significand + symbolic integer shift),
   checked against the pack; DIFFERENT internal representation from witness 1:
   ```
   python3 conformance/witness/gf1024/gf1024_decode_ref.py \
     conformance/vectors/gf1024_conformance_v0.json
   ```
   Expected: `15/15 exact`.

## Cross-check of the two paths (representative sweep)

`cross_check_representative.py` runs BOTH decode paths over a large representative
set (5-class + exponent boundaries + full-mantissa edges + deep-underflow/overflow
+ 200k deterministic random, seed=96) and asserts they agree bit-exactly. 2^128
exhaustive is infeasible; this is a falsifiable representative sweep, and the
analytic lemma covers the whole domain.

```
cd conformance/witness/gf1024 && python3 cross_check_representative.py
```
Expected: `201512/201512 agree` (count is deterministic for seed=96).

## Memory note (why no giant integers)

The gf1024 exponent range is +-2^390, so `2^(exp-BIAS)` must NEVER be materialized as
an integer (2^34e9 ~ 4 GB -> OOM). Both witnesses keep the huge power of two
symbolic (in `shift`) and only ever build small numerators (<= 2^60). Peak RSS in
the sandbox is ~14 MB.

## Files

| File | Role |
|---|---|
| `gf1024_decode_ref.py` | Golden Fraction decode oracle (witness 2). |
| `cross_check_representative.py` | Cross-check of the two independent paths over the representative sweep. |
| `SEPARATION_BOUND.md` | Analytic separation-bound (zero-rounding lemma + honesty boundaries). |

## Provenance

- Anchor identity: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
- Preprint: arXiv:2606.05017 (GoldenFloat). SSOT:
  gHashTag/t27 specs/numeric/formats_catalog.t27.
- gf48 (`conformance/witness/gf48_fp64/`) is the structural model for the pack
  `witnesses` array used here.
