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

**Normative closed-form rule (v1.2):**

```
e = round((N - 1) / φ²)        // exponent width
m = N - 1 - e                  // mantissa width
bias = 2^(e - 1) - 1           // for e >= 1
exp_max = 2^e - 1
```

Applicability: `N ≥ 4` (binary ladder). `N = 2` is reserved for **GA-T**
(2-bit `{-φ, 0, +φ}` code, no E/M split). The rule is the single closed law
across the entire 4-to-1024-bit ladder; it is what `conformance/FORMAT-SPEC-001.json`
v1.2 promotes to normative.

**Frozen-silicon anchor:** **GF16 = 1+6+9, bias 31** is the layout literally
used by the fabricated GF16 multiplier in
[`tt-trinity-gamma/src/gf16_v2_mul.v`](../../tt-trinity-gamma/src/gf16_v2_mul.v)
(TTSKY26b TT4913 Gamma). Quoting the RTL header verbatim:

```
// GoldenFloat16 Multiplication Unit -- [S(1) | E(6) | M(9)], bias 2^(E-1)-1.
//   wire [5:0] exp_a  = a[14:9];
//   wire [8:0] mant_a = a[8:0];
//   localparam [5:0] EXP_MAX = 63;
//   localparam signed [7:0] BIAS_S = 8'sd31;
```

Any artefact disagreeing with this layout (notably `tt-trinity-corona/tools/gen_rom.py`
CATALOG cluster 3, which records `GF16 = 1+5+10` and 5 other rungs off-rule) is a
bug against this SSOT — see
[`gHashTag/claim-audit-lab` CASE-09](https://github.com/gHashTag/claim-audit-lab)
for the public self-audit.

---


## Naming — GF-T and GA-T are different objects (W722)

Two φ-anchored ladders exist in this ecosystem, on the same anchor
`φ² + φ⁻² = 3`, and they were nearly identically named. **They are settled as:**

| | **GF-T** | **GA-T** |
|---|---|---|
| stands for | **G**olden**F**loat, **T**ernary | **G**olden **A**lphabet, **T**ernary |
| what it is | a floating-point **format** with a balanced-ternary exponent | a weight **alphabet**, a finite level set, no exponent |
| lives in | `gHashTag/tri-net` | this repository |
| indexed by | **width in bits**: GF-T4 / 8 / 16 / 32 … 1024 | **rung** = highest power of φ: GA-T0 … GA-T4 |
| definition | `sign · significand · 2^e`, `e` balanced ternary | `GA-T_n = {0} ∪ {±φ^k : 0 ≤ k ≤ n}`, `2n+3` levels |
| on silicon | GF-T8/16/32 multiply, bit-exact (AX7203) | GA-T0…GA-T4 placed and loaded (XC7A200T) |

**Read the index and you know which object it is:** GF-T indices are bit widths,
GA-T indices are small rung numbers. `GF-T16` stores a value; `GA-T2` is what a
weight may be.

**`{−φ, 0, +φ}` is not a rung of GA-T.** It is `φ · GA-T0` — the alphabet's own
spec has said so since it was written (`specs/numeric/gfternary.t27`, line 4:
*"the phi-scaled limit of the ternary-weight family: GFTernary = phi \* {-1,0,+1}"*).

**What was NOT renamed, and why.** The spec module `triformat-gfternary`, its
constants `GFT_ZERO/POS/NEG`, and the path `specs/numeric/gfternary.t27` keep
their names: `.trinity/seals/numeric_triformat-gfternary.json` seals the
generated C, Rust, Verilog and Zig by hash, and renaming any of them changes
those artefacts. That rename is a separate wave with a re-seal.


## 1. Verified family constants

These four columns are mathematically exact and independently re-derived.

| Format | S+E+M | Bits | BIAS = 2^(E−1)−1 | EXP_MAX = 2^E−1 | E/M | φ-distance \|E/M−1/φ\| | Claim | Spec |
|---|---|---|---|---|---|---|---|---|
| GA-T | 2-bit code | 2 | — | — | — | 0.000 | Conj | [`gfternary.t27`](../specs/numeric/gfternary.t27) |
| GF4   | 1+1+2   | 4    | 0                  | 1                  | 0.500 | 0.118 | Verified | [`gf4.t27`](../specs/numeric/gf4.t27) |
| GF6   | 1+2+3   | 6    | 1                  | 3                  | 0.667 | 0.049 | Conj | [`gf6.t27`](../specs/numeric/gf6.t27) ‡ |
| GF8   | 1+3+4   | 8    | 3                  | 7                  | 0.750 | 0.132 | Verified | [`gf8.t27`](../specs/numeric/gf8.t27) |
| GF10  | 1+3+6   | 10   | 3                  | 7                  | 0.500 | 0.118 | Conj | [`gf10.t27`](../specs/numeric/gf10.t27) ‡ |
| GF12  | 1+4+7   | 12   | 7                  | 15                 | 0.571 | 0.047 | Verified | [`gf12.t27`](../specs/numeric/gf12.t27) |
| GF14  | 1+5+8   | 14   | 15                 | 31                 | 0.625 | 0.007 | Conj | [`gf14.t27`](../specs/numeric/gf14.t27) ‡ |
| **GF16 (primary, frozen tape-out design)** | **1+6+9** | **16** | **31** | **63** | **0.667** | **0.049** | **Verified** | [`gf16.t27`](../specs/numeric/gf16.t27) |
| GF20  | 1+7+12  | 20   | 63                 | 127                | 0.583 | 0.035 | Verified | [`gf20.t27`](../specs/numeric/gf20.t27) |
| GF24  | 1+9+14  | 24   | 255                | 511                | 0.643 | 0.025 | Verified | [`gf24.t27`](../specs/numeric/gf24.t27) |
| GF32  | 1+12+19 | 32   | 2047               | 4095               | 0.632 | 0.014 | Verified | [`gf32.t27`](../specs/numeric/gf32.t27) |
| GF48  | 1+18+29 | 48   | 131071             | 262143             | 0.621 | 0.003 | Conj | [`gf48.t27`](../specs/numeric/gf48.t27) ‡ |
| GF64  | 1+24+39 | 64   | 8388607            | 16777215           | 0.615 | 0.003 | Verified | [`gf64.t27`](../specs/numeric/gf64.t27) |
| GF96  | 1+36+59 | 96   | 2³⁵−1 = 34359738367 | 2³⁶−1             | 0.610 | 0.008 | Conj | [`gf96.t27`](../specs/numeric/gf96.t27) ‡ |
| GF128 | 1+49+78 | 128  | 2⁴⁸−1              | 2⁴⁹−1              | 0.628 | 0.010 | Conj | [`gf128.t27`](../specs/numeric/gf128.t27) ‡ |
| GF256 | 1+97+158| 256  | 2⁹⁶−1 †             | 2⁹⁷−1 †            | 0.614 | 0.004 | Conj | [`gf256.t27`](../specs/numeric/gf256.t27) ‡ |
| GF512 | 1+195+316 | 512 | 2¹⁹⁴−1             | 2¹⁹⁵−1             | 0.617 | 0.0009 | Conj | [`gf512.t27`](../specs/numeric/gf512.t27) ‡ |
| GF1024| 1+391+632 | 1024 | 2³⁹⁰−1            | 2³⁹¹−1             | 0.619 | 0.0006 | Conj | [`gf1024.t27`](../specs/numeric/gf1024.t27) ‡ |
| TF3   | 1+3+4   | 8    | 3                  | 7                  | 0.750 | 0.132 | Conj | [`tf3.t27`](../specs/numeric/tf3.t27) |

‡ **New in v1.2** (added 2026-06-07). Spec only — no validated RTL.
GF96/GF128/GF256 spec-level; GF512/GF1024 extrapolated (no RTL planned).
All new rungs carry claim status `Conj` with falsification path = the closed-form rule
`e = round((N−1)/φ²)` must reproduce the values above; any RTL or external
implementation deviating from these splits falsifies the rung.

† **GF256 caveat.** Its *stored* exponent-bias constant in gamma RTL (≈ 2⁷¹) is
unreconciled with `2^(E−1)−1 = 2⁹⁶−1`, so the bias is **OPEN** (see §3); only its
geometry (97/158 split, φ-distance 0.004) is verified. The canonical spec now
lives at [`gf256.t27`](../specs/numeric/gf256.t27) in this repo as of v1.2; the
RTL stub remains at `tt-trinity-gamma/specs/fpga/gf256.t27`.

The last two rows are **not** float-ladder rungs: **TF3** is an 8-bit
ternary-weight container reusing GF8's 1:3:4 geometry (so it shares GF8's
constants); **GA-T** is the 2-bit {−φ, 0, +φ} limit — no exponent/mantissa
split (columns N/A), φ-distance 0 by construction.

Family base: [`goldenfloat_family.t27`](../specs/numeric/goldenfloat_family.t27).
Closest *split* to 1/φ in the ≤256-bit range is **GF64 (0.0026)**, then
**GF48 (0.0027)**, then GF256 (0.004). Among ≤16-bit: GF14 (0.007) is best,
then GF20 (0.035), then GF12 (0.047). GA-T's 0.000 is by construction.
The **extrapolated extension** GF512 (0.0009) and GF1024 (0.0006) push closer
to 1/φ as N → ∞ — by design, since `round((N−1)/φ²) / (N−1−round(…)) → 1/φ`
as N grows. GF64 carries `PHI_BIAS = 8388608` — the one format where it
coincides with `EXP_MAX − BIAS = 2²³`.

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
| **GF6, GF10, GF14, GF48, GF96, GF128, GF256, GF512, GF1024** | **OPEN** | not yet defined; v1.2 explicitly refuses to invent values via Fibonacci/Lucas/square coincidence |

Fibonacci/Lucas/square coincidences are **descriptive, not prescriptive** —
do not use them to generate PHI_BIAS for new formats.

## 3. GF256 — three distinct things (disambiguation)

| Kind | Locus | What | Detail |
|---|---|---|---|
| GF(2⁸) field | `trinity/specs/crypto/gf256.tri` | Galois field 𝔽₂₅₆ (crypto / erasure) | primitive poly `0x11D` (285), **not** AES `0x11B` |
| GF256 float (ratio) | `tt-trinity-gamma/specs/fpga/gf256.t27` | 256-bit φ-ratio float | `[S1 E97 M158]`, E/M 0.614, φ-dist 0.004 — **bias constant OPEN** (`0x7F:0xFFFF…` ≈ 2⁷¹ ≠ 2⁹⁶−1) |
| GF256 (range) | `zig-golden-float` whitepaper | proposed binary256-range φ float | candidate only — NOT CLAIMED |

## 4. Ternary members (TF3 ≠ GA-T) and candidates

Two **distinct** objects — do not conflate (the spec is authoritative):

- **TF3** — a real **8-bit** format, layout `[S1 E3 M4]` = 1:3:4 (the same
  geometry as GF8), BIAS 3, used to encode ternary neural-network *weights*. It
  is a storage container, **not** a 2-bit format. Spec:
  [`tf3.t27`](../specs/numeric/tf3.t27) (line 3: "8-bit representation").
- **GA-T** — the **2-bit** member, values in {−φ, 0, +φ} = φ·{−1, 0, +1}:
  a ternary-weight quantizer with the scale fixed at φ (cf. TWN / BitNet b1.58,
  whose scale α is learned per layer). φ-distance **0.000** by construction (its
  nonzero magnitude is exactly the anchor φ); the {−1, 0, +1} substrate of the
  Three Crowns. Spec: [`gfternary.t27`](../specs/numeric/gfternary.t27);
  Rust / `tri gen` codegen ROADMAP (same tier as GF64).
- **Candidates (no normative spec):** GF6 (φ-gap fill 4↔8), GF128
  (binary128-range φ). NOT CLAIMED.


### 4a. TF — the ternary-EXPONENT ladder (distinct from both of the above)

A **third** ternary object, and the one most often confused with GFTernary. TF
is not a ternary alphabet: it is an ordinary binary-radix float whose **exponent
field is a balanced-ternary integer** of `E_t` trits, with a constant `M`-bit
mantissa and fixed fields.

    value = (-1)^s * (1 + M/2^m) * 2^e,   e = sum_i t_i * 3^i

The scale is a power of **two**; only the exponent's *encoding* is ternary. A
format that scales by `3^e` — Ternary27 is the published example — is a different
design, and measurably a worse one at equal width: with the significand
quantised over `[1,r)` the mean relative error is `kappa(r) * 2^-M` with
`kappa(r) = (r-1)^2 / (r ln r)`, so radix 3 costs 0.75 positions of precision and
buys 0.42 of range, a net loss of 0.33 positions per number.

**Width rule:** `1 + E_t + M = N`, counting one position per trit. Eight of the
nine rungs satisfy it exactly. TNF16 does not — it inherits GF16's phi-optimal
`M = 9` and replaces GF16's six-bit exponent with four trits, leaving two
positions unallocated. Spending them as `M = 11` divides the error by 4.00 at
unchanged range for 19% more LUTs and no loss of frequency (measured, XC7A200T).
Not changed, because it invalidates the conformance vectors and the published
silicon numbers; recorded here and in the rung's `open_issue` field so the
decision stays explicit.

| Format | S+E+M | Bits | \|e\| max | Decades | E_t bits-equiv | φ-distance | Fits rule | Post-route XC7A200T | Spec |
|---|---|---|---|---|---|---|---|---|---|
| TNF4 | 1+2t+1 | 4 | 4 | 2 | 3.17 | 2.552 | **yes** | 12 LUT / 161.1 MHz | [`tnf4.t27`](../specs/numeric/tnf4.t27) |
| TNF8 | 1+3t+4 | 8 | 13 | 8 | 4.75 | 0.571 | **yes** | 50 / 153.2 | [`tnf8.t27`](../specs/numeric/tnf8.t27) |
| TNF16 | 1+4t+9 | 16 | 40 | 24 | 6.34 | 0.086 | **no (2 unallocated)** | 212 / 131.7 | [`tnf16.t27`](../specs/numeric/tnf16.t27) |
| TNF32 | 1+6t+25 | 32 | 364 | 219 | 9.51 | 0.238 | **yes** | 1477 / 83.3 | [`tnf32.t27`](../specs/numeric/tnf32.t27) |
| TNF64 | 1+7t+56 | 64 | 1093 | 658 | 11.09 | 0.420 | **yes** | 7479 / 48.2 | [`tnf64.t27`](../specs/numeric/tnf64.t27) |
| TNF128 | 1+8t+119 | 128 | 3280 | 1975 | 12.68 | 0.511 | **yes** | — | [`tnf128.t27`](../specs/numeric/tnf128.t27) |
| TNF256 | 1+9t+246 | 256 | 9841 | 5925 | 14.26 | 0.560 | **yes** | — | [`tnf256.t27`](../specs/numeric/tnf256.t27) |
| TNF512 | 1+10t+501 | 512 | 29524 | 17775 | 15.85 | 0.586 | **yes** | — | [`tnf512.t27`](../specs/numeric/tnf512.t27) |
| TNF1024 | 1+11t+1012 | 1024 | 88573 | 53326 | 17.43 | 0.601 | **yes** | — | [`tnf1024.t27`](../specs/numeric/tnf1024.t27) |

**Reading the φ-distance column.** It is the catalog's own `|E/M - 1/φ|` computed
on the exponent's bit-equivalent `E_t*log2(3)`, and it **rises toward 1/φ as N
grows**. That is structural rather than a defect. GF sizes its exponent by
`e = round((N-1)/φ²)`, which puts `E/M` at `1/φ` by construction; TF sizes its
exponent for **range** and takes `M = N-1-E_t`. The two ladders optimise different
axes, and TF should not be read as a worse GF.

**What the fixed field buys, measured.** TF has no regime codec at all. Under one
harness on XC7A200T, net of a 24-LUT harness, a unary regime codec (posit class)
costs 438 LUTs round trip and a length-prefixed one (takum class) 40, against 0
here. Converting through the measured area law `A(M) = 141 + 2.4455*M^2`, whose
derivative `λ = 4.891*M` is the marginal LUT cost of a mantissa bit, the unary
regime is **9.95 mantissa bits of silicon** at the 16-bit class — more than
TNF16's entire mantissa — and 1.00 at `M = 90`. The length-prefixed regime is
0.91 bits and 0.09. The area argument is therefore decisive against posit and weak
against takum, and is stated that way rather than averaged.

**What it does not buy.** By Kraft's inequality no prefix code on the integers has
`l(e) < log2|e|` for all but finitely many `e`, so no format of unbounded range
tapers more gently than one bit per doubling: takum's regime is asymptotically
optimal and **nothing, TF included, beats it beyond TF's own range**. The
measured 2.83x and 5.46x against takum16 at mid and far magnitudes hold inside
that range and must never be quoted without it.

Conformance packs: `conformance/vectors/gft{4,8,16,32,64,128,256,512,1024}_conformance_v0.json`.
Probes are powers of two and `1.5*2^e` inside each rung's own range, so every
`abs_error` is exactly zero by construction.

## 5. Split-revision history (cite the split when quoting φ-distance)

- GF32: 8:23 → 13:18 → **12:19** (current canonical).
- GF64: 21:42 → **24:39** (current canonical).
The whitepaper's latest family table matches the canonical splits above.

## 6. Implementation status (claim discipline)

- **GF16 — Verified (spec, codegen, RTL, FPGA testbench).** Production Rust (`trios-trainer-igla`) +
  C codegen ([`../gen/c/numeric/gf16.c`](../gen/c/numeric/gf16.c)) + RTL
  ([`tt-trinity-gamma/src/gf16_v2_mul.v`](../../tt-trinity-gamma/src/gf16_v2_mul.v)
  and `gf16_v2_add.v`); 35/35 FPGA testbench on Artix-7 (no operating frequency claimed — the 323 MHz figure is withdrawn, see docs/nona-03-manifest/RESEARCH_CLAIMS.md); benchmarked
  against the fp32 reference on synthetic round-trip. **An MNIST figure of "97.67 %,
  0.00 % accuracy gap vs f32" stood here and is withdrawn**: nothing in the tree
  produces it, and the only MNIST run present
  (`conformance/gf_family_bench.json`) has every format at accuracy 0.1187 with
  loss 2.3631 — chance is 0.1000 and ln(10) is 2.3026, so the model is untrained.
  Every format scoring identically to four decimals is itself the tell that the
  scenario measures nothing about the format; one format scores 0.098, below
  chance. Submitted to the TTSKY26b TT4913 Gamma shuttle; **no die has been
  received**, and `STATUS.md` reserves SILICON for a physical die with written
  bring-up and forbids the claim in this repository at all.
- **GF4/8/12/20/24/32 — Verified.** Spec + Verilog RTL in `tt-trinity-gamma/src/`.
- **GF64 — Verified.** Spec ([`gf64.t27`](../specs/numeric/gf64.t27), #916) +
  Verilog RTL in gamma; C codegen pending.
- **GF6/10/14 (NEW v1.2) — Conj.** Spec only. Falsification path: the closed-form
  rule above must reproduce the splits 1+2+3 / 1+3+6 / 1+5+8.
- **GF48/96/128 (NEW v1.2) — Conj.** Spec only. Useful as cross-format anchors;
  no RTL planned in this repo (gamma may add).
- **GF256 (NEW v1.2 here; existed at gamma) — Conj.** Spec promoted into this
  repo as `gf256.t27`; geometry 97/158 verified; bias constant remains OPEN.
- **GF512 / GF1024 (NEW v1.2 — extrapolated) — Conj.** Spec only. No RTL.
  These exist to demonstrate that the closed-form rule has no upper bound and
  the φ-distance continues to decrease (0.0009 and 0.0006 respectively). Any
  matched-substrate benchmark must be normalised to in-range regimes
  (`φ^512` / `φ^1024` overflows the dynamic range of `binary{N}` long before
  it overflows the GF rung itself).

**GF236 — does not exist (resolved).** `236` is the *mantissa width* of
IEEE **binary256** (1 + 19 + 236 = 256 bits), not a GoldenFloat format. The
name "GF236" conflated that mantissa count with a format label. The canonical
256-bit GoldenFloat is **GF256** (see §3); there is no GF236.

**Corona ROM CATALOG bug (cross-repo).** As of 2026-06-07, `tt-trinity-corona/tools/gen_rom.py`
cluster 3 (`CL_GOLDENFLOAT`) records six rungs with splits that violate this
SSOT's closed-form rule: GF16=1+5+10 (should be 1+6+9), GF24=1+8+15 (should be
1+9+14), GF32=1+11+20 (should be 1+12+19), GF48=1+16+31 (should be 1+18+29),
GF64=1+22+41 (should be 1+24+39), GF96=1+33+62 (should be 1+36+59),
GF128=1+44+83 (should be 1+49+78), GF256=1+88+167 (should be 1+97+158). Corona
is a registry chip; the on-die arithmetic for GF formats lives in Gamma
(per `tt-trinity-corona/docs/goldenfloat_ladder_crossreference.md`), and
Gamma's frozen tape-out design implements 1+6+9 for GF16. The CATALOG bug is tracked
in [`gHashTag/claim-audit-lab` CASE-09](https://github.com/gHashTag/claim-audit-lab).

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

## 8. Uniqueness (defensible claim, Conj)

We are not aware of another published float family whose E:M split across a
4-to-1024-bit ladder is generated by **one closed rule**
(`e = round((N−1)/φ²)`, `m = N−1−e`, anchored on φ²+φ⁻²=3), paired with
**Lucas-closure-exact accumulators** (φ²ⁿ+φ⁻²ⁿ ∈ ℤ). This is a
**methodology claim** (`Conj`), not a performance claim: optimality is OPEN,
and throughput/accuracy are measured only for GF16. Posit (Gustafson 2017),
OCP-MX (Rouhani 2023), LNS (Arnold/Parhami), and takum (Hunhold 2024,
[arXiv:2404.18603](https://arxiv.org/abs/2404.18603)) are **allies and
falsification targets**, not competitors crushed. The honest shield is
**multiplier-free at the anchor** (φ² = φ + 1 collapses gain to unity), not
uniqueness of φ as a base — Daubechies et al. (Golden Ratio Encoder,
[IEEE TIT 56(10) 2010](https://arxiv.org/abs/0809.1257)) establish that
other β values also yield robust encoders.

---

*Maintainers: keep this file and `specs/numeric/*.t27` in lockstep. CI
invariant: each `gfN.t27` defines `PHI_BIAS`; codegen matches spec.*
