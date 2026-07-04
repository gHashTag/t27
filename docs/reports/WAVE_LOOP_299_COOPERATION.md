# Wave Loop 299 → Wave Loop 300 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W300

---

## Current State (Post-W299)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥39** (FIRST TIME) — 15 specs @ 39 |
| **CODER** | **ALL ≥29** (FIRST TIME) — 10 specs @ 29 |
| **Pool B** | systolic_ternary @ 54 |
| **Integration** | ternary_inference @ 39 |
| **Lean 4** | 31 ternary theorems / 66 total |
| **Zero-entrant streak** | 64 waves (63rd consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥40 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 39→40 AND push ALL 10 CODER specs from 29→30.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 39→40
- No spec already above 40

### CODER Depth (10 specs × +1 invariant = +10 invariants)
- Target: ALL 10 specs 29→30

### Pool B (1 spec)
- systolic_ternary 54→55 (+1 invariant)

### Integration (1 spec)
- ternary_inference 39→40 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceUniformActivationsAllPlus` — uniform activations with all-plus weights produce proportional sums

**Total:** +54 tests, +27 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥40; first time ALL CODER ≥30.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TOM / VitaLLM v2 / TernaryCore / T-MAC.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 8 specs 39→40 (+8 invariants)

### Pool B Depth
- systolic_ternary 54→55 (+1 invariant)

### Lean 4
- `TernaryLUT.lean` with 2 theorems

**Total:** +22 tests, +15 invariants, +2 theorems, +1 new spec.
**Milestone:** First LUT-based ternary spec in t27.

---

## Variant C: Lean 4 Proof-Assistant Expansion + RISC-V Response

**Goal:** Add 3 new Lean 4 theorems AND respond to OpenVM FV / SP1 Lean / Sparkle HDL RISC-V dominance.

### Lean 4 (+3 theorems)
- `ternaryInferenceUniformActivationsAllPlus`
- `ternaryInferenceMixedSignOutputBounds`
- `ternaryGemmCommutativityConcrete`

### Pool A (8 specs 39→40)
- +8 invariants

### CODER (3 specs 29→30)
- +3 invariants

**Total:** +30 tests, +14 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~69).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 40 | 8 specs → 40 | 8 specs → 40 |
| CODER target | ALL → 30 | maintain | 3 specs → 30 |
| Pool B target | 54→55 | 54→55 | maintain |
| Integration target | 39→40 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +54 | +22 | +30 |
| Total invariants | +27 | +15 | +14 |
| Historic milestone | Pool A ≥40 + CODER ≥30 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TOM/VitaLLM/TernaryCore | Sparkle HDL / OpenVM FV |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥40 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥40 is a historic milestone — first time crossing the 40-invariant barrier
2. CODER has 10 specs at 29 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 66) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W301 once Pool A reaches ≥40 and CODER ≥30
