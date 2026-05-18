# GF16 vs bfloat16 NMSE Protocol

> **Status:** SPEC. Protocol-level only. This document standardises *how* a
> GF16-vs-bfloat16 NMSE comparison is run and reported in the TRI-NET line.
> It does **not** publish silicon numbers -- those belong with the chip repos
> when (and only when) silicon evidence is available.
>
> **Source of truth:** `specs/benchmarks/gf16_bfloat16_nmse.t27` is the
> machine-readable spec; this document is the human-readable mirror. If the
> two disagree, the `.t27` spec wins and the disagreement is an issue.
>
> **R5-HONEST:** No row in this document asserts a measured silicon result.
> Reference distributions and tolerance windows are stated as protocol
> parameters, not outcomes.

---

## 1. Scope and intent

The TRI-NET numeric kernel is **GoldenFloat GF16** (primary path; see
`FORMAT_REGISTRY.md`). The dominant industry alternative at the same width
for inference workloads is **bfloat16** (BF16). Multiple parties have asked
for a like-for-like NMSE (normalised mean-squared error) comparison. The
purpose of this document is to define **one protocol** for that comparison
so that:

1. results produced under it are reproducible from this repo;
2. results from chip repos (`tt-trinity-phi`, `tt-trinity-euler`,
   `tt-trinity-gamma`) can be compared with the same methodology;
3. nothing in the protocol presumes a TOPS race -- only numeric fidelity.

Out of scope: latency, throughput, energy. Those have their own (also
restrained) treatments in `BENCHMARKS.md`.

---

## 2. Quantities

Let `x` be a reference real value (drawn from a defined distribution,
section 4) and let `Q(x)` be the result of round-trip
`real -> format -> real` through a numeric format. Define

```
NMSE(F)
    = E[ (x - Q_F(x))^2 ] / E[ x^2 ]
```

where the expectation is taken over the protocol's reference distribution.
Two formats are compared by reporting `NMSE(GF16)` and `NMSE(BF16)` against
the same sampled `x` set and the same RNG seed.

**Important:** the ratio `NMSE(GF16) / NMSE(BF16)` is the headline number.
A ratio of 1.0 means equal numeric fidelity at the protocol's distribution;
< 1.0 means GF16 is closer to the reference for that distribution; > 1.0
means BF16 is closer. No claim is attached to either direction without a
specific distribution and seed.

---

## 3. Format definitions used by the protocol

### 3.1 GF16

GF16 is defined by `specs/numeric/gf16.t27` and recorded in the SSOT
`conformance/FORMAT-SPEC-001.json`. Bit layout (mirror of
`FORMAT_REGISTRY.md` section 1):

```
GF16 = [ S(1) | E(6) | M(9) ]
value = (-1)^S * 2^(E - 31) * (1 + M / 2^9)
```

The protocol uses the canonical round-to-nearest, ties-to-even rounding
rule defined in the spec.

### 3.2 bfloat16

BF16 is defined externally by the IEEE-754 binary32 layout with mantissa
truncated to 7 bits:

```
BF16 = [ S(1) | E(8) | M(7) ]
value = (-1)^S * 2^(E - 127) * (1 + M / 2^7)
```

The protocol uses round-to-nearest, ties-to-even. No subnormal handling
deviation is permitted; BF16 implementations that flush subnormals to zero
must declare that fact in the manifest (section 6).

### 3.3 Why the comparison is meaningful

GF16 and BF16 occupy the same memory footprint (16 bits). They differ in
how those 16 bits are split: GF16 gives 9 bits to the mantissa, BF16 gives
7. GF16's exponent field is 6 bits with bias 31; BF16's is 8 bits with
bias 127. The expected outcome is that **GF16 wins on near-1.0 dynamic
range, BF16 wins on very large / very small values**. The protocol must
not preempt this with a distribution chosen to favour either side.

---

## 4. Reference distributions

A run reports NMSE under **each** of these distributions independently.
A single number reported without naming a distribution is invalid under
this protocol.

| Tag       | Distribution                                  | Rationale                            |
|-----------|------------------------------------------------|--------------------------------------|
| `D_NORM`  | `x ~ N(0, 1)`                                 | Generic weight-like distribution     |
| `D_LOG`   | `log2|x| ~ U(-10, 10)`, sign uniform           | Geometric coverage of dynamic range  |
| `D_RELU`  | `x = max(0, N(0, 1))`                         | Post-activation weight distribution  |
| `D_PHI`   | `x ~ N(phi, 1/phi)`, where `phi=(1+sqrt 5)/2` | Identity-anchored sanity (L5)        |
| `D_DEEP`  | mixture: 0.7 `D_NORM` + 0.3 `D_LOG`           | Heuristic for transformer weights    |

Each run uses 10 million samples per distribution unless explicitly
overridden in the manifest.

---

## 5. Tolerance and identity check (L5)

Before any NMSE figure is reported, a run **must** witness:

```
|phi^2 - (phi + 1)|  <  1e-15        // f64 identity check
|phi^2 + 1/phi^2 - 3| < 1e-15        // canonical Trinity identity
```

Failing either witness aborts the run. This is L5 IDENTITY enforced at the
benchmark boundary.

---

## 6. Results manifest

A run produces one JSON file conforming to `schemas/nmse-protocol-v1.json`.
The schema requires, at minimum:

- protocol version (semver);
- toolchain seal hash (matches `bootstrap/stage0/FROZEN_HASH`);
- RNG family and seed;
- sample count per distribution;
- per-distribution `NMSE_GF16`, `NMSE_BF16`, and their ratio;
- BF16 subnormal policy (`ieee` or `ftz`);
- runner identity (host architecture, compiler version);
- timestamp (RFC3339).

A run that omits any required field is non-conforming and must not be
cited in TRI-NET documentation.

---

## 7. Test obligations (L4)

The companion spec `specs/benchmarks/gf16_bfloat16_nmse.t27` includes:

- a `test` block that runs the identity witness;
- an `invariant` block that asserts `NMSE >= 0` for each format;
- a `bench` block that defines the measurement procedure.

These are the L4 TESTABILITY requirements for this benchmark family.

---

## 8. Reporting policy

When a chip-repo or third-party result is cited:

- ratio reported only when both sides came from the same seed/distribution;
- protocol version stated;
- seal hash stated (or `unsealed` if not measured under a sealed toolchain
  -- in which case the result is informational, not certifying);
- no comparison against a commercial-NPU number is permitted in this
  protocol's outputs (see `COMPETITORS.md` for why).

---

## 9. Cross-links

- Numeric SSOT: `conformance/FORMAT-SPEC-001.json`, `FORMAT_REGISTRY.md`.
- Sibling repos that may emit conforming manifests:
  - `tt-trinity-phi` (phi-anchor, 1x1) -- identity-domain NMSE only.
  - `tt-trinity-euler` (8x2, safety) -- `D_NORM` and `D_RELU` envelopes.
  - `tt-trinity-gamma` (8x4, 32-PE mesh) -- `D_DEEP` is the headline.
- TRI-NET API doc: `docs/TRI_NET_API.md` -- how an external integrator
  reads NMSE manifests programmatically.
- Roadmap: `docs/SCIENTIFIC_IMPROVEMENT_PLAN.md` -- PUB-02 names "one
  sealed-toolchain NMSE manifest" as a 2026 target deliverable.

---

## 10. Non-claims (R5-HONEST)

- This document does **not** claim a measured silicon NMSE for any product.
- This document does **not** claim GF16 is universally better than BF16.
- This document does **not** claim a fixed `NMSE(GF16) / NMSE(BF16)` ratio.
- It defines **how** to measure such ratios so claims, when made, are
  reproducible.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
