# Wave Loop 507 Close-Out Report

**Issue:** #1476 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-507`  
**Variant:** A — model bounded `while` loops in the Icarus-lowerable subset  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 507 executes **Variant A** from the W507 cooperation plan: it adds bounded `while` loops to the Icarus-lowerable formal model. The Rust Verilog backend already emitted procedural `while` blocks, but the Lean operational semantics, the shallow Verilog model, the lowerability/sequential predicate, and the generic equivalence theorem had no `whileLoop` case.

The loop now consumes one fuel unit per iteration, re-evaluates its combinational condition at every step, and is proven equivalent to the emitted procedural `while (cond) begin ... end` block through a new `P_whileLoop` forward-simulation predicate.

Three scratch witnesses exercise count-up, linear-search, and nested-while-inside-for patterns. All pass the Icarus-lowerability classifier, the yosys smoke gate, the Icarus smoke gate, and the generic sequential equivalence theorem. The Icarus smoke gate stays at **0 documented baseline failures**.

---

## Weak-point analysis

- **`while` was absent from the Icarus-lowerable operational semantics.** The Verilog backend generated `while` blocks, but the Lean model had no `Stmt.whileLoop` or `VStmt.whileLoop` constructor.
- **Fuel semantics for dynamic termination was unverified.** Unlike bounded `for` loops, a `while` re-evaluates a combinational condition each iteration, so the generic equivalence theorem needs a loop-specific fuel invariant.
- **The generic `all_equiv` theorem had no `whileLoop` case.** The forward-simulation predicate had to be extended with a dedicated `P_whileLoop` branch.
- **No Icarus-lowerability witness covered `while`.** The classifier accepted `StmtWhile`, but no scratch spec or Lean witness exercised it end-to-end.

---

## Scientific / engineering anchors

- **CompCert / Clight** — fuel-based big-step semantics for loops; the `while` case mirrors CompCert's condition-then-body recursion with an explicit fuel counter. ([Leroy et al., *CompCert*](https://compcert.org/))
- **CakeML** — clocked big-step semantics for unbounded loops, where each iteration ticks the clock and dynamic termination is built into the evaluator.
- **IEEE 1800 SystemVerilog** — procedural `while` semantics in `always_comb` / function bodies; the shallow `VStmt.whileLoop` mirrors the LRM's condition-guarded repetition.
- **Icarus Verilog** — the emitted `while` blocks are validated against `iverilog -g2005-sv`; the smoke gate confirms no regressions.
- **Kami / Bluespec** — rule-based trace refinement; the extended `all_equiv` invariant continues to serve as the forward-simulation bridge from t27 to Verilog.

---

## What changed

### t27 specs and seals

- `specs/scratch/w507_while_counter.t27` — count-up counter bounded by a `u32` parameter.
- `specs/scratch/w507_while_search.t27` — linear search over a fixed `[5]u32` array, terminating naturally when the array is exhausted.
- `specs/scratch/w507_while_nested.t27` — `while` loop nested inside a bounded `for` loop.
- `.trinity/seals/scratch_w507_while_counter.json`
- `.trinity/seals/scratch_w507_while_search.json`
- `.trinity/seals/scratch_w507_while_nested.json`

### Lean 4 model

- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`
  - Added `Stmt.whileLoop (cond : Expr) (body : List Stmt)`.

- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`
  - Added `VStmt.whileLoop (cond : VExpr) (body : List VStmt)` and placeholder handling.

- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
  - Added `evalWhileLoopTotal` / `evalVWhileLoopTotal`; each iteration consumes one fuel unit and re-evaluates the combinational condition at the smaller fuel.

- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean`
  - Added a partial-model catch-all for `VStmt.whileLoop` to keep the partial evaluator compiling; the total evaluator carries the proof-relevant semantics.

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - Added `whileLoop` cases to `Stmt.isLowerableFuel`, `Stmt.isCombinationalFuel` / `Stmt.isCombinational'`, `Stmt.isSequential'`, `Stmt.functionNamesFuel`, and the combinational/sequential contradiction lemma.

- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
  - Added emission of `Stmt.whileLoop` to `VStmt.whileLoop`.

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - Added helper lemmas for call context, combinationality, sequentiality, and emit-default decomposition.
  - Added `P_whileLoop` and extended `all_equiv` to a 6-tuple.
  - Proved the `Stmt.whileLoop` case by applying the `P_whileLoop` induction hypothesis, keeping fuel accounting aligned with the total evaluators.

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W507 witness environments and modules:
    - `w507WhileCounterEnv` / `w507WhileCounterModule`
    - `w507WhileSearchEnv` / `w507WhileSearchModule`
    - `w507WhileNestedEnv` / `w507WhileNestedModule`

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - Added lowerability, sequentiality, and value-preservation theorems for all three witnesses, each applying `module_value_equiv_proved_sequential`:
    - `w507_while_counter_value_equiv` for `count_to(3)`.
    - `w507_while_search_value_equiv` for `find_index(1)`.
    - `w507_while_nested_value_equiv` for `nested_sum()`.

### Rust classifier

- The existing `IcarusAnalyzer` already recursed into `StmtWhile` conditions and bodies, and the classifier emitted procedural `while` blocks correctly. No Rust code change was required; W507 only had to confirm the backend path through the new witnesses and the Lean gate.

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0 disagreements.
- `./scripts/tri test`:
  - 715 / 715 non-smoke PASS.
  - 195 / 195 yosys smoke PASS, 0 baseline failures.
  - 195 / 195 Icarus smoke PASS, 0 documented baseline failures.
  - 715 / 715 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
  - Gen C / Fixed Point: clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- `break` and `continue` inside loops remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).
- Only deterministic combinational conditions are modeled; non-lowerable side effects in the `while` condition are rejected by the predicate.

---

## Next wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W508_2026-07-07.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
