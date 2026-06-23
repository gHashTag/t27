# Wave Loop 302 — Cooperation Variants for W303

**Date:** 2026-06-23  
**Commit:** `0cafedc09`  
**Current State:** Pool A ALL ≥42, CODER ALL ≥32, Pool B 57, Integration 42, Lean 4 36 theorems  
**Zero-Entrant Streak:** 63 waves

---

## Executive Summary

Three cooperation variants are proposed for W303.
The default is **Variant A**, which continues the uniform floor elimination strategy
and adds a new generic `∀` quantifier theorem (GEMM reference equivalence).
**Variant B** targets the high-risk generic GEMM equivalence proof.
**Variant C** is a deep-integration variant combining Pool A ↔ CODER cross-spec invariants.

---

## Variant A — Uniform Floor Elimination + Generic GEMM Lemma (Recommended)

**Objective:** Maintain the uniform floor elimination momentum while extending generic proof coverage.

**Target Commit:** `feat(wave-loop-303): +27 invariants +54 tests +1 Lean theorem`

### Pool A: ALL 15 specs 42→43
- +1 invariant per spec (15 total)
- +2 tests per spec (30 total)
- Expected seals: `race_igla-adder_tree-43`, ..., `race_igla-yosys-43`

### CODER: ALL 10 specs 32→33
- +1 invariant per spec (10 total)
- +2 tests per spec (20 total)
- Expected seals: `coder_igla-arch-33`, ..., `coder_igla-weights-33`

### Pool B: systolic_ternary 57→58
- +1 invariant, +2 tests

### Integration: ternary_inference 42→43
- +1 invariant, +2 tests

### Lean 4: +1 Generic Theorem
- **Theorem:** `∀ a w, ternaryGemm2x2 a w = referenceGemm2x2 a w`
- **Proof Strategy:** Expand `ternaryGemm2x2` to element-wise sum over `ternaryMac` applications;
  for each position, apply the LUT DSE proof trinity (zero=wire, plus=add, minus=sub)
  and show the result matches the reference GEMM definition.
- **Tactic:** `simp` + `omega` + `ring` (manual case analysis may be needed)
- **Risk:** High — may require manual proof; fallback to concrete theorems if blocked

**Success Criteria:**
- All 27 specs parse with 0 errors
- All 27 seals regenerate
- Lean 4 build passes in <2s
- `lake exe trinity` reports all theorems verified

---

## Variant B — Deep Generic GEMM Equivalence Proof

**Objective:** Prioritize a single high-impact generic theorem over uniform floor expansion.

**Target Commit:** `feat(wave-loop-303): +0 invariants +0 tests +1 Lean generic theorem`

### Pool A / CODER: Maintain Current Floors
- No new invariants or tests
- Re-seal existing specs to ensure no drift

### Pool B / Integration: Maintain Current Depth
- No changes

### Lean 4: +1 Deep Generic Theorem
- **Theorem:** `∀ a w, ternaryGemm2x2 a w = referenceGemm2x2 a w`
- **Approach:** Write a manual proof in Lean 4:
  ```lean
  theorem ternaryGemm2x2EquivReferenceGeneric (a : Array Int) (w : Array TernaryWeight) :
      ternaryGemm2x2 a w = referenceGemm2x2 a w := by
    unfold ternaryGemm2x2 referenceGemm2x2
    simp [ternaryMac_eq_acc_plus_mul, ternaryMul_eq_mul_decode, ternaryDecode]
    <;> try { omega }
    <;> try { ring }
    <;> try native_decide
  ```
- **Risk:** High — may require restructuring the theorem to expose structure to Lean's automation
- **Fallback:** If blocked, add 2-3 simpler generic theorems (e.g., associativity of `ternaryMac`)

### Deliverables
- Full formal proof in `TernaryInference.lean`
- A new `TernaryInferenceGemm.lean` module if the proof is large

**Success Criteria:**
- Lean 4 build passes
- Theorem is fully generic (`∀ a w`)
- Proof is readable and documented

---

## Variant C — Cross-Spec Integration + Deep Pool B

**Objective:** Build integration depth and cross-spec invariants linking hardware and software specs.

**Target Commit:** `feat(wave-loop-303): +5 invariants +10 tests +5 cross-spec`

### Pool A: Maintain floors (42)
- No new invariants

### CODER: Maintain floors (32)
- No new invariants

### Pool B: systolic_ternary 57→62
- +5 invariants, +10 tests
- Focus: Systolic array ternary GEMM invariants (tiling, blocking, dataflow)

### Integration: ternary_inference 42→48
- +6 invariants, +12 tests
- Focus: End-to-end inference pipeline properties (output correctness bounds, sparsity propagation)

### Cross-Spec Invariants: +5
Link hardware specs (Pool A) to software specs (CODER):
1. `ternary_mac output matches ternary_gemm cell output` — Pool A ↔ Pool A
2. `gemm output bounds match ternary_inference output bounds` — Pool A ↔ Integration
3. `tokenizer encoding preserves adder_tree precision` — CODER ↔ Pool A
4. `weights ternary quantization matches bram_weights storage` — CODER ↔ Pool A
5. `benchmark latency matches rtl cycle count` — CODER ↔ Pool A

**Deliverables:**
- 5 new cross-spec invariant files under `specs/igla/integration/cross_*.t27`
- Updated integration spec with cross-spec references

**Success Criteria:**
- All cross-spec invariants parse and seal
- Integration spec references all cross-spec invariants
- No circular dependencies between spec layers

---

## Comparative Summary

| Dimension | Variant A (Recommended) | Variant B (High-Risk) | Variant C (Deep) |
|-----------|----------------------|----------------------|------------------|
| Invariants added | +27 | +0 | +11 |
| Tests added | +54 | +0 | +22 |
| Lean 4 theorems | +1 generic | +1 deep generic | +0 |
| Cross-spec invariants | +0 | +0 | +5 |
| Risk | Low | **High** | Medium |
| Impact on Pool A | Floor ↑ 42→43 | None | None |
| Impact on CODER | Floor ↑ 32→33 | None | None |
| Impact on Lean 4 | +1 generic | +1 deep generic | None |
| Time estimate | ~20 min | ~40 min (with risk) | ~25 min |
| Recommended phase | W303 (now) | W304 (next) | W305 |

---

## Decision

**Execute Variant A for W303.**

The uniform floor elimination strategy has proven robust across 13 waves.
Maintaining momentum while adding one generic theorem is the optimal risk/reward.
Variant B (deep generic proof) is deferred to W304 with a dedicated time block.
Variant C (cross-spec integration) is deferred to W305 when Pool A and CODER floors stabilize.

**Phase complete: SYNTHESIZE**
→ Phase 6: LEARN
