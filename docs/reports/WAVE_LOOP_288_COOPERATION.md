# Wave Loop 288 → Wave Loop 289 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W289

---

## Current State (Post-W288)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥30** (FIRST TIME) — 8 specs @ 30, 2 @ 31, 2 @ 32, formal @ 39 |
| **CODER** | **ALL ≥19** (FIRST TIME) — 2 specs @ 19, 5 specs @ 20–21, 3 specs @ 24+ |
| **Pool B** | systolic_ternary @ 44 |
| **Integration** | ternary_inference @ 26 |
| **Lean 4** | 19 ternary theorems / 51 total |
| **Zero-entrant streak** | 53 waves (52nd consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥31 + CODER Depth + Lean 4

**Goal:** Raise ALL 8 Pool A specs from 30→31 AND push 2-3 CODER specs from 19→20.

### Pool A (8 specs × +1 invariant = +8 invariants)
- adder_tree, bram_weights, cordic, gemm, opcodes, rtl, ternary_gemm, ternary_mac → ALL 31
- 2 specs already at 31: eda, yosys
- 2 specs already at 32: backend, systolic_array
- formal already at 39

### CODER Depth (2-3 specs × +1 invariant = +2-3 invariants)
- Target: dataset 19→20, prm 19→20, tokenizer 19→20
- Maintain ALL ≥19

### Pool B (1 spec)
- systolic_ternary 44→45 (+2 tests, +1 invariant)

### Integration (1 spec)
- ternary_inference 26→27 (+2 tests, +1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceOutputBounds` — output always within [-S, +S] where S is sum of absolute activations

**Total:** +26 tests, +14 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥31.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TerEffic / Sparkle.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 3 specs 30→31 (+3 invariants)

### Pool B Depth
- systolic_ternary 44→45 (+2 invariants)

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
- `ternaryInferenceOutputBounds` — generic output bounds
- `ternaryInferenceSparsityImpliesZero` — zero weights → zero output (generic)
- `ternaryInferenceIdentityGeneric` — identity weights preserve any input

### Pool A (5 specs 30→31)
- +5 invariants

### CODER (3 specs 19→20)
- +3 invariants

**Total:** +22 tests, +11 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~54).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 31 | 3 specs → 31 | 5 specs → 31 |
| CODER target | 2-3 specs → 20 | maintain | 3 specs → 20 |
| Pool B target | 44→45 | 44→45 | maintain |
| Integration target | 26→27 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +26 | +18 | +22 |
| Total invariants | +14 | +12 | +11 |
| Historic milestone | Pool A ≥31 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TerEffic/Sparkle | Sparkle HDL |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥31 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥31 is the natural next step after achieving ≥30
2. CODER has 2-3 specs at 19 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 51) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W290 once Pool A reaches ≥31

---
