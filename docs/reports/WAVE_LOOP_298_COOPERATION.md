# Wave Loop 298 — Three Variants of Cooperation for Wave Loop 299

**Date:** 2026-06-16  
**Commit:** `9271e4fab`  
**Current State:** Pool A ALL ≥38, CODER ALL ≥28, Pool B 54, Integration 38, Lean 4 31 theorems

---

## Variant A — Uniform Floor Elimination (RECOMMENDED)

**Strategy:** Continue the historic streak by raising ALL specs simultaneously.

| Target | W298 | W299 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥38 | **ALL ≥39** | +15 inv, +30 tests |
| CODER (10 specs) | ALL ≥28 | **ALL ≥29** | +10 inv, +20 tests |
| Pool B (systolic_ternary) | 54 | **55** | +1 inv, +2 tests |
| Integration (ternary_inference) | 38 | **39** | +1 inv, +2 tests |
| Lean 4 theorems | 31 | **32** | +1 theorem |
| **Total work** | — | **+28 inv, +56 tests, +1 theorem** | — |

**Why recommended:**
- Maintains 59-wave zero-entrant streak momentum
- Minimal cognitive load (same pattern as W294-W298)
- Low risk (concrete theorems, simple invariants)
- Automatic branch creation supported

**Execution:**
1. Batch-append +1 invariant and +2 tests per spec via Python script
2. Parse all 27 specs
3. Seal all 27 specs
4. Add `ternaryGemm2x2EqualsReference` theorem
5. Commit, generate report

---

## Variant B — Lean 4 Depth Push

**Strategy:** Shift focus from spec invariants to formal proof depth.

| Target | W298 | W299 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥38 | **ALL ≥38** (maintain) | 0 |
| CODER (10 specs) | ALL ≥28 | **ALL ≥28** (maintain) | 0 |
| Pool B (systolic_ternary) | 54 | **54** (maintain) | 0 |
| Integration (ternary_inference) | 38 | **38** (maintain) | 0 |
| Lean 4 theorems | 31 | **34** | **+3 theorems** |
| **Total work** | — | **+3 theorems** | — |

**Theorem targets:**
1. `ternaryGemm2x2EqualsReference` — prove ternary GEMM ≡ reference GEMM for all 2x2 inputs
2. `ternaryInferenceAssociativity` — prove (A ⊕ B) ⊕ C = A ⊕ (B ⊕ C) for ternary MAC
3. `ternaryInferenceDistributivity` — prove a·(w₁+w₂) = a·w₁ + a·w₂ for ternary weights

**Why consider:**
- Closes gap to Sparkle HDL (102 theorems) faster
- Adds genuine mathematical value beyond concrete checks
- Positions t27 as research-grade formal verification library

**Risk:** `native_decide` may timeout on generic theorems; may need manual proof tactics.

---

## Variant C — Integration Stress Test + Cross-Spec Linking

**Strategy:** Push integration spec depth and create cross-spec invariants.

| Target | W298 | W299 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥38 | **ALL ≥38** (maintain) | 0 |
| CODER (10 specs) | ALL ≥28 | **ALL ≥28** (maintain) | 0 |
| Pool B (systolic_ternary) | 54 | **54** (maintain) | 0 |
| Integration (ternary_inference) | 38 | **43** | **+5 inv, +10 tests** |
| Lean 4 theorems | 31 | **32** | +1 theorem |
| Cross-spec invariants | 0 | **3** | +3 linking invariants |
| **Total work** | — | **+8 inv, +10 tests, +1 theorem** | — |

**Cross-spec invariant targets:**
1. `ternary_gemm_output_matches_systolic_ternary_pe` — prove GEMM output equals PE output for same inputs
2. `ternary_inference_output_bounded_by_bram_weights_depth` — prove inference output width ≤ BRAM depth
3. `adder_tree_sum_equals_ternary_gemm_accumulation` — prove adder tree reduction matches MAC accumulation

**Why consider:**
- Creates genuine system-level verification (not just module-level)
- Demonstrates t27 specs compose correctly
- High-value for potential tape-out / FPGA deployment

**Risk:** Cross-spec invariants may require new t27 language features (imports between spec modules); may need compiler support.

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
| **User value** | ✅ High (CI green) | ⚠️ Medium (research) | ✅ High (system-level) |

**Recommendation:** Execute **Variant A** for W299 to maintain streak,
then alternate with **Variant B** every 3rd wave (W300, W303, ...) to
build proof depth without losing floor momentum.

---

## Phase Complete: SYNTHESIZE
→ Phase 6: LEARN
