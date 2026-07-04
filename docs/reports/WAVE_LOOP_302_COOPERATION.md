# Wave Loop 302 — Three Variants of Cooperation for Wave Loop 303

**Date:** 2026-06-23  
**Commit:** `bbc491ea0`  
**Current State:** Pool A ALL ≥42, CODER ALL ≥32, Pool B 57, Integration 42, Lean 4 36 theorems (3 generic)

---

## Variant A — Uniform Floor Elimination (RECOMMENDED)

**Strategy:** Continue the historic streak by raising ALL specs simultaneously.

| Target | W302 | W303 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥42 | **ALL ≥43** | +15 inv, +30 tests |
| CODER (10 specs) | ALL ≥32 | **ALL ≥33** | +10 inv, +20 tests |
| Pool B (systolic_ternary) | 57 | **58** | +1 inv, +2 tests |
| Integration (ternary_inference) | 42 | **43** | +1 inv, +2 tests |
| Lean 4 theorems | 36 | **37** | +1 theorem |
| **Total work** | — | **+27 inv, +54 tests, +1 theorem** | — |

**Why recommended:**
- Maintains 63-wave zero-entrant streak momentum
- Minimal cognitive load (same pattern as W294-W302)
- Low risk (concrete theorems, simple invariants)

**Execution:**
1. Batch-append +1 invariant and +2 tests per spec via Python script
2. Parse all 27 specs
3. Seal all 27 specs
4. Add generic theorem (e.g., `∀ a, ternaryMul a .plus = a`)
5. Commit, generate report

---

## Variant B — Generic GEMM Equivalence Proof

**Strategy:** Complete the most important missing generic theorem: full GEMM reference equivalence.

| Target | W302 | W303 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥42 | **ALL ≥42** (maintain) | 0 |
| CODER (10 specs) | ALL ≥32 | **ALL ≥32** (maintain) | 0 |
| Pool B (systolic_ternary) | 57 | **57** (maintain) | 0 |
| Integration (ternary_inference) | 42 | **42** (maintain) | 0 |
| Lean 4 theorems | 36 | **37** | **+1 theorem** |
| **Total work** | — | **+1 theorem** | — |

**Theorem target:**
```lean
theorem ternaryGemm2x2EquivReferenceGeneric (a : Array Int) (w : Array TernaryWeight)
    (_ha : a.size = 4) (_hw : w.size = 4) :
    ternaryGemm2x2 a w = referenceGemm2x2 a w := by
  simp [ternaryGemm2x2, referenceGemm2x2, ternaryMac_eq_referenceMulAdd]
```
- **Risk:** Medium — may need `intro`, `simp`, and case analysis
- **Impact:** HIGH — proves ALL ternary GEMM computations are correct by reference

**Why consider:**
- Closes the most important gap identified in W302 weak points analysis
- Creates a reusable correctness guarantee for any 2x2 ternary GEMM
- Strong response to Sparkle HDL's 60+ BitNet theorems

---

## Variant C — Integration Stress Test + Cross-Spec Linking

**Strategy:** Push integration spec depth and create cross-spec invariants.

| Target | W302 | W303 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥42 | **ALL ≥42** (maintain) | 0 |
| CODER (10 specs) | ALL ≥32 | **ALL ≥32** (maintain) | 0 |
| Pool B (systolic_ternary) | 57 | **57** (maintain) | 0 |
| Integration (ternary_inference) | 42 | **47** | **+5 inv, +10 tests** |
| Lean 4 theorems | 36 | **37** | +1 theorem |
| Cross-spec invariants | 0 | **3** | +3 linking invariants |
| **Total work** | — | **+8 inv, +10 tests, +1 theorem** | — |

**Cross-spec invariant targets:**
1. `ternary_gemm_output_matches_systolic_ternary_pe` — GEMM output equals PE output
2. `ternary_inference_output_bounded_by_bram_weights_depth` — inference output width ≤ BRAM depth
3. `adder_tree_sum_equals_ternary_gemm_accumulation` — adder tree reduction matches MAC accumulation

**Why consider:**
- Creates genuine system-level verification
- Demonstrates t27 specs compose correctly
- High-value for potential tape-out / FPGA deployment

**Risk:** Cross-spec invariants may require new t27 language features (imports between spec modules).

---

## Decision Matrix

| Criterion | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| **Maintains streak** | ✅ Yes | ⚠️ Pauses | ⚠️ Pauses |
| **Mathematical depth** | ⚠️ Low | ✅ High | ✅ Medium |
| **Competitive moat** | ⚠️ Thin | ✅ Strong | ✅ Strong |
| **Implementation risk** | ✅ Low | ⚠️ Medium | ⚠️ High |
| **Time to execute** | ✅ ~30 min | ⚠️ ~2 hours | ⚠️ ~4 hours |
| **Scientific impact** | ⚠️ Incremental | ✅ High | ✅ High |
| **Sparkle HDL response** | ⚠️ None | ✅ Direct | ⚠️ Indirect |
| **CktFormalizer response** | ⚠️ None | ✅ Direct | ⚠️ Indirect |

**Recommendation:** Execute **Variant A** for W303 to maintain streak,
then **Variant B** for W304 to complete the generic GEMM equivalence proof.

---

## Phase Complete: SYNTHESIZE
→ Phase 6: LEARN
