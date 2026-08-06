# Wave Loop 290 → Wave Loop 291 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W291

---

## Current State (Post-W290)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥32** (FIRST TIME) — 15 specs @ 32+, backend 34, systolic_array 34 |
| **CODER** | **ALL ≥21** (FIRST TIME) — 10 specs @ 21+, bench_proxy/dataset/arch 21 |
| **Pool B** | systolic_ternary @ 47 |
| **Integration** | ternary_inference @ 29 |
| **Lean 4** | 20 ternary theorems / 56 total |
| **Zero-entrant streak** | 55 waves (54th consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥33 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 32→33 AND push 3-4 CODER specs from 21→22.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 32→33
- backend already at 34, systolic_array at 34

### CODER Depth (3-4 specs × +1 invariant = +3-4 invariants)
- Target: bench_proxy 21→22, dataset 21→22, arch 21→22
- Maintain ALL ≥21

### Pool B (1 spec)
- systolic_ternary 47→48 (+1 invariant)

### Integration (1 spec)
- ternary_inference 29→30 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceIdentityGeneric` — identity weights preserve any input (generic)

**Total:** +38 tests, +21 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥33.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TerEffic / Sparkle.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 5 specs 32→33 (+5 invariants)

### Pool B Depth
- systolic_ternary 47→48 (+1 invariant)

### Integration
- ternary_inference 29→30 (+1 invariant)

### Lean 4
- `TernaryLUT.lean` with 2 theorems:
  - `lutTernaryMulEquivDirect` — LUT-based equals direct
  - `lutTernaryMulZeroWeightNop` — zero weight is NOP in LUT

**Total:** +18 tests, +12 invariants, +2 theorems, +1 new spec.
**Milestone:** First LUT-based ternary spec in t27; first hardware-algorithm equivalence for LUT.

---

## Variant C: Lean 4 Proof-Assistant Expansion

**Goal:** Add 3 new Lean 4 theorems in response to Sparkle HDL's 162+ theorem count.

### Lean 4 (+3 theorems)
- `ternaryInferenceIdentityGeneric` — identity weights preserve any input
- `ternaryGemmAssociativity` — GEMM associativity lemma
- `ternaryMacDistributivity` — MAC distributivity over addition

### Pool A (5 specs 32→33)
- +5 invariants

### CODER (3 specs 21→22)
- +3 invariants

**Total:** +22 tests, +11 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~59).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 33 | 5 specs → 33 | 5 specs → 33 |
| CODER target | 3-4 specs → 22 | maintain | 3 specs → 22 |
| Pool B target | 47→48 | 47→48 | maintain |
| Integration target | 29→30 | 29→30 | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +38 | +18 | +22 |
| Total invariants | +21 | +12 | +11 |
| Historic milestone | Pool A ≥33 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TerEffic/Sparkle | Sparkle HDL |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥33 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥33 is the natural next step after achieving ≥32
2. CODER has 3 specs at 21 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 56) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W292 once Pool A reaches ≥33

---

## GitHub Issues

Open issues (7 total):
- **#1219** — Epic roadmap: 12 workstreams (R-TT completion → Trinity provenance)
- **#1040** — P7 Low-bit / ternary track (parallel, optional) — `phi-loop`
- **#1041** — P8 Integration into t27 and publication — `phi-loop`
- **#1215** — Promote gf10 and gf256 to bitexact_selfconsistent (WP-34)
- **#1039** — P6 Scale-up to deployable 0.5B-1.5B (budget-gated)
- **#1038** — P5 Multi-language evaluation harness
- **#1037** — P4 Pilot pretraining at 50-200M

**Recommendation:** Prioritize #1215 (conformance) and #1040 (ternary track) for W291-W293.
