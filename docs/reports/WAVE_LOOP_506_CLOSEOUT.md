# Wave Loop 506 Close-Out Report

**Issue:** #1475 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-506`  
**Variant:** B — model `switch` statements for enum / trit dispatch  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 506 executes **Variant B** from the W506 cooperation plan: it adds `switch` control flow to the Icarus-lowerable formal model. Because the t27 frontend parses `switch` as an expression, the loop first landed an expression-level model (`Expr.switch` lowered to nested `VExpr.ternary` operators) and then added the statement-level construct required by the issue (`Stmt.switch` lowered to procedural Verilog `case` / `default`). Both forms are covered by the generic `module_value_equiv_proved_sequential` theorem.

A scratch witness `w506_switch_enum_dispatch.t27` and a hand-written Lean witness `w506_switch` demonstrate end-to-end lowerability, sequentiality, and value preservation. The Icarus smoke gate stays at **0 documented baseline failures**.

---

## Weak-point analysis

- **Statement-level `switch` was unmodeled.** Existing specs use `switch (disc) { .variant => { stmts } }` as procedural dispatch, but the Icarus-lowerable operational semantics had no `Stmt.switch` constructor and no matching `VStmt.switch`.
- **Expression-level `switch` was also unmodeled.** The frontend parses `switch` as an expression, and return-value dispatch (`return switch (x) { ... }`) could not be classified or verified.
- **Enum values were rejected by the lowerability predicate.** `Expr.enumVal` was treated as non-lowerable, blocking enum-driven `switch` dispatch even though the Verilog backend can emit the numeric constant.
- **The generic equivalence theorem had no `switch` case.** `all_equiv` needed a new branch for both expression and statement `switch` that relates the source case-walker to the emitted Verilog.

---

## Scientific / engineering anchors

- **CompCert / Clight** — fuel-based big-step semantics for conditionals and loops; the `switch` case-walker uses the same fuel-threaded structural recursion. ([Leroy et al., *CompCert*](https://compcert.org/))
- **SystemVerilog LRM** — procedural `case` / `unique case` / `default` semantics; the shallow `VStmt.switch` mirrors the LRM's deterministic case selection with a combinational discriminant.
- **Icarus Verilog** — the emitted `case` blocks are validated against Icarus simulation; the smoke gate confirms no regressions in the generated procedural Verilog.
- **Kami / Bluespec** — Coq-embedded HDL trace refinement; the extended `all_equiv` invariant continues to serve as the forward-simulation bridge from t27 to Verilog.

---

## What changed

### t27 specs and seals

- `specs/scratch/w506_switch_enum_dispatch.t27` — enum `OpKind` dispatch to numeric codes via a statement `switch`.
- `.trinity/seals/scratch_w506_switch_enum_dispatch.json` — deterministic seal for the new spec.

### Lean 4 model

- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`
  - Added `Expr.switch` (expression with case list and default).
  - Added `Stmt.switch` (statement with case list and default body list).

- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`
  - Added `VExpr.ternary` for the nested-ternary lowering of expression `switch`.
  - Added `VStmt.switch` for the procedural `case` / `default` lowering of statement `switch`.

- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
  - Added fuel-threaded total evaluators `evalSwitchCasesTotal` / `evalVTernaryCasesTotal` for expression switch.
  - Added fuel-threaded total evaluators for `Stmt.switch` / `VStmt.switch`.
  - Added `Expr.enumVal` semantics that resolve the variant to a 32-bit numeric literal.

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - Added `Expr.isCombinationalSwitchCaseListFuel`, `Expr.isLowerableSwitchCaseListFuel`, and similar helpers.
  - Made `Expr.enumVal` combinational/lowerable when the enum/variant is known.
  - Made empty switch case lists combinational and lowerable.
  - Added `Stmt.switch` to the sequential/lowerable predicates.

- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
  - Emit `Expr.switch` as a folded chain of `VExpr.ternary` nodes.
  - Emit `Stmt.switch` as a procedural `case (disc) ... default ... endcase` block.
  - Emit `Expr.enumVal` as a 32-bit literal.

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - Added switch helper lemmas (call-context and combinationality decomposition, tail lemmas, foldr/ternary congruence lemmas).
  - Added the expression `switch` case to the `P_expr` branch of `all_equiv`.
  - Added the statement `switch` case to the `P_stmt` branch of `all_equiv`.

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W506 witness environment/module: `w506SwitchEnv` / `w506SwitchModule` / `w506SwitchMain`.

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - Added `w506_switch_lowerable`, `w506_switch_sequential`, and `w506_switch_value_equiv` (for `main(1)`), applying `module_value_equiv_proved_sequential`.

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0 disagreements.
- `./scripts/tri test`:
  - 712 / 712 non-smoke PASS
  - 192 / 192 yosys smoke PASS, 0 baseline failures
  - 192 / 192 Icarus smoke PASS, 0 documented baseline failures
  - 712 / 712 seal matches
  - FPGA board-less smoke gate / replay: OK
  - Standalone lake-package build: OK
  - Gen C / Fixed Point: clean
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- `while` loops remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).
- Only deterministic `case` dispatch is modeled; non-unique or wildcard pattern matching is out of scope.

---

## Next wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W507_2026-07-07.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
