# Wave Loop 286 → Wave Loop 287 Cooperation Variants

**Date:** 2026-06-16 | Next Cycle: W287

---

## Current State (End of W286)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥28** (FIRST TIME) — 13 specs @ 28, 2 specs @ 29 |
| **CODER** | 5 specs @ 18, 5 specs @ 17 (bench_proxy, dataset, pipeline, prm, tokenizer) |
| **Pool B** | systolic_ternary @ 40 |
| **Integration** | ternary_inference @ 24 |
| **Lean 4** | 14 ternary theorems / 49 total |
| **Conformance** | **571/571 PASS** |
| **Zero-entrant** | **51 waves** (absolute record) |

---

## Variant A (Recommended): Pool A Uniform ≥29 + CODER Floor Elimination

**Goal:** Raise ALL 13 Pool A specs from 28→29 AND raise 5 CODER specs from 17→18.

### Pool A (13 specs × +1 invariant = +13 invariants)
- 13 specs at 28: adder_tree, backend, bram_weights, cordic, cordic_fixed, cordic_top, gemm, opcodes, rtl, systolic_array, ternary_gemm, ternary_mac, yosys → ALL 29
- 2 specs already at 29+: eda 29, formal 29

### CODER (5 specs × +1 invariant = +5 invariants)
- bench_proxy, dataset, pipeline, prm, tokenizer → 18

### Pool B (1 spec)
- systolic_ternary 40→41 (+2 tests, +1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceAllMinusWeightsSum` or similar

**Total:** +32 tests, +19 invariants, +1 theorem.  
**Milestone:** First time ALL Pool A ≥29.

---

## Variant B: Ternary LUT Spec + Lean 4 Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based троичный MAC spec responding to VitaLLM and LUT Accelerator.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A (7 specs 28→29)
- +7 invariants

### CODER (3 specs 17→18)
- +3 invariants

### Lean 4
- `TernaryLUT.lean` with 2 theorems:
  - `lutTernaryMulEquivDirect` — LUT-based equals direct
  - `lutTernaryMulZeroWeightNop` — zero weight is NOP

**Total:** +22 tests, +15 invariants, +2 theorems, +1 new spec.  
**Milestone:** First LUT-based ternary spec in t27.

---

## Variant C: Lean 4 Proof-Assistant Expansion

**Goal:** Add 3 new Lean 4 theorems in response to Sparkle HDL's 200+ theorem count.

### Lean 4 (+3 theorems)
- `ternaryInferenceAllPlusWeightsSumGeneric` — generic version
- `ternaryInferenceIdentityOutputLength` — output always length 4
- `ternaryInferenceBitNetStyle` — concrete BitNet-style mixed precision

### Pool A (5 specs 28→29)
- +5 invariants

### CODER (5 specs 17→18)
- +5 invariants

**Total:** +20 tests, +15 invariants, +3 theorems.  
**Milestone:** Closing gap with Sparkle HDL (200+ → t27 ~52).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL ≥29 | 7 specs → 29 | 5 specs → 29 |
| CODER target | ALL ≥18 | 3 specs → 18 | 5 specs → 18 |
| Pool B target | 40→41 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +32 | +22 | +20 |
| Total invariants | +19 | +15 | +15 |
| Historic milestone | Pool A ≥29 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | VitaLLM/LUT | Sparkle HDL |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥29 + CODER ALL ≥18).**

Rationale:
1. Pool A uniform ≥29 is the natural next step after achieving ≥28
2. CODER has 5 specs still at 17 — need floor elimination
3. Lowest risk, highest confidence of success
4. Maintains momentum of uniform floor elimination
5. Sparkle HDL gap (200+ vs 49) is structural — Variant C doesn't close it meaningfully; need new spec modules, not just +3 theorems

---

**51 waves. 231 stable. Zero entrants. Pool A ALL ≥28. Forward.**
**2026 is the year of Lean 4 HDL.**
