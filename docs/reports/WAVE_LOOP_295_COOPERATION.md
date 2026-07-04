# Wave Loop 295 → Wave Loop 296 Cooperation Variants

**Date:** 2026-06-16 | Next Cycle: W296

---

## Current State (Post-W295)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥35** (FIRST TIME) — 15 specs @ 35 |
| **CODER** | **ALL ≥25** (FIRST TIME) — 10 specs @ 25 |
| **Pool B** | systolic_ternary @ 50 |
| **Integration** | ternary_inference @ 35 |
| **Lean 4** | 27 ternary theorems / 62 total |
| **Zero-entrant streak** | 60 waves (59th consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥36 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 35→36 AND push CODER specs from 25→26.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 35→36
- No spec already above 36

### CODER Depth (10 specs × +1 invariant = +10 invariants)
- Target: ALL 10 specs 25→26
- Maintain ALL ≥25

### Pool B (1 spec)
- systolic_ternary 50→51 (+1 invariant)

### Integration (1 spec)
- ternary_inference 35→36 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceAllWeightsPlusSum` — all-plus weights produce sum of activations

**Total:** +54 tests, +27 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥36; first time ALL CODER ≥26.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TOM / VitaLLM v2 / TernaryCore / T-MAC.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 8 specs 35→36 (+8 invariants)

### Pool B Depth
- systolic_ternary 50→51 (+1 invariant)

### Lean 4
- `TernaryLUT.lean` with 2 theorems

**Total:** +22 tests, +15 invariants, +2 theorems, +1 new spec.
**Milestone:** First LUT-based ternary spec in t27.

---

## Variant C: Lean 4 Proof-Assistant Expansion + RISC-V Response

**Goal:** Add 3 new Lean 4 theorems AND respond to OpenVM FV / SP1 Lean / Sparkle HDL RISC-V dominance.

### Lean 4 (+3 theorems)
- `ternaryInferenceAllWeightsPlusSum`
- `ternaryInferenceMixedSignOutputBounds`
- `ternaryGemmCommutativityConcrete`

### Pool A (8 specs 35→36)
- +8 invariants

### CODER (3 specs 25→26)
- +3 invariants

**Total:** +30 tests, +14 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~65).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 36 | 8 specs → 36 | 8 specs → 36 |
| CODER target | ALL → 26 | maintain | 3 specs → 26 |
| Pool B target | 50→51 | 50→51 | maintain |
| Integration target | 35→36 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +54 | +22 | +30 |
| Total invariants | +27 | +15 | +14 |
| Historic milestone | Pool A ≥36 + CODER ≥26 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TOM/VitaLLM/TernaryCore | Sparkle HDL / OpenVM FV |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥36 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥36 is the natural next step after achieving ≥35
2. CODER has 10 specs at 25 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 62) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W297 once Pool A reaches ≥36 and CODER ≥26
