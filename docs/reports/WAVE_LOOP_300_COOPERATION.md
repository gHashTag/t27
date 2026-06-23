# Wave Loop 300 — Three Variants of Cooperation for Wave Loop 301

**Date:** 2026-06-23  
**Commit:** `02546a022`  
**Current State:** Pool A ALL ≥40, CODER ALL ≥30, Pool B 55, Integration 40, Lean 4 33 theorems

---

## Variant A — Uniform Floor Elimination (RECOMMENDED)

**Strategy:** Continue the historic streak by raising ALL specs simultaneously.

| Target | W300 | W301 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥40 | **ALL ≥41** | +15 inv, +30 tests |
| CODER (10 specs) | ALL ≥30 | **ALL ≥31** | +10 inv, +20 tests |
| Pool B (systolic_ternary) | 55 | **56** | +1 inv, +2 tests |
| Integration (ternary_inference) | 40 | **41** | +1 inv, +2 tests |
| Lean 4 theorems | 33 | **34** | +1 theorem |
| **Total work** | — | **+27 inv, +54 tests, +1 theorem** | — |

**Why recommended:**
- Maintains 61-wave zero-entrant streak momentum
- Minimal cognitive load (same pattern as W294-W300)
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

| Target | W300 | W301 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥40 | **ALL ≥40** (maintain) | 0 |
| CODER (10 specs) | ALL ≥30 | **ALL ≥30** (maintain) | 0 |
| Pool B (systolic_ternary) | 55 | **55** (maintain) | 0 |
| Integration (ternary_inference) | 40 | **40** (maintain) | 0 |
| Lean 4 theorems | 33 | **36** | **+3 theorems** |
| **Total work** | — | **+3 theorems** | — |

**Theorem targets:**
1. `ternaryGemm2x2EqualsReferenceGeneric` — `∀ a w, ternaryGemm2x2 a w = referenceGemm2x2 a w`
   - **Risk:** `native_decide` will fail; requires manual proof with `intro`, `simp`, `omega`
2. `ternaryMacZeroWeightZeroResult` — `∀ a, ternaryMac 0 a .zero = 0`
   - Likely provable with `intro` + `simp` + `native_decide`
3. `ternaryMacPlusWeightIdentity` — `∀ a, ternaryMac 0 a .plus = a`
   - Likely provable with `intro` + `simp` + `native_decide`

**Why consider:**
- Closes gap to Sparkle HDL (60+ BitNet theorems)
- Adds genuine mathematical generalization
- Positions t27 as research-grade formal verification library

**Risk:** Generic theorems may require manual proof tactics; execution time ~2-4 hours.

---

## Variant C — Integration Stress Test + Cross-Spec Linking

**Strategy:** Push integration spec depth and create cross-spec invariants linking Pool A and CODER.

| Target | W300 | W301 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥40 | **ALL ≥40** (maintain) | 0 |
| CODER (10 specs) | ALL ≥30 | **ALL ≥30** (maintain) | 0 |
| Pool B (systolic_ternary) | 55 | **55** (maintain) | 0 |
| Integration (ternary_inference) | 40 | **45** | **+5 inv, +10 tests** |
| Lean 4 theorems | 33 | **34** | +1 theorem |
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

**Risk:** Cross-spec invariants may require new t27 language features (imports between spec modules).

---

## Decision Matrix

| Criterion | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| **Maintains streak** | ✅ Yes | ⚠️ Pauses | ⚠️ Pauses |
| **Mathematical depth** | ⚠️ Low | ✅ High | ✅ Medium |
| **Competitive moat** | ⚠️ Thin | ✅ Strong | ✅ Strong |
| **Implementation risk** | ✅ Low | ⚠️ High | ⚠️ High |
| **Time to execute** | ✅ ~30 min | ⚠️ ~3 hours | ⚠️ ~4 hours |
| **Scientific impact** | ⚠️ Incremental | ✅ High | ✅ High |
| **Sparkle HDL response** | ⚠️ None | ✅ Direct | ⚠️ Indirect |

**Recommendation:** Execute **Variant A** for W301 to maintain streak,
then **Variant B** for W302 to build proof depth and respond to Sparkle HDL.

---

## Phase Complete: SYNTHESIZE
→ Phase 6: LEARN
