# Wave Loop 503 Close-Out Report

**Issue:** #1472  
**Branch:** `wave-loop-503`  
**Variant:** A — extend Icarus equivalence proof to sequential constructs  
**Date:** 2026-07-12  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loops 499–502 removed reachability clutter, generalized the theorem beyond
`main`, and covered parameterized non-`main` entry points.  Wave Loop 503 widens
the modeled language subset by adding `ifThenElse` and bounded `forLoop` to the
t27 operational semantics, the shallow Verilog semantics, the emitter, the
lowerability predicate, and the generic forward-simulation proof.

The `ifThenElse` case is now covered by the generic
`module_value_equiv_statement` theorem.  Bounded `forLoop` is modeled and
lowerable, but kept outside the generic combinational theorem as a residual
boundary; value preservation for a concrete for-loop witness is proved by
computation with `native_decide`.

The Icarus smoke gate remains at **0 documented baseline failures**.

---

## Weak-point analysis

- **Conditional statements outside the model.**  Real t27 specs use `if` guards
  for early returns and value selection, but the equivalence theorem previously
  covered only assignment, return, and sequential composition.
- **Bounded loops as unmodeled control flow.**  `for` loops are lowered by the
  Rust emitter (constant-range loops are unrolled), yet there was no semantic
  contract in the proof that the unrolled Verilog matches the t27 evaluation.
- **Lowerability predicate gap.**  `isCombinationalFuel` had no rules for `if` or
  `for`, so lowerable sequential programs could be rejected by the classifier.

---

## Scientific / engineering anchors

- **CompCert Clight / Cminor operational semantics** — big-step evaluation of
  `if` and bounded `for` as standard control-flow constructs.
  ([Leroy, Appel, Blazy, Stewart, *CompCert*](https://compcert.org/))
- **CompCert `RTLgenproof`** — structural forward simulation over control-flow
  constructs provides the induction shape used in `all_equiv`.
- **Csmith / YARPGen** — adversarial hand-written witnesses for conditionals and
  loops, mirroring compiler-fuzzing practice.
  ([Yang et al., PLDI 2011](https://doi.org/10.1145/1993316.1993532))
- **Icarus Verilog LRM** — `if` and `for` are supported inside procedural
  contexts; the emitter keeps the generated subset packed-vector friendly.

---

## What changed

### Lean 4 model

- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`
  - Added `VStmt.ifThenElse` and `VStmt.forLoop` constructors.
  - Updated `VStmt.hasPlaceholder` to recurse into the new constructors.

- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
  - Added total evaluation rules for `Stmt.ifThenElse` on the t27 side.
  - Added total evaluation rules for `VStmt.ifThenElse` on the Verilog side.
  - Added `evalForLoopTotal` / `evalVForLoopTotal` helpers and wired bounded
    `forLoop` evaluation into both sides.

- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean`
  - Added partial-evaluation cases for `Stmt.ifThenElse`, `VStmt.ifThenElse`,
    `Stmt.forLoop`, `VStmt.forLoop`, plus `evalVForLoop`.

- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
  - `emitStmt` now emits real `ifThenElse` and `forLoop` shallow-Verilog
    statements when the predicate allows them.
  - Cleaned an unused-variable warning in `widthOfType`.

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - `Stmt.isCombinationalFuel` / `Stmt.isCombinational'` accept `ifThenElse`
    when the condition and both branches are combinational.
  - Bounded `forLoop` remains lowerable but is still rejected by the
    combinational predicate, matching the residual boundary.
  - Added a structural `Stmt.isCombinationalList'` helper in a mutual block.

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - Added default-fuel, succ/zero, call-context, combinational, and
    function-name lemmas for `ifThenElse` and `forLoop`.
  - Extended `all_equiv` with the full `ifThenElse` case.
  - The `forLoop` case falls through to contradiction because the
    combinationality hypothesis is false.

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W503 witness environments and modules:
    `w503IfReturnEnv` / `w503IfReturnModule` and
    `w503ForAccumulatorEnv` / `w503ForAccumulatorModule`.

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - Added lowerability and value-preservation theorems for the two W503
    witnesses.
  - `w503_if_return_pick_true_value_equiv` and
    `w503_if_return_pick_false_value_equiv` apply the generic
    `module_value_equiv_statement` theorem.
  - `w503_for_accumulator_sum_three_value_equiv` uses direct `native_decide`
    because the body contains a `forLoop`.

### t27 specs and seals

- `specs/scratch/w503_if_return.t27` — conditional return of a numeric literal.
- `specs/scratch/w503_for_accumulator.t27` — bounded `for` summing into a local
  variable.
- `.trinity/seals/scratch_w503_if_return.json`
- `.trinity/seals/scratch_w503_for_accumulator.json`

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0
  disagreements.
- `./scripts/tri test`:
  - 705 / 705 non-smoke PASS
  - 185 / 185 yosys smoke PASS, 0 baseline failures
  - 185 / 185 Icarus smoke PASS, 0 documented baseline failures
  - 705 / 705 seal matches
  - FPGA board-less smoke gate / replay: OK
  - Standalone lake-package build: OK
  - Gen C / Fixed Point: clean
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- Bounded `forLoop` is modeled and lowerable, but not yet covered by the generic
  `module_value_equiv_statement` theorem because it is not combinational.
- `while` and `switch` remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

## Next wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W504_2026-07-12.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
