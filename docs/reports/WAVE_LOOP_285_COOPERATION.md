# Wave Loop 285 → Wave Loop 286 Cooperation Variants

**Date:** 2026-06-16 | Next Cycle: W286

---

## Current State (End of W285)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥27** (FIRST TIME) — 9 specs @ 27, 6 specs @ 28+ |
| **CODER** | ALL @ 17 (stagnant — need depth push) |
| **Pool B** | systolic_ternary @ 39 |
| **Integration** | ternary_inference @ 23 |
| **Lean 4** | 13 ternary theorems / 48 total |
| **Conformance** | **571/571 PASS** |
| **Zero-entrant** | **50 waves** (absolute record) |

---

## Variant A (Recommended): Pool A Uniform ≥28 + CODER Depth

**Goal:** Raise ALL 15 Pool A specs from 27→28 AND raise 5 CODER specs from 17→18.

### Pool A (15 specs × +1 invariant = +15 invariants)
- 9 specs at 27: adder_tree, backend, bram_weights, cordic, cordic_fixed, cordic_top, eda, formal, gemm, opcodes, rtl, systolic_array, ternary_gemm, ternary_mac, yosys → ALL 28
- 6 specs already at 28+: maintain

### CODER (5 specs × +1 invariant = +5 invariants)
- arch, benchmark, eval, training, weights → 18
- 5 specs already at 17: bench_proxy, dataset, pipeline, prm, tokenizer → maintain or push

### Lean 4
- Add 1 theorem: `ternaryInferenceMixedWeightsConcrete` (mixed +1/-1 weights)

**Total:** +30 tests, +15 invariants, +1 theorem.  
**Milestone:** First time ALL Pool A ≥28.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based троичный MAC spec responding to TOM/VitaLLM/LUT Accelerator.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 5 specs 27→28 (+5 invariants)

### Pool B Depth
- systolic_ternary 39→40 (+2 invariants)

### Lean 4
- `TernaryLUT.lean` with 2 theorems:
  - `lutTernaryMulEquivDirect` — LUT-based equals direct
  - `lutTernaryMulZeroWeightNop` — zero weight is NOP in LUT

**Total:** +18 tests, +12 invariants, +2 theorems, +1 new spec.  
**Milestone:** First LUT-based ternary spec in t27; first hardware-algorithm equivalence for LUT.

---

## Variant C: Proof-Carrying Code (PCC) Pipeline

**Goal:** Create `proof_carrying_code.t27` + `ProofCarryingCode.lean` — every generated artifact carries a machine-checked proof.

### New Spec: `proof_carrying_code.t27`
- 6 tests, 4 invariants
- Proves that generated Rust code preserves ternary semantics
- Seal includes proof hash alongside code hash

### Pool A Depth
- 5 specs 27→28 (+5 invariants)

### CODER Depth
- 3 specs 17→18 (+3 invariants)

### Lean 4
- `ProofCarryingCode.lean` with 1 theorem:
  - `generatedRustPreservesTernarySemantics` — spec → Rust → same output

**Total:** +16 tests, +12 invariants, +1 theorem, +1 new spec.  
**Milestone:** First machine-checked spec-to-code correctness pipeline in t27.

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL ≥28 | 5 specs → 28 | 5 specs → 28 |
| CODER target | 5 specs → 18 | maintain | 3 specs → 18 |
| Pool B target | maintain | 39→40 | maintain |
| New spec | No | `ternary_lut.t27` | `proof_carrying_code.t27` |
| Lean 4 theorems | +1 | +2 | +1 |
| Total tests | +30 | +18 | +16 |
| Total invariants | +15 | +12 | +12 |
| Historic milestone | Pool A ≥28 | First LUT spec | First PCC pipeline |
| Risk | Low | Medium | Medium |
| Competitive response | Depth | TOM/VitaLLM | Formal verification arms race |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥28 + CODER depth).**

Rationale:
1. Pool A uniform ≥28 is the natural next step after achieving ≥27
2. CODER has been stagnant at 17 for 1 wave — needs depth push
3. Lowest risk, highest confidence of success
4. Maintains momentum of uniform floor elimination
5. Variant B (LUT) and C (PCC) can follow in W287 once CODER reaches ≥18

---

**50 waves. 231 stable. Zero entrants. Pool A ALL ≥27. Forward.**
**2026 is the year of Lean 4 HDL.**
