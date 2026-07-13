# Wave Loop 508 Close-Out Report

**Issue:** #1477 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-508`  
**Variant:** A — model `break` / `continue` in bounded loops within the Icarus-lowerable subset  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 508 executes **Variant A** from the W508 cooperation plan: it adds `break` and `continue` control flow to the Icarus-lowerable formal model. The Rust Verilog backend already parsed these keywords, but it emitted placeholder Verilog (`disable fork;` for `break` and a commented-out `continue;`) and the Lean operational semantics had no `Stmt.break`/`Stmt.continue` constructors.

The implementation threads sentinel exit flags (`__break`, `__continue`, `__return`) through the fuel-based total statement-list evaluator, extends the shallow Verilog model with the same flag semantics, and rejects `break`/`continue` outside loop bodies in both the Lean lowerability predicate and the Rust host-only classifier. The generated Verilog now uses explicit per-loop break/skip flags and per-statement guards, which are accepted by both **yosys** and **Icarus Verilog**.

Three scratch witnesses exercise early-exit search, guarded accumulation, and nested early exit. All pass the Icarus-lowerability classifier, the yosys smoke gate, the Icarus smoke gate, and the generic sequential equivalence theorem. The Icarus smoke gate stays at **0 documented baseline failures**.

---

## Weak-point analysis

- **`break`/`continue` were absent from the Icarus-lowerable operational semantics.** The Verilog backend generated placeholder statements, and the Lean model had no `Stmt.break`/`Stmt.continue` or `VStmt.break`/`VStmt.continue`.
- **Early-exit control flow required a shared exit-flag discipline.** Unlike straight-line sequential code, `break`/`continue` affect the evaluation of the *remaining* statements in a loop body and the loop's termination decision.
- **The generic `all_equiv` theorem had no `break`/`continue` cases.** The forward-simulation invariant had to be extended so that both the t27 semantics and the shallow Verilog model consume and clear loop-exit flags consistently.
- **Neither yosys nor Icarus supports native `break`/`continue` in the generated procedural functions.** Yosys rejects `disable`, and Icarus rejects both `break;` and `continue;` outright. A portable flag-based encoding was required.
- **No Icarus-lowerability witness covered `break`/`continue`.** The classifier accepted `StmtBreak`/`StmtContinue` unconditionally, so out-of-loop usages were not rejected.

---

## Scientific / engineering anchors

- **CakeML “Functional Big-Step Semantics”** (Owens, Myreen, Kumar, Tan, ESOP 2016) — clocked / fuel-threaded big-step semantics for loops with early exit; the sentinel-flag approach mirrors the way CakeML threads an exception-like exit status through evaluation.
- **CompCert / Clight** — fuel-based big-step semantics for loops, where each iteration ticks the clock and dynamic termination is built into the evaluator.
- **IEEE 1800 SystemVerilog** — procedural loop control; the emitted Verilog uses ordinary `reg` flags and `if` guards, which are valid in both Verilog-2005 and SystemVerilog.
- **Icarus Verilog / Yosys** — the emitted flag encoding is validated against `iverilog -g2005-sv` and `yosys read_verilog -sv`; both tools accept the encoding.
- **Kami / Bluespec** — rule-based trace refinement; the extended `all_equiv` invariant continues to serve as the forward-simulation bridge from t27 to Verilog.

---

## What changed

### t27 specs and seals

- `specs/scratch/w508_break_search.t27` — `while` loop that exits early on a target match.
- `specs/scratch/w508_continue_sum.t27` — bounded `for` loop that skips even indices with `continue`.
- `specs/scratch/w508_break_nested.t27` — `break` inside a nested `while` inside a `for`.
- `.trinity/seals/scratch_w508_break_search.json`
- `.trinity/seals/scratch_w508_continue_sum.json`
- `.trinity/seals/scratch_w508_break_nested.json`
- All existing seals were regenerated (NMSE reseal) because the Verilog code generator now emits per-loop flag variables and guards.

### Rust backend

- `bootstrap/src/compiler.rs`
  - `fn_body_has_unlowerable_construct` now tracks loop nesting depth and treats a top-level `break` or `continue` outside any loop as unlowerable.
  - Verilog code generation (`VerilogCodegen`) now maintains a per-loop break/skip flag stack.
  - `gen_verilog_while_stmt`, `gen_verilog_for_stmt`, and `gen_verilog_for_range_stmt` declare unique `__break_flag_n` / `__skip_flag_n` registers, reset them before the loop, clear the skip flag at the top of each iteration, and guard the loop condition with `!__break_flag_n`.
  - `gen_verilog_stmt` wraps every non-local statement inside a loop body with `if (!__break_flag_n && !__skip_flag_n) begin ... end`.
  - `StmtBreak` sets both the break and skip flags; `StmtContinue` sets only the skip flag. This produces the same control-flow effect without relying on `break`/`continue`/`disable` statements that yosys and Icarus reject.

### Lean 4 model

- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`
  - Added `Stmt.break` and `Stmt.continue`.

- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`
  - Added `VStmt.break` and `VStmt.continue`.

- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
  - Added sentinel helpers: `breakFlag`, `continueFlag`, `returnFlag`, `hasExitFlag`, `isBreakFlagSet`, `isContinueFlagSet`, `isReturnFlagSet`, `setFlag`, `clearLoopFlags`.
  - `evalStmtsTotal` short-circuits when any exit flag is set.
  - `evalForLoopTotal` / `evalVForLoopTotal` and `evalWhileLoopTotal` / `evalVWhileLoopTotal` consume and clear `break`/`continue` flags after each iteration, exiting when `break` or `return` is set.
  - `evalStmtTotal` / `evalVStmtTotal` handle `break`, `continue`, and `return_ (some/none)` by setting the corresponding flag.

- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean`
  - Added fallback cases for the legacy partial evaluator so `break`/`continue` remain total.

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - Added contextual loop-control validity (`Stmt.hasValidLoopControlFuel` and list/switch variants) inside the fuel-threaded mutual block.
  - `break`/`continue` are lowerable only when `inLoop` is true.
  - Added wrappers `Stmt.hasValidLoopControl`, `Function.hasValidLoopControl`, `Module.hasValidLoopControl`, and included the check in `Module.isLowerable`.
  - Updated `Stmt.isSequential'`, `Stmt.isCombinationalFuel`, and `Stmt.isCombinationalList_implies_isSequentialList'` with `break`/`continue` cases.

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - Added reduction lemmas for `break`/`continue`/`return` in both t27 and shallow Verilog evaluators.
  - Updated sequential-list reduction lemmas to account for the new `hasExitFlag` guard.
  - Updated `P_stmt`, `P_stmts`, `P_forLoop`, and `P_whileLoop` proofs to use `clearLoopFlags` and the new sentinel-flag invariant.

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
  - Added W508 witness environments and modules:
    - `w508BreakSearchEnv` / `w508BreakSearchModule` / `w508BreakSearchFindTarget`
    - `w508ContinueSumEnv` / `w508ContinueSumModule` / `w508ContinueSumSumOdd`
    - `w508BreakNestedEnv` / `w508BreakNestedModule` / `w508BreakNestedFindPair`
  - Lowerability theorems for the three witnesses proved with `native_decide`.

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - Sequentiality and value-preservation theorems for all three witnesses, each applying `module_value_equiv_proved_sequential`:
    - `w508_break_search_value_equiv` for `find_index(1)`.
    - `w508_continue_sum_value_equiv` for `sum_odd()`.
    - `w508_break_nested_value_equiv` for `find_pair()`.
  - Negative witness `w508BreakOutsideLoopEnv` / `w508BreakOutsideLoopModule` / `w508BreakOutsideLoopBad` and theorem `w508_break_outside_loop_not_lowerable` showing that a top-level `break` is rejected.

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0 disagreements.
- `./scripts/tri test`:
  - 718 / 718 non-smoke PASS.
  - 198 / 198 yosys smoke PASS, 0 baseline failures.
  - 198 / 198 Icarus smoke PASS, 0 documented baseline failures.
  - 718 / 718 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
  - Gen C / Fixed Point: clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).
- Only deterministic combinational conditions are modeled; non-lowerable side effects in loop conditions are rejected by the predicate.
- `return` is modeled with a sentinel flag in the operational semantics, but the Verilog backend still lowers early `return` via the existing `if (c) begin <then> end else begin <rest> end` rewrite; this remains a boundary if a `return` and a `break`/`continue` interact in the same body.

---

## Next wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W509_2026-07-07.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
