# gf256 strict SW-bitexact: analytic separation-bound

Format: **GoldenFloat256** -- S1 E97 M158, BIAS = 79228162514264337593543950335 =
2^96 - 1 (IEC 60559 interchange bias 2^(E-1)-1). SSOT: `specs/numeric/gf256.t27`
and `specs/numeric/formats_catalog.t27`. Preprint anchor: arXiv:2606.05017.
Anchor identity: phi^2 + phi^-2 = 3.

Status tag: **[verified SW]**. This is a software argument (an analytic lemma plus
two independent exact decode paths). It is NOT an on-silicon Tier-E claim.
HW-decode / HW-compute for gf256 remain [REQUIRES USER ACTION] -- in fact gf256 has
NO RTL at all (storage class `u256_software`), so it is a software-only interchange
format and there is no HW cell to close (a 4/4 Tier-E chain on AX7203 does not apply;
Tier-E work is tracked on issue #199 of the trinity-fpga repo for formats that have RTL).

---

## 1. Why a separation-bound (not only an oracle)

gf48 (M=29 <= 52) lowered into IEEE binary64, so its bit-exact proof involved RNE
rounding on the subnormal edge and was checked by a fixed-width RTL bit-model plus
an iverilog run (lesson 04.07: Python arbitrary-width does NOT catch fixed-width
bugs).

gf256 has **M=158 >> 52** -- the mantissa does NOT fit in binary64 (52 bits). A
binary64 lowering WOULD round. Therefore the pack keeps every value as an EXACT
dyadic literal `A*2^B` (`value_encoding=dyadic`), and the conformance target is the
exact rational value itself, not an FP lowering. The question the separation-bound
closes: **is there any place in the gf256 decode path where rounding could be
non-deterministic or lose precision?** The answer is no, proven below without
hardware and without an infeasible 2^256 exhaustive sweep.

---

## 2. Decode law (5 classes, parametric in E, M, BIAS)

Bit-layout LSB-aligned: `[sign:1][exp:E][mant:M]`, total 1+E+M = 1+97+158 = 256 bits.
EXP_MAX = 2^E - 1 = 2^97 - 1.

| Class | Condition | Value |
|---|---|---|
| Inf | exp=EXP_MAX, mant=0 | (-1)^s * inf |
| NaN | exp=EXP_MAX, mant!=0 | quiet NaN (payload irrelevant) |
| Zero | exp=0, mant=0 | (-1)^s * 0 |
| Subnormal | exp=0, mant!=0 | (-1)^s * (mant / 2^M) * 2^(1-BIAS) |
| Normal | otherwise | (-1)^s * (1 + mant / 2^M) * 2^(exp-BIAS) |

BIAS is used only as the closed-form interchange bias 2^(E-1)-1 = 2^96-1. It is the
identical rule already applied to the promoted ladder steps gf128 (2^(49-1)-1) and
gf512 (2^(195-1)-1). The descriptive PHI_BIAS metadata that appears in the spec is
NOT part of the decode path and plays no role in any decoded value (see the bias
audit note `wave_audit/gf256_bias_audit_2026-07-05.md`).

---

## 3. Lemma (exact representability -- "zero rounding error")

**Claim.** For any 256-bit code `raw`, the finite decoded gf256 value is an exact
dyadic rational of the form `odd * 2^k` (odd is odd or 0, k in Z), and this
representation is unique. Consequently the gf256 decode path contains NO rounding,
and abs_error = 0 holds identically (not statistically) over the whole domain.

**Proof.** Consider the finite classes.

*Normal.*
  V = (1 + mant/2^M) * 2^(exp-BIAS)
    = (2^M + mant) * 2^(exp - BIAS - M).
The numerator `2^M + mant` is an integer in [2^M, 2^(M+1)-1] (since 0 <= mant <=
2^M-1). The exponent `exp - BIAS - M` is an integer. So V = integer * 2^integer ->
dyadic.

*Subnormal.*
  V = (mant/2^M) * 2^(1-BIAS)
    = mant * 2^(1 - BIAS - M),
with mant an integer in [1, 2^M-1] and the exponent an integer -> dyadic.

*Zero.* V = 0, trivially dyadic.

Canonicalizing `num * 2^k` by stripping trailing zero bits of `num` yields a unique
`odd * 2^k'` with `odd` odd (or 0). QED.

There is NO step where a value is projected onto a coarser grid; hence no rounding,
hence no tie-break rule to disambiguate, hence abs_error = 0 exactly. The exponent
magnitude reaches ~2^96, but neither path materializes 2^(exp-BIAS) as an integer;
the value is kept as (significand, shift) / (odd, shift) with `shift` a plain
Python int, so there is no overflow and no OOM.

---

## 4. Two independent exact witnesses

To make the lemma falsifiable in code, two decoders were written INDEPENDENTLY, with
structurally different internal representations, and required to agree bit-exactly:

- **Witness 1 (dyadic integer normalizer):** `conformance/gf_wide_independent_witness.py`
  -- forms the integer `2^M + mant` (or `mant` for subnormals), then normalizes the
  pair `(num, shift)` to canonical `(odd, shift)` by factoring out powers of two.

- **Witness 2 (Fraction significand + symbolic shift):** `gf256_decode_ref.py` --
  carries the significand as a genuine `fractions.Fraction` in [0,2) (mantissa
  arithmetic only, tiny denominator <= 2^M) and keeps the huge power of two as a
  separate symbolic integer `shift`; canonicalizes to a dyadic pair only for the
  final exact comparison.

These are two different decompositions of the same exact rational. Their agreement is
a genuine second witness (not a self-check of one decode law).

---

## 5. Empirical gates (all GREEN)

1. **Gate 0 (parametric-law reproduces existing pack):** the parametric decode law
   with (E=97, M=158, BIAS=2^96-1) reproduces the pre-existing 8-vector
   `gf256_conformance_v0.json` pack -> 8/8.

2. **Witness-1 vs wide pack:** `gf_wide_independent_witness.py` over the 2021-vector
   dyadic pack -> 2021/2021 bit-exact.

3. **Witness-2 vs wide pack:** `gf256_decode_ref.py` (Fraction oracle) over the same
   pack -> 2021/2021 exact, abs_error=0.

4. **Cross-check (witness-1 == witness-2):** `cross_check_representative.py` over a
   201512-code representative sweep (5-class + exponent boundaries + full-mantissa
   edges + deterministic random seed=256) -> 201512/201512 agree, abs_error=0.

2^256 exhaustive is infeasible; the representative sweep is falsifiable (any single
disagreement fails the gate). Combined with the analytic lemma (Section 3, which
holds identically over the whole domain), this closes strict SW-bitexact for gf256.

---

## 6. Honest scope

- This is **SW-bitexact only** ([verified SW]). No categorical claims.
- gf256 is a software interchange format (`u256_software`): there is no RTL, so there
  is no decode-HW or compute-HW Tier-E cell to close for it. The Tier-E ceiling
  (71 of the formats then catalogued, on XC7A200T-2FBG484I, IDCODE 0x13636093)
  is unaffected by this promotion.
- The catalog count is a CI invariant (it has grown since this witness was
  written; 109 at v3, Sep 2026). This promotion moves gf256 from `bitexact_selfconsistent` to
  `bitexact` (INDEX 74/1/8 -> 75/0/8 at the time).
- Strict SW-bitexact (independent 2nd witness, abs_error=0) is a stronger tier than
  `bitexact_selfconsistent` (one decode law, no 2nd witness) and than `structural`.
