# gf128 strict SW-bitexact: analytic separation-bound

Format: **GoldenFloat128** -- S1 E49 M78, BIAS = 281474976710655 = 2^48 - 1
(IEC 60559 interchange bias 2^(E-1)-1). SSOT: `specs/numeric/formats_catalog.t27`.

Status tag: **[verified SW]**. This is a software argument (an analytic lemma plus
two independent exact decode paths). It is NOT an on-silicon Tier-E claim.
HW-decode / HW-compute remain [REQUIRES USER ACTION] (a 4/4 Tier-E chain on
AX7203, tracked on issue #199 of the trinity-fpga repo).

---

## 1. Why a separation-bound (not only an oracle)

gf48 (M=29 <= 52) lowered into IEEE binary64, so its bit-exact proof involved RNE
rounding on the subnormal edge and was checked by a fixed-width RTL bit-model plus
an iverilog run (lesson 04.07: Python arbitrary-width does NOT catch fixed-width
bugs).

gf128 has **M=78 > 52** -- the mantissa does NOT fit in binary64 (52 bits). A
binary64 lowering WOULD round. Therefore the pack keeps every value as an EXACT
dyadic literal `A*2^B` (`value_encoding=dyadic`), and the conformance target is the
exact rational value itself, not an FP lowering. The question the separation-bound
closes: **is there any place in the gf128 decode path where rounding could be
non-deterministic or lose precision?** The answer is no, proven below without
hardware and without an infeasible 2^128 exhaustive sweep.

---

## 2. Decode law (5 classes, parametric in E, M, BIAS)

Bit-layout LSB-aligned: `[sign:1][exp:E][mant:M]`, total 1+E+M = 128 bits.
EXP_MAX = 2^E - 1.

| Class | Condition | Value |
|---|---|---|
| Inf | exp=EXP_MAX, mant=0 | (-1)^s * inf |
| NaN | exp=EXP_MAX, mant!=0 | quiet NaN (payload irrelevant) |
| Zero | exp=0, mant=0 | (-1)^s * 0 |
| Subnormal | exp=0, mant!=0 | (-1)^s * (mant / 2^M) * 2^(1-BIAS) |
| Normal | otherwise | (-1)^s * (1 + mant / 2^M) * 2^(exp-BIAS) |

---

## 3. Lemma (exact representability -- "zero rounding error")

**Claim.** For any 128-bit code `raw`, the finite decoded gf128 value is an exact
dyadic rational of the form `odd * 2^k` (odd is odd or 0, k in Z), and this
representation is unique. Consequently the gf128 decode path contains NO rounding,
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
mant an integer in [1, 2^M-1], integer exponent -> dyadic.

*Zero.* V = 0 = 0*2^0 -- dyadic (degenerate edge).

The canonical form `odd * 2^k` is obtained by factoring all powers of two out of the
integer numerator into the exponent; it is unique (a nonzero rational has exactly
one representation odd*2^k with odd numerator). QED.

**Corollary (separation-bound, strong form).** Let Delta be the minimum distance
between two distinct representable gf128 values (the ULP at a given exponent =
2^(exp-BIAS-M)). Since each value is represented EXACTLY (sec. 3), the maximum
decode rounding error is 0 < Delta/2 for every exponent. The condition
"separation of representable values strictly exceeds the maximum rounding error"
holds with room to spare (the error is exactly 0). Rounding is therefore
deterministic and cannot select a neighbouring value: the decode map raw -> value
is injective up to signed-zero / NaN class and is independent of any rounding mode.

Note: the argument does NOT materialize 2^BIAS (~2^48 bits). It operates on an
integer numerator (<= 2^80) and a symbolic integer exponent. This is exactly what
both witness implementations do (sec. 4), so their execution fits in sandbox memory
(peak ~18 MB).

---

## 4. Two independent exact decode paths (witness set)

The lemma is necessary but not sufficient: the SPECIFIC pack vectors must also match
an independently computed value. Two structurally DIFFERENT exact decode
implementations provide this, agreeing with each other and with the pack:

1. **Dyadic independent witness** -- `conformance/gf_wide_independent_witness.py`
   (already in-repo; parametric in (e,m,bias) from the catalog; written from
   scratch, does NOT reuse the encoder). Internal representation: an integer
   `(2^M+mant)` normalized to `(odd, shift)` by factoring out twos.
   Result: **15/15 pack vectors bit-exact, abs_error=0**.

2. **Golden Fraction oracle** -- `conformance/witness/gf128/gf128_decode_ref.py`
   (new; independent implementation). DIFFERENT internal representation: the
   significand is a genuine `fractions.Fraction` (1 + mant/2^M), while the large
   exponent is carried as a separate symbolic integer `shift`; final
   canonicalization via `sig.numerator / sig.denominator * 2^shift`.
   Result: **15/15 pack vectors exact, abs_error=0**.

**Cross-check of the two paths** (`cross_check_representative.py`): on the 15 pack
vectors plus a representative set (5 classes + exponent boundaries + full-mantissa
edges + deep-underflow/overflow + 200k deterministic random, seed=96) --
**201512/201512 agreements**, abs_error=0. 2^128 exhaustive is infeasible; this is a
falsifiable representative sweep, and lemma sec. 3 covers the whole domain
analytically.

---

## 5. Why this is strict bitexact, not selfconsistent

`bitexact_selfconsistent` = one decode law, no second independent witness. Here we
have: (a) an analytic lemma of zero rounding error (decode with no rounding at all),
and (b) TWO structurally different exact paths agreeing with each other and with the
pack. This meets the strict SW-`bitexact` definition (independent decoder +
abs_error=0 + second witness). Promotion: gf128 `bitexact_selfconsistent -> bitexact`.

## 6. Honesty boundaries

- This is an **SW** argument. NOT Tier-E (no CI GREEN + bitstream SHA256 + UART
  N/N + IDCODE 0x13636093). gf128 decode-HW / compute-HW = [REQUIRES USER ACTION].
- The iverilog-FP64 witness (as used for gf48) is **not applicable** here: M=78 >
  52, the target type is not binary64 but the exact dyadic value; no fixed-width FP
  datapath participates in gf128 SW conformance. An RTL bit-model is not required
  because there is no rounding.
- No categorical superiority claims about gf128 over any other format. The
  GoldenFloat ladder earns its place through breadth and toolchain coherence, not
  per-rung superiority; takum (Hunhold 2024, arXiv:2412.20273) is the standing
  counterexample and is not suppressed.

Anchor: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
Preprint: arXiv:2606.05017 (GoldenFloat). SSOT: gHashTag/t27
specs/numeric/formats_catalog.t27.
