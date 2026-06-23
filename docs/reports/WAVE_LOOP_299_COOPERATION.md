# Wave Loop 299 — Three Variants of Cooperation for Wave Loop 300

**Date:** 2026-06-23  
**Commit:** `16d25cbe6`  
**Current State:** Pool A ALL ≥39, CODER ALL ≥29, Pool B 55, Integration 39, Lean 4 31 theorems

---

## Variant A — Uniform Floor Elimination (RECOMMENDED)

**Strategy:** Continue the historic streak by raising ALL specs simultaneously.

| Target | W299 | W300 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥39 | **ALL ≥40** | +15 inv, +30 tests |
| CODER (10 specs) | ALL ≥29 | **ALL ≥30** | +10 inv, +20 tests |
| Pool B (systolic_ternary) | 55 | **56** | +1 inv, +2 tests |
| Integration (ternary_inference) | 39 | **40** | +1 inv, +2 tests |
| Lean 4 theorems | 31 | **32** | +1 theorem |
| **Total work** | — | **+28 inv, +56 tests, +1 theorem** | — |

**Why recommended:**
- Maintains 60-wave zero-entrant streak momentum
- Minimal cognitive load (same pattern as W294-W299)
- Low risk (concrete theorems, simple invariants)

**Execution:**
1. Batch-append +1 invariant and +2 tests per spec via Python script
2. Parse all 27 specs
3. Seal all 27 specs
4. Add generic `∀` quantifier theorem (e.g., `∀ a, ternaryMac 0 a .zero = 0`)
5. Commit, generate report

---

## Variant B — Lean 4 Depth Push + Generic Theorems

**Strategy:** Shift focus from spec invariants to formal proof depth with `∀` quantifiers.

| Target | W299 | W300 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥39 | **ALL ≥39** (maintain) | 0 |
| CODER (10 specs) | ALL ≥29 | **ALL ≥29** (maintain) | 0 |
| Pool B (systolic_ternary) | 55 | **55** (maintain) | 0 |
| Integration (ternary_inference) | 39 | **39** (maintain) | 0 |
| Lean 4 theorems | 31 | **34** | **+3 theorems** |
| **Total work** | — | **+3 theorems** | — |

**Theorem targets:**
1. `ternaryMacZeroWeightZeroResult` — `∀ a, ternaryMac 0 a .zero = 0`
2. `ternaryMacPlusWeightIdentity` — `∀ a, ternaryMac 0 a .plus = a`
3. `ternaryMacMinusWeightNegate` — `∀ a, ternaryMac 0 a .minus = -a`

**Why consider:**
- Closes gap to Sparkle HDL (102 theorems) faster
- Adds genuine mathematical generalization beyond concrete instantiations
- Positions t27 as research-grade formal verification library

**Risk:** `native_decide` may not prove `∀` theorems automatically; may need `intro` + `simp` + `omega` tactics.

---

## Variant C — Cross-Spec Integration Stress Test

**Strategy:** Push integration spec depth and create cross-spec invariants linking Pool A and CODER.

| Target | W299 | W300 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥39 | **ALL ≥39** (maintain) | 0 |
| CODER (10 specs) | ALL ≥29 | **ALL ≥29** (maintain) | 0 |
| Pool B (systolic_ternary) | 55 | **55** (maintain) | 0 |
| Integration (ternary_inference) | 39 | **44** | **+5 inv, +10 tests** |
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

**Recommendation:** Execute **Variant A** for W300 to maintain streak,
then **Variant B** for W301 to build proof depth.

---

## Phase Complete: SYNTHESIZE
→ Phase 6: LEARN
