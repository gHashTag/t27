# P3109 Cross-Walk (4-Parameter Re-Anchored)

**Status:** Draft. Implementor cross-reference only. This document does NOT
assert P3109 compliance. Where our decode semantics disagree with the P3109
formal model, we defer to FLoPS (arXiv:2602.15965, Lean formalization) and to
the IEEE P3109 working group's own definitions.

**Re-anchored:** 2026-06-14. Supersedes any cross-walk labeled "P3109 v3.2.0".
The public P3109 anchor is the WG interim report (v0.9.1, 2024-10-29) **plus**
the working group's own 4-parameter redefinition in
[arXiv:2606.04028](https://arxiv.org/abs/2606.04028) ("Novel Aspects of IEEE SA
P3109 Arithmetic Formats for Machine Learning", Fitzgibbon, Wintersteiger,
Sarnoff, 1 Jun 2026). We map to THAT model, in the WG's own vocabulary.

---

## The P3109 4-parameter model (their language)

A P3109 format is defined by four parameters (per arXiv:2606.04028 and FLoPS):

| Param | Symbol | Meaning |
|---|---|---|
| Bitwidth | K | total stored bits |
| Precision | P | significand bits **including the implicit leading bit** |
| Signedness | Sigma | Signed or Unsigned |
| Domain | Delta | Finite or Extended (Extended = +/-Inf and NaN present) |

8-bit instances are named `binary8pP` for P in 1..7. For a `binaryK pP` format:
exponent field width `w = K - P` (signed), significand stored bits `= P - 1`.

Two P3109 design rules relevant to the mapping:
- Single NaN (no payload), single zero (no -0).
- Domain controls special values: `Extended` carries +/-Inf and one NaN;
  `Finite` carries neither. (Saturation handling is an operation/rounding-mode
  property, not a format parameter.)

---

## Our representative rows -> P3109 (K, P, Sigma, Delta)

Precision P = stored mantissa bits (`m`) + 1 implicit bit. Sigma = Signed for
every row below (all our FP rows carry a sign bit, `s=1`). Delta is determined
by whether the format defines Inf/NaN.

| Our id | bits K | s | e | m | P = m+1 | P3109 tuple | P3109 name | Match quality |
|---|---|---|---|---|---|---|---|---|
| `fp8_e4m3` | 8 | 1 | 4 | 3 | 4 | (K=8, P=4, Signed, Finite\*) | binary8p4 | element match; domain/overflow gap (see kappa) |
| `fp8_e5m2` | 8 | 1 | 5 | 2 | 3 | (K=8, P=3, Signed, Extended) | binary8p3 | element + domain match (true Inf/NaN) |
| `mxfp4` (element) | 4 | 1 | 2 | 1 | 2 | (K=4, P=2, Signed, Finite) | binary4p2 | element match; block scale is orthogonal to P3109 |
| `binary16` (fp16) | 16 | 1 | 5 | 10 | 11 | (K=16, P=11, Signed, Extended) | binary16p11 | IEEE 754 half; P3109 conversion target |
| `bf16` | 16 | 1 | 8 | 7 | 8 | (K=16, P=8, Signed, Extended) | binary16p8 | element match; P3109 conversion target |
| `gf16` (this work) | 16 | 1 | 6 | 9 | 10 | (K=16, P=10, Signed, Extended) | binary16p10 | structural match; GF bias differs (PHI_BIAS) |
| `e8m0` (block scale) | 8 | 0 | 8 | 0 | -- | scale-only; NOT a P3109 datum format | -- | Orthogonal (block scale layer) |

\* **fp8_e4m3 domain note (the kappa hook).** Our `fp8_e4m3` follows the
tt-metal/AMD convention: no Inf, single NaN at 0x7F/0xFF, encode-overflow
**saturates** to max-finite 448.0 (0x7E). JAX/TPU `ml_dtypes` uses
overflow-to-NaN (0x7F). OCP MX v1.0 permits both. In P3109 4-parameter terms
this is a `Finite`-vs-near-`Extended` distinction combined with a saturation
choice. It is exactly the divergence P3109's **kappa-approximation** measure is
designed to quantify (see `KAPPA_FP8_E4M3.md`).

### Orthogonal-by-design (no P3109 tuple)

- **E8M0 block scale**: a shared exponent scale for a block of elements, not a
  scalar datum format. P3109 defines block operations over a shared scale
  uniformly, but the scale encoding itself is outside the binaryKpP datum
  family. Listed as Orthogonal.
- **GF16 bias**: GF16's exponent bias is phi-derived (PHI_BIAS), not the P3109
  `2^(w-1)` / `2^(w-1)-1` convention. The bit *layout* maps cleanly to
  binary16p10; the *value set* differs by bias. Marked structural match, value
  set divergence -- defer to FLoPS-style value-set comparison, do not assert
  equality.

## Note on the P3109 bias convention (live, contested)

The P3109 WG voted the exponent bias to `2^(w-1)-1` on 2025-04-28, then reversed
to `2^(w-1)` on 2025-07-07 (Overton, "Update Regarding the Exponent Bias",
2025-07-20). Any bias-sensitive row in this cross-walk (notably the FP8 rows)
should be read against whichever bias the consuming P3109 draft pins. We carry
both interpretations in the conformance packs and do not hard-code one as
"the" P3109 bias.

## What we defer on (honesty wall)

- **Decode semantics / rounding / projection:** defer to FLoPS (Lean) and the
  WG. Our packs assert bit-exact encode/decode of the element layout, not the
  full operation semantics P3109 specifies (Fast2Sum, ExtractScalar, FMA,
  scaled add/mul, stochastic rounding). We are the conformance/registry layer,
  not the proof layer.
- **The 4-bit and sub-8-bit P3109 names** (binary4pP) are our reading of the
  parameterized family extended below 8 bits; the WG's public naming focuses on
  binary8pP. Treat binary4p2 as an implementor label, not a WG-blessed name.
