# gf256 strict SW-bitexact witness chain (exact-dyadic target)

Promotion of `gf256` (GoldenFloat256: S1 E97 M158, BIAS=79228162514264337593543950335=2^96-1)
from `bitexact_selfconsistent` to strict SW-`bitexact` in
`conformance/vectors/INDEX_all_formats.json`.

Status tag: **[verified SW]**. This is a software chain (an analytic
separation-bound + two structurally independent exact decode paths). It is NOT an
on-silicon Tier-E claim. gf256 is a software interchange format (storage class
`u256_software`): it has NO RTL, so there is no decode-HW / compute-HW cell to close
and the Tier-E ceiling (71/83) is unaffected.

## Why NO FP-target (contrast with gf48)

Phase-A formats (gf4..gf32) decode into IEEE binary32 (23-bit mantissa). gf48 has
M=29 and lowers EXACTLY into binary64 (52 bits) with RNE only on the FP64-subnormal
edge, so its proof used a fixed-width FP64 RTL bit-model + an iverilog witness.

gf256 has **M=158 >> 52**, so binary64 CANNOT hold the mantissa exactly and a
binary64 lowering WOULD round. The pack therefore keeps every gf256 value as an EXACT
dyadic literal `A*2^B` (`value_encoding=dyadic`), and the conformance target is the
exact rational value itself. Consequently the decode path has NO rounding: every
representable gf256 code maps to an exact dyadic rational (see `SEPARATION_BOUND.md`,
Lemma sec. 3). No FP-lowering witness and no RTL bit-model is required, because there
is no rounding to model.

## Two independent witnesses (both pass in-sandbox)

1. **Dyadic independent decoder** -- `../../gf_wide_independent_witness.py`
   run with the pack path:
   ```
   python3 conformance/gf_wide_independent_witness.py \
     conformance/vectors/gf256_conformance_v0.json
   ```
   Expected: `2021/2021 bit-exact (abs_error=0)`.
   Internal representation: integer `(2^M+mant)` normalized to `(odd, shift)`.

2. **Golden Fraction oracle** -- `gf256_decode_ref.py`
   Exact-rational decode (fractions.Fraction significand + symbolic integer shift),
   checked against the pack; DIFFERENT internal representation from witness 1:
   ```
   python3 conformance/witness/gf256/gf256_decode_ref.py \
     conformance/vectors/gf256_conformance_v0.json
   ```
   Expected: `2021/2021 exact`.

## Cross-check of the two paths (representative sweep)

`cross_check_representative.py` runs BOTH decode paths over a large representative
set (5-class + exponent boundaries + full-mantissa edges + deep-underflow/overflow
+ 200k deterministic random, seed=256) and asserts they agree bit-exactly. 2^256
exhaustive is infeasible; this is a falsifiable representative sweep, and the
analytic lemma covers the whole domain.

```
cd conformance/witness/gf256 && python3 cross_check_representative.py
```
Expected: `201512/201512 agree` (count is deterministic for seed=256).

## Memory note (why no giant integers)

The gf256 exponent range is +-2^96, so `2^(exp-BIAS)` must NEVER be materialized as
an integer (would OOM). Both witnesses keep the huge power of two symbolic (in
`shift`) and only ever build small numerators (<= 2^159). Peak RSS in the sandbox is
a few MB.

## Bias disambiguation (important)

The gf256 spec carries a descriptive `PHI_BIAS` metadata field whose provenance was
audited (`wave_audit/gf256_bias_audit_2026-07-05.md`). That field is NOT part of the
decode path. The decode uses only the closed-form interchange bias
BIAS = 2^(E-1)-1 = 2^96-1, the identical rule already applied to the promoted ladder
steps gf128 (2^48-1) and gf512 (2^194-1). No decoded value depends on PHI_BIAS.

## Files

| File | Role |
|---|---|
| `gf256_decode_ref.py` | Golden Fraction decode oracle (witness 2). |
| `cross_check_representative.py` | Cross-check of the two independent paths over the representative sweep. |
| `SEPARATION_BOUND.md` | Analytic separation-bound (zero-rounding lemma + honesty boundaries). |

## Provenance

- Anchor identity: phi^2 + phi^-2 = 3. Vasilev (gHashTag), ORCID 0009-0008-4294-6159,
  admin@t27.ai.
- Preprint: arXiv:2606.05017 (GoldenFloat). SSOT:
  gHashTag/t27 specs/numeric/gf256.t27 and specs/numeric/formats_catalog.t27.
- gf48 (`conformance/witness/gf48_fp64/`) is the structural model for the pack
  `witnesses` array used here.
