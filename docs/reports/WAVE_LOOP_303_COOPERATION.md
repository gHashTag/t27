# Wave Loop 303 — Three Variants of Cooperation for Wave Loop 304

**Date:** 2026-06-23  
**Commit:** \`99a13a490\`  
**Current State:** Pool A ALL ≥43, CODER ALL ≥33, Pool B 58, Integration 43, Lean 4 39 theorems (6 generic)

---

## Variant A — Uniform Floor Elimination (RECOMMENDED)

**Strategy:** Continue the historic streak by raising ALL specs simultaneously.

| Target | W303 | W304 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥43 | **ALL ≥44** | +15 inv, +30 tests |
| CODER (10 specs) | ALL ≥33 | **ALL ≥34** | +10 inv, +20 tests |
| Pool B (systolic_ternary) | 58 | **59** | +1 inv, +2 tests |
| Integration (ternary_inference) | 43 | **44** | +1 inv, +2 tests |
| Lean 4 theorems | 39 | **40** | +1 theorem |
| **Total work** | — | **+27 inv, +54 tests, +1 theorem** | — |

**Why recommended:**
- Maintains 64-wave zero-entrant streak momentum
- Minimal cognitive load (same pattern as W294-W303)
- Low risk (concrete theorems, simple invariants)

**Execution:**
1. Batch-append +1 invariant and +2 tests per spec via Python script
2. Parse all 27 specs
3. Seal all 27 specs
4. Add generic theorem (e.g., \`∀ a, ternaryMul a .minus = -a\`)
5. Commit, generate report

---

## Variant B — Generic GEMM Equivalence Proof

**Strategy:** Complete the most important missing generic theorem: full GEMM reference equivalence.

| Target | W303 | W304 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥43 | **ALL ≥43** (maintain) | 0 |
| CODER (10 specs) | ALL ≥33 | **ALL ≥33** (maintain) | 0 |
| Pool B (systolic_ternary) | 58 | **58** (maintain) | 0 |
| Integration (ternary_inference) | 43 | **43** (maintain) | 0 |
| Lean 4 theorems | 39 | **40** | **+1 theorem** |
| **Total work** | — | **+1 theorem** | — |

**Theorem target:**
\`\`\`lean
theorem ternaryGemm2x2EquivReferenceGeneric (a : Array Int) (w : Array TernaryWeight)
    (_ha : a.size = 4) (_hw : w.size = 4) :
    ternaryGemm2x2 a w = referenceGemm2x2 a w := by
  simp [ternaryGemm2x2, referenceGemm2x2, ternaryMac_eq_referenceMulAdd]
\`\`\`
- **Risk:** Medium — may need \`intro\`, \`simp\", and case analysis on weight codes
- **Impact:** HIGH — proves ALL ternary GEMM computations are correct by reference

**Why consider:**
- Closes the most important gap identified in W303 weak points analysis
- Creates a reusable correctness guarantee for any 2x2 ternary GEMM
- Strong response to Sparkle HDL's 60+ BitNet theorems and CktFormalizer autoformalization

---

## Variant C — Integration Stress Test + Cross-Spec Linking

**Strategy:** Push integration spec depth and create cross-spec invariants.

| Target | W303 | W304 | Δ |
|--------|------|------|---|
| Pool A (15 specs) | ALL ≥43 | **ALL ≥43** (maintain) | 0 |
| CODER (10 specs) | ALL ≥33 | **ALL ≥33** (maintain) | 0 |
| Pool B (systolic_ternary) | 58 | **58** (maintain) | 0 |
| Integration (ternary_inference) | 43 | **48** | **+5 inv, +10 tests** |
| Lean 4 theorems | 39 | **40** | +1 theorem |
| Cross-spec invariants | 0 | **3** | +3 linking invariants |
| **Total work** | — | **+8 inv, +10 tests, +1 theorem** | — |

**Cross-spec invariant targets:**
1. \`ternary_gemm_output_matches_systolic_ternary_pe\` — GEMM output equals PE output
2. \`ternary_inference_output_bounded_by_bram_weights_depth\` — inference width ≤ BRAM depth
3. \`adder_tree_sum_equals_ternary_gemm_accumulation\` — adder tree matches MAC accumulation

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

**Recommendation:** Execute **Variant A** for W304 to maintain streak,
then **Variant B** for W305 to complete the generic GEMM equivalence proof.

---

## Phase Complete: SYNTHESIZE
→ Phase 6: LEARN
