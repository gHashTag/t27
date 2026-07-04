# Wave Loop 289 → Wave Loop 290 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W290

---

## Current State (Post-W289)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥31** (FIRST TIME) — 13 specs @ 31+, backend 32, systolic_array 33 |
| **CODER** | **ALL ≥20** (FIRST TIME) — 10 specs @ 20+, eval 21 |
| **Pool B** | systolic_ternary @ 46 |
| **Integration** | ternary_inference @ 28 |
| **Lean 4** | 19 ternary theorems / 52 total |
| **Zero-entrant streak** | 54 waves (53rd consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥32 + CODER Depth + Lean 4

**Goal:** Raise ALL 13 Pool A specs from 31→32 AND push 3-4 CODER specs from 20→21.

### Pool A (13 specs × +1 invariant = +13 invariants)
- All Pool A specs currently at 31→32
- backend already at 32, systolic_array already at 33

### CODER Depth (3-4 specs × +1 invariant = +3-4 invariants)
- Target: benchmark 20→21, pipeline 20→21, training 20→21, weights 20→21
- Maintain ALL ≥20

### Pool B (1 spec)
- systolic_ternary 46→47 (+1 invariant)

### Integration (1 spec)
- ternary_inference 28→29 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceSparsityImpliesZero` — zero weights → zero output (generic)

**Total:** +34 tests, +19 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥32.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TerEffic / Sparkle.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 5 specs 31→32 (+5 invariants)

### Pool B Depth
- systolic_ternary 46→47 (+1 invariant)

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
- `ternaryInferenceSparsityImpliesZero` — zero weights → zero output (generic)
- `ternaryInferenceIdentityGeneric` — identity weights preserve any input
- `ternaryGemmAssociativity` — GEMM associativity lemma

### Pool A (5 specs 31→32)
- +5 invariants

### CODER (3 specs 20→21)
- +3 invariants

**Total:** +22 tests, +11 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~55).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 32 | 5 specs → 32 | 5 specs → 32 |
| CODER target | 3-4 specs → 21 | maintain | 3 specs → 21 |
| Pool B target | 46→47 | 46→47 | maintain |
| Integration target | 28→29 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +34 | +18 | +22 |
| Total invariants | +19 | +12 | +11 |
| Historic milestone | Pool A ≥32 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TerEffic/Sparkle | Sparkle HDL |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥32 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥32 is the natural next step after achieving ≥31
2. CODER has 5-6 specs at 20 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 52) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W291 once Pool A reaches ≥32

---
