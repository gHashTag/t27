# Wave Loop 291 → Wave Loop 292 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W292

---

## Current State (Post-W291)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥31** (FIRST TIME) — systolic_array 31, all others 31+ |
| **CODER** | **ALL ≥21** (FIRST TIME) — all 10 specs at 21+ |
| **Pool B** | systolic_ternary @ 46 |
| **Integration** | ternary_inference @ 30 |
| **Lean 4** | 21 ternary theorems / 57 total |
| **Zero-entrant streak** | 56 waves (55th consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥32 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 31→32 AND push 3–4 CODER specs from 21→22.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 31→32
- adder_tree/bram_weights/eda/formal/gemm/ternary_gemm/ternary_mac already at 32

### CODER Depth (3–4 specs × +1 invariant = +3-4 invariants)
- Target: arch 21→22, bench_proxy 21→22, dataset 21→22, benchmark 21→22
- Maintain ALL ≥21

### Pool B (1 spec)
- systolic_ternary 46→47 (+1 invariant)

### Integration (1 spec)
- ternary_inference 30→31 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceSignGeneric` — generic proof that plus/minus weights preserve/invert sign for any activation

**Total:** +36 tests, +21 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥32.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TerEffic / Sparkle.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 8 specs 31→32 (+8 invariants)

### Pool B Depth
- systolic_ternary 46→47 (+1 invariant)

### Lean 4
- `TernaryLUT.lean` with 2 theorems:
  - `lutTernaryMulEquivDirect` — LUT-based equals direct
  - `lutTernaryMulZeroWeightNop` — zero weight is NOP in LUT

**Total:** +20 tests, +15 invariants, +2 theorems, +1 new spec.
**Milestone:** First LUT-based ternary spec in t27; first hardware-algorithm equivalence for LUT.

---

## Variant C: Lean 4 Proof-Assistant Expansion

**Goal:** Add 3 new Lean 4 theorems in response to Sparkle HDL's 102+ theorem count.

### Lean 4 (+3 theorems)
- `ternaryInferenceSignGeneric` — generic sign preservation/inversion
- `ternaryInferenceAssociativity` — (A ⊗ B) ⊗ C == A ⊗ (B ⊗ C)
- `ternaryInferenceDistributivity` — A ⊗ (B + C) == A ⊗ B + A ⊗ C

### Pool A (8 specs 31→32)
- +8 invariants

### CODER (3 specs 21→22)
- +3 invariants

**Total:** +28 tests, +14 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (102+ → t27 ~60).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 32 | 8 specs → 32 | 8 specs → 32 |
| CODER target | 3-4 specs → 22 | maintain | 3 specs → 22 |
| Pool B target | 46→47 | 46→47 | maintain |
| Integration target | 30→31 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +36 | +20 | +28 |
| Total invariants | +21 | +15 | +14 |
| Historic milestone | Pool A ≥32 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TerEffic/Sparkle | Sparkle HDL |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥32 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥32 is the natural next step after achieving ≥31
2. CODER has 7 specs at 21 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (102+ vs 57) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W293 once Pool A reaches ≥32

---
