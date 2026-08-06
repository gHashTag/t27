# Wave Loop 293 → Wave Loop 294 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W294

---

## Current State (Post-W293)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥33** (FIRST TIME) — 8 specs @ 33, 8 specs @ 33 |
| **CODER** | **ALL ≥23** (FIRST TIME) — 6 specs @ 23, 4 specs @ 23 |
| **Pool B** | systolic_ternary @ 48 |
| **Integration** | ternary_inference @ 33 |
| **Lean 4** | 23 ternary theorems / 59 total |
| **Zero-entrant streak** | 58 waves (57th consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥34 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 33→34 AND push 6-7 CODER specs from 23→24.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 33→34
- No spec already above 34

### CODER Depth (6-7 specs × +1 invariant = +6-7 invariants)
- Target: ALL 10 specs 23→24
- Maintain ALL ≥23

### Pool B (1 spec)
- systolic_ternary 48→49 (+1 invariant)

### Integration (1 spec)
- ternary_inference 33→34 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceLutMinusWeightNegate` — minus weight negates activation (LUT entry)

**Total:** +46 tests, +24 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥34.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TOM / VitaLLM v2 / TernaryCore / T-MAC.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul
- Responds to Microsoft T-MAC, OpenBitSys vlut.cpp, KU Leuven ternary-lut-dse

### Pool A Depth
- 8 specs 33→34 (+8 invariants)

### Pool B Depth
- systolic_ternary 48→49 (+1 invariant)

### Lean 4
- `TernaryLUT.lean` with 2 theorems:
  - `lutTernaryMulEquivDirect` — LUT-based equals direct
  - `lutTernaryMulZeroWeightNop` — zero weight is NOP in LUT

**Total:** +22 tests, +15 invariants, +2 theorems, +1 new spec.
**Milestone:** First LUT-based ternary spec in t27; first hardware-algorithm equivalence for LUT.

---

## Variant C: Lean 4 Proof-Assistant Expansion + RISC-V Response

**Goal:** Add 3 new Lean 4 theorems AND respond to OpenVM FV / SP1 Lean / Sparkle HDL RISC-V dominance.

### Lean 4 (+3 theorems)
- `ternaryInferenceLutMinusWeightNegate` — minus weight negates activation (LUT entry)
- `ternaryInferenceLutPlusWeightIdentity` — plus weight preserves activation (LUT entry)
- `ternaryGemmAssociativityConcrete` — GEMM associativity for 2x2 concrete case

### Pool A (8 specs 33→34)
- +8 invariants

### CODER (3 specs 23→24)
- +3 invariants

**Total:** +30 tests, +14 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~62).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 34 | 8 specs → 34 | 8 specs → 34 |
| CODER target | ALL → 24 | maintain | 3 specs → 24 |
| Pool B target | 48→49 | 48→49 | maintain |
| Integration target | 33→34 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +46 | +22 | +30 |
| Total invariants | +24 | +15 | +14 |
| Historic milestone | Pool A ≥34 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TOM/VitaLLM/TernaryCore | Sparkle HDL / OpenVM FV |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥34 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥34 is the natural next step after achieving ≥33
2. CODER has 10 specs at 23 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 59) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W295 once Pool A reaches ≥34

---
