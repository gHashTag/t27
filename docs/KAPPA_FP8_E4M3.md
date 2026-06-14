# Quantifying the FP8 E4M3 Overflow Divergence with P3109 kappa

**Status:** Draft. Implementor reference. No compliance claim, no superiority
claim. This document expresses a *known, documented* cross-vendor divergence in
the vocabulary the IEEE P3109 working group introduced for exactly this purpose.

**Anchor:** kappa-approximation is defined in
[arXiv:2606.04028](https://arxiv.org/abs/2606.04028) (Fitzgibbon, Wintersteiger,
Sarnoff, "Novel Aspects of IEEE SA P3109 Arithmetic Formats for Machine
Learning", 1 Jun 2026): a scale-invariant accuracy measure, akin to units in the
last place but precisely defined for all operations, by which a vendor describes
an **approximate** implementation. An exact implementation must always be
provided; kappa describes how far an approximate one may deviate.

---

## The divergence (already documented, now measured)

For FP8 E4M3 (S1E4M3, bias 7, max finite 448.0 at 0x7E), the encode of an
overflowing input such as `1000.0` is **spec-permissible in two ways** under
OCP MX v1.0:

| Convention | Behavior on encode(1000.0) | Result code | Decoded value |
|---|---|---|---|
| tt-metal / AMD | saturate to max finite | 0x7E | 448.0 |
| JAX / TPU (`ml_dtypes`) | overflow to NaN | 0x7F | NaN |

Both are spec-compliant. The divergence silently changes model behavior when a
tensor is ported across vendors -- the exact "silent divergence" our conformance
packs exist to catch.

## Expressing it as a kappa-approximation

Treat the saturation path as an approximate implementation of the real value
`x = 1000.0`, and measure its deviation.

- Max finite (decode 0x7E) = **448.0** (bit-exact).
- ULP at the top binade (exponent e_max = 15 - 7 = 8, 3 mantissa bits):
  `2^(8-3) = 32.0`.
- Saturation absolute error vs the true real: `|448.0 - 1000.0| = 552.0`.
- In ULP at the top binade: `552.0 / 32.0 = 17.25 ULP`.
- **kappa (scale-invariant, relative):** `|approx - true| / |true|
  = 552.0 / 1000.0 = 0.552`.

For the JAX/TPU path, the implementation **declines to approximate**: it signals
NaN rather than returning a finite value. In kappa terms this is **undefined**
(no finite approximation is offered). This is the honest, precise distinction:

| Path | P3109 Domain reading | kappa(encode 1000.0) |
|---|---|---|
| tt-metal / AMD saturate-to-max | Finite (saturating) | **0.552** (finite, large) |
| JAX / TPU overflow-to-NaN | Extended (signals NaN) | **undefined** (refuses to approximate) |

## Why this is the right framing (and what we do NOT claim)

- We do **not** claim either vendor is correct. Both are OCP-permissible. The
  kappa value simply makes the *cost* of the saturation choice explicit and
  scale-invariant, in the WG's own measure.
- We do **not** claim to define or own kappa. It is the P3109 WG's construct
  (arXiv:2606.04028). We are applying it to a divergence we already ship as a
  conformance vector, so the divergence becomes a *number a vendor can act on*
  rather than a footnote.
- The bit-exact codes (0x7E saturate, 0x7F NaN) and the decoded values are the
  Verified part. The kappa value 0.552 is a derived, reproducible measure
  (`kappa_calc.py`), not an accuracy claim about any model.

## Hook into the conformance pack

The `fp8_e4m3` conformance pack already carries the 1000.0 row with both the
0x7E (ours) and the documented 0x7F (JAX/TPU) interpretation. This document adds
the kappa value as the standards-native way to report that row's cross-vendor
cost. A future pack field `kappa_overflow` can carry this per-vendor so the
divergence is machine-readable, not just prose.
