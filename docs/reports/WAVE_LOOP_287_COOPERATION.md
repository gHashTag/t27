# Wave Loop 287 → Wave Loop 288 Cooperation Variants

**Date:** 2026-06-23 | Next Cycle: W288

---

## Current State (Post-W287)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥29** (FIRST TIME) — 11 specs @ 29, 4 specs @ 30+, 2 specs @ 31+ |
| **CODER** | **ALL ≥18** (FIRST TIME) — 2 specs @ 18, 1 @ 19, 5 @ 20+, 2 @ 24 |
| **Pool B** | systolic_ternary @ 43 |
| **Integration** | ternary_inference @ 25 |
| **Lean 4** | 17 ternary theorems / 50 total |
| **Zero-entrant streak** | 52 waves (51st consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥30 + CODER Depth

**Goal:** Raise ALL 11 Pool A specs from 29→30 AND push 2-3 CODER specs from 18→19.

### Pool A (11 specs × +1 invariant = +11 invariants)
- adder_tree, gemm, opcodes, rtl, bram_weights, cordic, ternary_gemm, ternary_mac → 30
- 4 specs already at 30+: cordic_fixed 30, cordic_top 30, eda 31, yosys 31
- 2 specs already at 32+: backend 32, systolic_array 32
- formal already at 39

### CODER Depth (2-3 specs × +1 invariant = +2-3 invariants)
- Target: arch 18→19, bench_proxy 18→19
- Maintain ALL ≥18

### Pool B (1 spec)
- systolic_ternary 43→44 (+2 tests, +1 invariant)

### Integration (1 spec)
- ternary_inference 25→26 (+2 tests, +1 invariant)

### Lean 4 (+1 theorem)
- `ternaryInferenceBitNetStyle` — concrete BitNet-style mixed precision

**Total:** +28 tests, +17 invariants, +1 theorem.
**Milestone:** First time ALL Pool A ≥30.

---

## Variant B: Ternary LUT Spec + Hardware-Algorithm Equivalence

**Goal:** Create `ternary_lut.t27` — LUT-based ternary MAC spec responding to TOM/VitaLLM/KU Leuven.

### New Spec: `ternary_lut.t27`
- 8 tests, 5 invariants
- LUT-based ternary multiplication (no DSP, no multiplier — table lookup)
- Equivalence proof: LUT-based mul == direct ternary mul

### Pool A Depth
- 3 specs 29→30 (+3 invariants)

### Pool B Depth
- systolic_ternary 43→44 (+2 invariants)

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

### CODER Depth
- 2 specs 18→19 (+2 invariants)

### Pool A Depth
- 2 specs 29→30 (+2 invariants)

### Lean 4
- `ProofCarryingCode.lean` with 1 theorem:
  - `generatedRustPreservesTernarySemantics` — spec → Rust → same output

**Total:** +16 tests, +9 invariants, +1 theorem, +1 new spec.
**Milestone:** First machine-checked spec-to-code correctness pipeline in t27.

---

## Comparison Matrix

| Dimension | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Pool A target | ALL → 30 | 3 specs → 30 | 2 specs → 30 |
| CODER target | 2-3 specs → 19 | maintain | 2 specs → 19 |
| Pool B target | 43→44 | 43→44 | maintain |
| Integration target | 25→26 | maintain | maintain |
| New spec | No | `ternary_lut.t27` | `proof_carrying_code.t27` |
| Lean 4 theorems | +1 | +2 | +1 |
| Total tests | +28 | +18 | +16 |
| Total invariants | +17 | +12 | +9 |
| Historic milestone | Pool A ≥30 | First LUT spec | First PCC pipeline |
| Risk | Low | Medium | Medium |
| Competitive response | Depth | TOM/VitaLLM/KU Leuven | Formal verification arms race |

---

## Recommendation

**Execute Variant A (Pool A uniform ≥30 + CODER depth).**

Rationale:
1. Pool A uniform ≥30 is the natural next step after achieving ≥29
2. CODER has 2 specs at floor 18 — need depth push to maintain momentum
3. Lowest risk, highest confidence of success
4. Maintains the rhythm of uniform floor elimination across categories
5. Variant B (LUT) and C (PCC) can follow in W289 once Pool A reaches ≥30

---
