# Wave Loop 504 Close-Out Report

**Issue:** #1473 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-504`  
**Variant:** A — extend generic Icarus equivalence theorem to bounded `forLoop`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 504 closes the last residual boundary left by W503: bounded `forLoop`
is now inside the generic forward-simulation theorem, not just modeled and
lowerable.  The proof is generalized from the purely combinational subset to the
**sequential** subset — statements that are combinational, plus bounded `forLoop`
whose range and body are sequential.  Combinational modules remain a strict
subset, so all prior W501–W503 theorems continue to apply unchanged.

A new scratch witness `w504_for_sum.t27` defines `sum_n(n : u32)` using a bounded
`for i in 0..n` accumulator.  Its value-preservation theorem is the first
for-loop witness proved directly by `module_value_equiv_proved_sequential`
instead of `native_decide`.

The Icarus smoke gate stays at **0 documented baseline failures**.

---

## Weak-point analysis

- **Generic theorem rejected bounded loops.**  W503 added `forLoop` semantics and
demonstrated concrete value preservation by computation, but the generic
`module_value_equiv_statement` still assumed combinationality and could not be
applied to any loop.
- **Fuel handling for loops was unaligned.**  The original loop evaluators recursed
at the *same* fuel for the next iteration, breaking the outer fuel induction that
justifies the loop body.
- **Sequential vs combinational distinction was missing.**  The model needed a
predicate that keeps `ifThenElse` and adds `forLoop`, while preserving the
existing combinational implication chain.

---

## Scientific / engineering anchors

- **CompCert Clight / Cminor** — structural forward simulation over bounded loops
  via an outer fuel induction, matching the `P_forLoop` predicate introduced here.
  ([Leroy et al., *CompCert*](https://compcert.org/))
- **Icarus Verilog LRM** — `for` loops are procedural control flow; the shallow
  model keeps the unrolled behavior bit-exact with the t27 evaluator.
- **Csmith / YARPGen** — hand-written adversarial witnesses (e.g. accumulator
  loops with variable bounds) to stress loop-related compiler fuzzing.
  ([Yang et al., PLDI 2011](https://doi.org/10.1145/1993316.1993532))

---

## What changed

### Lean 4 model

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - Added `Stmt.isSequential'` / `Stmt.isSequentialList'` mutual definitions and
    wrappers.
  - Added `Function.isSequential` and `Module.isSequential`.
  - Proved implication theorems:
    `Stmt.isCombinationalList_implies_isSequentialList'` (pattern-matching recursive),
    `Stmt.isCombinational_implies_isSequential`,
    `Stmt.isCombinationalList_implies_isSequentialList`,
    `Function.isCombinational_implies_isSequential`,
    `Module.isCombinational_implies_isSequential`.

- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
  - Changed `evalForLoopTotal` / `evalVForLoopTotal` so each iteration consumes
    one fuel unit: body evaluation and the next iteration both happen at the
    smaller fuel.  This aligns the loop recursion with the outer fuel induction.

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - Added sequential helper lemmas (`Stmt.isSequential_*`,
    `Module.isSequential_function_body`, `Module.isCombinational_function_body`).
  - Replaced the `all_equiv` module hypothesis with `Module.isSequential`.
  - Added a new `P_forLoop` predicate to the combined forward-simulation invariant.
  - Proved the `Stmt.forLoop` case of `all_equiv` by invoking `P_forLoop`.
  - Proved `P_forLoop` by fuel induction: the body uses the statement IH at the
    smaller fuel, the next iteration uses the loop IH at the smaller fuel.
  - Added `module_value_equiv_proved_sequential` and
    `module_value_equiv_main_sequential`; kept the original combinational
    corollaries as wrappers.

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W504 witness environment/module:
    `w504ForSumEnv` / `w504ForSumModule` / `w504ForSumSumN`.

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - Added `w504_for_sum_lowerable`, `w504_for_sum_sequential`, and
    `w504_for_sum_value_equiv`.
  - The value-equivalence theorem applies the new sequential generic theorem to
    `sum_n(5)`.
  - Updated the W503 `w503_for_accumulator_sum_three_value_equiv` comment to
    remove the stale "generic theorem does not apply" note.

### t27 specs and seals

- `specs/scratch/w504_for_sum.t27` — bounded `for` with a parameter `n`, summing
  `0..n-1` into a local accumulator.
- `.trinity/seals/scratch_w504_for_sum.json`

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 259 lowerable specs, 0
  disagreements.
- `./scripts/tri test`:
  - 706 / 706 non-smoke PASS
  - 186 / 186 yosys smoke PASS, 0 baseline failures
  - 186 / 186 Icarus smoke PASS, 0 documented baseline failures
  - 706 / 706 seal matches
  - FPGA board-less smoke gate / replay: OK
  - Standalone lake-package build: OK
  - Gen C / Fixed Point: clean
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- `while` and `switch` remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

## Next wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W505_2026-07-07.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
