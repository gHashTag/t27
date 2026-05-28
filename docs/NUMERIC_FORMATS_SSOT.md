# GoldenFloat Numeric Formats — Single Source of Truth (SSOT)

**Status:** canonical. This document is the one source of truth for the
GoldenFloat (GF) numeric-format family. Every other artefact — the
`zig-golden-float` whitepaper, the GOLDEN CHAIN brochure chapter *"Numeric
Formats — A Short History and the GoldenFloat Family"*, slide decks, READMEs —
is **derived** from this file and the per-format specs it links. If they
disagree, this file and the linked `specs/numeric/*.t27` win.

Anchor identity: **φ² + φ⁻² = 3** (Trinity). Design rule: choose the
exponent:mantissa split so **E/M → 1/φ ≈ 0.6180339887** (see
[`phi_ratio.t27`](../specs/numeric/phi_ratio.t27)).

---

## 1. Verified family constants

These four columns are mathematically exact and independently re-derived.

| Format | S+E+M | Bits | BIAS = 2^(E−1)−1 | EXP_MAX = 2^E−1 | E/M | φ-distance \|E/M−1/φ\| | Spec |
|---|---|---|---|---|---|---|---|
| GF4  | 1+1+2  | 4  | 0       | 1        | 0.500 | 0.118 | [`gf4.t27`](../specs/numeric/gf4.t27) |
| GF8  | 1+3+4  | 8  | 3       | 7        | 0.750 | 0.132 | [`gf8.t27`](../specs/numeric/gf8.t27) |
| GF12 | 1+4+7  | 12 | 7       | 15       | 0.571 | 0.047 | [`gf12.t27`](../specs/numeric/gf12.t27) |
| GF16 | 1+6+9  | 16 | 31      | 63       | 0.667 | 0.049 | [`gf16.t27`](../specs/numeric/gf16.t27) |
| GF20 | 1+7+12 | 20 | 63      | 127      | 0.583 | 0.035 | [`gf20.t27`](../specs/numeric/gf20.t27) |
| GF24 | 1+9+14 | 24 | 255     | 511      | 0.643 | 0.025 | [`gf24.t27`](../specs/numeric/gf24.t27) |
| GF32 | 1+12+19| 32 | 2047    | 4095     | 0.632 | 0.014 | [`gf32.t27`](../specs/numeric/gf32.t27) |
| GF64 | 1+24+39| 64 | 8388607 | 16777215 | 0.615 | 0.003 | [`gf64.t27`](../specs/numeric/gf64.t27) |
| GF256| 1+97+158|256| 2⁹⁶−1 †  | 2⁹⁷−1 †  | 0.614 | 0.004 | *gamma; bias OPEN* |
| TF3  | 1+3+4  | 8  | 3       | 7        | 0.750 | 0.132 | [`tf3.t27`](../specs/numeric/tf3.t27) |
| GFTernary | 2-bit code | 2 | — | — | — | 0.000 | [`gfternary.t27`](../specs/numeric/gfternary.t27) |

† **GF256 caveat.** Its *stored* exponent-bias constant (≈ 2⁷¹) is unreconciled
with `2^(E−1)−1 = 2⁹⁶−1`, so the bias is **OPEN** (see §3); only its geometry
(97/158 split, φ-distance 0.004) is verified. The spec lives in
`tt-trinity-gamma/specs/fpga/gf256.t27`, **not** this repo.

The last two rows are **not** float-ladder rungs: **TF3** is an 8-bit
ternary-weight container reusing GF8's 1:3:4 geometry (so it shares GF8's
constants); **GFTernary** is the 2-bit {−φ, 0, +φ} limit — no exponent/mantissa
split (columns N/A), φ-distance 0 by construction.

Family base: [`goldenfloat_family.t27`](../specs/numeric/goldenfloat_family.t27).
Closest *split* to 1/φ is GF64 (0.003), then GF256 (0.004); GF12 (0.047) is best
among ≤16-bit. (GFTernary's 0.000 is by construction, not an E/M split.)
GF64 carries `PHI_BIAS = 8388608` — the one format where it coincides with
`EXP_MAX − BIAS = 2²³`.

## 2. PHI_BIAS — empirical per format (H_E), NOT a closed-form law

The published formula `PHI_BIAS = EXP_MAX − BIAS` is **RETRACTED**: it
reproduces GF64 only (8388608 = 2²³) and none of the other seven. No tested
closed form (2·BIAS−2, EXP_MAX−1, BIAS−1, 2^E−1, golden-ratio floor,
Lucas-indexed) reproduces all values. PHI_BIAS is defined **per format**:

| Format | PHI_BIAS | Status / justification |
|---|---|---|
| GF4  | 0       | empirical, minimal for 4-bit |
| GF8  | 1       | empirical (coincides L₁ = 1²) |
| GF12 | 2       | empirical |
| GF16 | 60      | **normative** (production; 2·BIAS−2 = 2⁶−4) |
| GF20 | 289     | empirical (coincides 17²) |
| GF24 | 1364    | empirical (coincides Lucas L₁₅) |
| GF32 | 0       | empirical (minimises MSE vs round-to-nearest-even) |
| GF64 | 8388608 | empirical (equals EXP_MAX − BIAS for this format only) |

Fibonacci/Lucas/square coincidences are **descriptive, not prescriptive** —
do not use them to generate PHI_BIAS for new formats.

## 3. GF256 — three distinct things (disambiguation)

| Kind | Locus | What | Detail |
|---|---|---|---|
| GF(2⁸) field | `trinity/specs/crypto/gf256.tri` | Galois field 𝔽₂₅₆ (crypto / erasure) | primitive poly `0x11D` (285), **not** AES `0x11B` |
| GF256 float (ratio) | `tt-trinity-gamma/specs/fpga/gf256.t27` | 256-bit φ-ratio float | `[S1 E97 M158]`, E/M 0.614, φ-dist 0.004 — **bias constant OPEN** (`0x7F:0xFFFF…` ≈ 2⁷¹ ≠ 2⁹⁶−1) |
| GF256 (range) | `zig-golden-float` whitepaper | proposed binary256-range φ float | candidate only — NOT CLAIMED |

## 4. Ternary members (TF3 ≠ GFTernary) and candidates

Two **distinct** objects — do not conflate (the spec is authoritative):

- **TF3** — a real **8-bit** format, layout `[S1 E3 M4]` = 1:3:4 (the same
  geometry as GF8), BIAS 3, used to encode ternary neural-network *weights*. It
  is a storage container, **not** a 2-bit format. Spec:
  [`tf3.t27`](../specs/numeric/tf3.t27) (line 3: "8-bit representation").
- **GFTernary** — the **2-bit** member, values in {−φ, 0, +φ} = φ·{−1, 0, +1}:
  a ternary-weight quantizer with the scale fixed at φ (cf. TWN / BitNet b1.58,
  whose scale α is learned per layer). φ-distance **0.000** by construction (its
  nonzero magnitude is exactly the anchor φ); the {−1, 0, +1} substrate of the
  Three Crowns. Spec: [`gfternary.t27`](../specs/numeric/gfternary.t27);
  Rust / `tri gen` codegen ROADMAP (same tier as GF64).
- **Candidates (no normative spec):** GF6 (φ-gap fill 4↔8), GF128
  (binary128-range φ). NOT CLAIMED.

## 5. Split-revision history (cite the split when quoting φ-distance)

- GF32: 8:23 → 13:18 → **12:19** (current canonical).
- GF64: 21:42 → **24:39** (current canonical).
The whitepaper's latest family table matches the canonical splits above.

## 6. Implementation status (claim discipline)

- **GF16 — VERIFIED.** Production Rust (`trios-trainer-igla`) + C codegen
  ([`../gen/c/numeric/gf16.c`](../gen/c/numeric/gf16.c)); benchmarked
  (97.67% MNIST MLP, 0.00% accuracy gap vs f32).
- **GF32 — claimed; three historical layouts** (12:19 canonical).
- **GF4/8/12/20/24 — ROADMAP** (spec / extract-only).
- **GF64 — ROADMAP** (spec present: [`gf64.t27`](../specs/numeric/gf64.t27), #916; C codegen still missing).
- **GF256 float — ROADMAP** (bias OPEN). **GF236 — does not exist (resolved).** `236` is the *mantissa width* of
IEEE **binary256** (1 + 19 + 236 = 256 bits), not a GoldenFloat format. The
name "GF236" conflated that mantissa count with a format label. The canonical
256-bit GoldenFloat is **GF256** (see §3); there is no GF236.

## 7. Measured comparison (IGLA RACE v2 format sweep)

Validation bits-per-byte (lower = better) for the same byte-level LM trained
under each numeric format (`trios-trainer-igla`; 30-log frozen snapshot,
2026-05-25; 2–3 seeds; `fp8_e4m3` died pre-eval → NO_EVAL):

| Format | Mean val_bpb | Reading |
|---|---|---|
| f32      | 2.5414 | full-precision baseline |
| fp16     | 2.5501 | best 16-bit in this sweep |
| **gf16** | **2.5725** | +0.031 bpb vs f32; **beats bf16** |
| bf16     | 2.6135 | worst of the 16-bit group |
| posit8   | 2.9322 | best 8-bit in this sweep |
| **gf8**  | **2.9322** | **bit-identical to posit8** |
| int8     | 3.4189 | linear 8-bit, well behind |
| mxfp8    | 3.4677 | ≈ int8 |
| fp8_e5m2 | 3.8528 | range-biased FP8, degrading |
| nf4      | 3.9985 | 4-bit, at the edge |
| int4     | 7.0184 | does not learn |

**Honest reading.** GF16 is parity-class with fp16 and a measured win over bf16
— *not* the single best 16-bit format (fp16 edges it). GF8 ties posit8 exactly.
**No GF-vs-E4M3 head-to-head** exists (E4M3 NO_EVAL). NF4/E5M2 underperform here
because this is from-scratch *training*, not post-training weight-quant. A
**bounded** empirical result, not a universal optimum.

## 8. Uniqueness (defensible claim)

GoldenFloat is the only published float family whose E:M split across the
4-to-256-bit ladder is generated by one closed rule (E/M → 1/φ from
φ²+φ⁻²=3), with Lucas-closure-exact accumulators (φ²ⁿ+φ⁻²ⁿ ∈ ℤ). This is a
methodology claim, not a performance claim: optimality is OPEN, and
throughput/accuracy are measured only for GF16.

---

*Maintainers: keep this file and `specs/numeric/*.t27` in lockstep. CI
invariant: each `gfN.t27` defines `PHI_BIAS`; codegen matches spec.*
