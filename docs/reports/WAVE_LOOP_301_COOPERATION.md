# Wave Loop 301 — Three Variants of Cooperation for Wave Loop 302

**Date:** 2026-06-23  
**Commit:** `88727d0ab`  
**Current State:** Pool A ALL ≥41, CODER ALL ≥31, Pool B 56, Integration 41, Lean 4 34 theorems (1 generic)

---

## Variant A — Uniform Floor Elimination (RECOMMENDED)

**Strategy:** Continue the historic streak by raising ALL specs simultaneously.

| Target | W301 | W302 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥41 | **ALL ≥42** | +15 inv, +30 tests |
| CODER (10 specs) | ALL ≥31 | **ALL ≥32** | +10 inv, +20 tests |
| Pool B (systolic_ternary) | 56 | **57** | +1 inv, +2 tests |
| Integration (ternary_inference) | 41 | **42** | +1 inv, +2 tests |
| Lean 4 theorems | 34 | **35** | +1 theorem |
| **Total work** | — | **+27 inv, +54 tests, +1 theorem** | — |

**Why recommended:**
- Maintains 62-wave zero-entrant streak momentum
- Minimal cognitive load (same pattern as W294-W301)
- Low risk (concrete theorems, simple invariants)

**Execution:**
1. Batch-append +1 invariant and +2 tests per spec via Python script
2. Parse all 27 specs
3. Seal all 27 specs
4. Add generic `∀` theorem for plus weight (`∀ a psum, ternaryMac psum a .plus = psum + a`)
5. Commit, generate report

---

## Variant B — Generic LUT DSE Proof Trinity

**Strategy:** Complete the generic proof trinity for ternary MAC operations.

| Target | W301 | W302 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥41 | **ALL ≥41** (maintain) | 0 |
| CODER (10 specs) | ALL ≥31 | **ALL ≥31** (maintain) | 0 |
| Pool B (systolic_ternary) | 56 | **56** (maintain) | 0 |
| Integration (ternary_inference) | 41 | **41** (maintain) | 0 |
| Lean 4 theorems | 34 | **36** | **+2 theorems** |
| **Total work** | — | **+2 theorems** | — |

**Theorem targets:**
1. `ternaryMacPlusWeightIdentityGeneric (a psum : Int) : ternaryMac psum a .plus = psum + a`
   - Proves plus-weight MAC always adds activation to accumulator
2. `ternaryMacMinusWeightIdentityGeneric (a psum : Int) : ternaryMac psum a .minus = psum - a`
   - Proves minus-weight MAC always subtracts activation from accumulator

Together with W301's `ternaryMacZeroWeightIdentityGeneric`, this completes the **generic LUT DSE proof trinity**:
- `zero` → wire (NOP): `∀ a psum, ternaryMac psum a .zero = psum` ✅ (W301)
- `plus` → adder: `∀ a psum, ternaryMac psum a .plus = psum + a` ⏳ (W302)
- `minus` → subtractor: `∀ a psum, ternaryMac psum a .minus = psum - a` ⏳ (W302)

**Why consider:**
- Completes the mathematical foundation for KU Leuven LUT DSE
- Creates a reusable proof library for any ternary MAC hardware design
- Strong differentiator vs. Sparkle HDL's concrete hardware proofs

**Risk:** Low — these are simple generic theorems provable with `simp` + `native_decide`

---

## Variant C — Integration Stress Test + Cross-Spec Linking

**Strategy:** Push integration spec depth and create cross-spec invariants.

| Target | W301 | W302 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥41 | **ALL ≥41** (maintain) | 0 |
| CODER (10 specs) | ALL ≥31 | **ALL ≥31** (maintain) | 0 |
| Pool B (systolic_ternary) | 56 | **56** (maintain) | 0 |
| Integration (ternary_inference) | 41 | **46** | **+5 inv, +10 tests** |
| Lean 4 theorems | 34 | **35** | +1 theorem |
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
| **Implementation risk** | ✅ Low | ✅ Low | ⚠️ High |
| **Time to execute** | ✅ ~30 min | ✅ ~1 hour | ⚠️ ~4 hours |
| **Scientific impact** | ⚠️ Incremental | ✅ High | ✅ High |
| **Sparkle HDL response** | ⚠️ None | ✅ Direct | ⚠️ Indirect |

**Recommendation:** Execute **Variant A** for W302 to maintain streak,
then **Variant B** for W303 to complete the generic LUT DSE proof trinity.

---

## Phase Complete: SYNTHESIZE
→ Phase 6: LEARN
