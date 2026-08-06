# Wave Loop 292 → Wave Loop 293 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W293

---

## Current State (Post-W292)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥32** (FIRST TIME) — 8 specs @ 33, 7 specs @ 32 |
| **CODER** | **ALL ≥22** (FIRST TIME) — 7 specs @ 22, 3 specs @ 23 |
| **Pool B** | systolic_ternary @ 47 |
| **Integration** | ternary_inference @ 32 |
| **Lean 4** | 22 ternary theorems / 58 total |
| **Zero-entrant streak** | 57 waves (56th consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥33 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 32→33 AND push 5-6 CODER specs from 22→23.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 32→33
- 8 specs already at 33 (backend, cordic, cordic_fixed, cordic_top, opcodes, rtl, systolic_array, yosys)
- 7 specs at 32 need +1 (adder_tree, bram_weights, eda, formal, gemm, ternary_gemm, ternary_mac)

### CODER Depth (5-6 specs × +1 invariant = +5-6 invariants)
- Target: eval 22→23, pipeline 22→23, prm 22→23, tokenizer 22→23, training 22→23, weights 22→23
- Maintain ALL ≥22

### Pool B (1 spec)
- systolic_ternary 47→48 (+1 invariant)

### Integration (1 spec)
- ternary_inference 32→33 (+1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceLutMinusWeightNegate` — minus weight negates activation (LUT entry)

**Total:** +38 tests, +23 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥33.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to KU Leuven / TENET / TeLLMe v2 / TernaryCore.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 7 specs 32→33 (+7 invariants)

### Pool B Depth
- systolic_ternary 47→48 (+1 invariant)

### Lean 4
- `TernaryLUT.lean` with 2 theorems:
  - `lutTernaryMulEquivDirect` — LUT-based equals direct
  - `lutTernaryMulZeroWeightNop` — zero weight is NOP in LUT

**Total:** +20 tests, +14 invariants, +2 theorems, +1 new spec.
**Milestone:** First LUT-based ternary spec in t27; first hardware-algorithm equivalence for LUT.

---

## Variant C: Lean 4 Proof-Assistant Expansion + RISC-V Response

**Goal:** Add 3 new Lean 4 theorems in response to Sparkle HDL's 162+ theorem count AND OpenVM's 45 RV32IM opcodes.

### Lean 4 (+3 theorems)
- `ternaryInferenceLutMinusWeightNegate` — minus weight negates activation (LUT entry)
- `ternaryInferenceLutPlusWeightIdentity` — plus weight preserves activation (LUT entry)
- `ternaryGemmAssociativityConcrete` — GEMM associativity for 2x2 concrete case

### Pool A (7 specs 32→33)
- +7 invariants

### CODER (3 specs 22→23)
- +3 invariants

**Total:** +26 tests, +13 invariants, +3 theorems.
**Milestone:** Closing gap with Sparkle HDL (162+ → t27 ~61).

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 33 | 7 specs → 33 | 7 specs → 33 |
| CODER target | 5-6 specs → 23 | maintain | 3 specs → 23 |
| Pool B target | 47→48 | 47→48 | maintain |
| Integration target | 32→33 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | No |
| Lean 4 theorems | +1 | +2 | +3 |
| Total tests | +38 | +20 | +26 |
| Total invariants | +23 | +14 | +13 |
| Historic milestone | Pool A ≥33 | First LUT spec | Proof depth |
| Risk | Low | Medium | Low |
| Competitive response | Depth | KU Leuven/TENET/TeLLMe/TernaryCore | Sparkle HDL / OpenVM |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥33 + CODER depth + Lean 4).**

Rationale:
1. Pool A uniform ≥33 is the natural next step after achieving ≥32
2. CODER has 6 specs at 22 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Sparkle HDL gap (162+ vs 58) is structural — Variant C doesn't close it meaningfully; need new spec modules (Variant B) or sustained depth growth
6. Variant B (LUT) should follow in W294 once Pool A reaches ≥33

---
