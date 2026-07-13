# Wave Loop 508 — Decomposed Plan

**Issue:** #1477 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-508`  
**Variant:** A — model `break` / `continue` in bounded loops  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

- `break` and `continue` are the last major procedural control-flow constructs
  absent from the Icarus-lowerable operational semantics.
- The Verilog backend currently emits `break;` / `continue;` directly, but the
  Lean model has no corresponding statement constructors and no flag-threaded
  statement-list semantics.
- Early exit changes the shape of `evalStmtsTotal` from a pure `Option Valuation`
  to a pair `(Option Valuation, exit_flag)`, affecting every statement-list proof.

## 2. Scientific / engineering anchors

- **CompCert / Clight** — fuel-based big-step semantics with loop-control
  statements; early exit is encoded by returning a continuation outcome.
- **CakeML** — clocked evaluation with `Break` / `Continue` / `Return` outcomes.
- **SystemVerilog LRM** — procedural `break` / `continue` inside loops.
- **Icarus Verilog** — smoke validation of generated `break` / `continue`.

## 3. Implementation phases

### Phase 1 — Model
- Add `Stmt.break` and `Stmt.continue` to `Ast.lean`.
- Add `VStmt.break` and `VStmt.contounter` to `Verilog.lean`.

### Phase 2 — Semantics
- Change total statement-list evaluation to return a loop-control outcome:
  `Ok Valuation | Break | Continue | Return Value`.
- Update `evalStmtTotal` cases for `break`, `continue`, `return_`, and statement
  lists to propagate the outcome.
- Keep the partial evaluator compiling with catch-all cases in `Semantics.lean`.

### Phase 3 — Predicate
- Add `break`/`continue` to `isLowerableFuel`, `isCombinationalFuel`, and
  `isSequential'`.
- Add a loop-context well-formedness predicate: `break`/`continue` only lowerable
  inside a loop body.

### Phase 4 — Emitter
- Emit `Stmt.break` as `VStmt.break` and `Stmt.continue` as `VStmt.continue`.

### Phase 5 — Equivalence
- Extend the forward-simulation relation to include an outcome flag.
- Prove the `break` and `continue` cases in `all_equiv`.

### Phase 6 — Witnesses
- Add scratch specs:
  - `w508_break_search.t27`
  - `w508_continue_sum.t27`
  - `w508_break_nested.t27`
- Add Lean environments/modules in `Lemmas.lean`.
- Prove lowerability, sequentiality, and value preservation in `Soundness.lean`.

### Phase 7 — Verification
- `lake build Trinity.IcarusLowerable.Soundness`
- `./scripts/tri verify --lean-lowerable`
- `./scripts/tri test`
- `cargo test -p t27c --bin t27c`

### Phase 8 — Close-out
- Write `docs/reports/WAVE_LOOP_508_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W509_2026-07-07.md`.
- Update `.trinity/current-issue.md`, `docs/NOW.md`, and persistent memory.
- Commit and create `wave-loop-509` branch.

## 4. Acceptance criteria

- `lake build Trinity.IcarusLowerable.Soundness` succeeds with zero `sorry`.
- All three W508 scratch specs pass the Icarus lowerability classifier and smoke.
- `./scripts/tri test` is green with 0 baseline failures.
- At least one witness value-preservation theorem uses
  `module_value_equiv_proved_sequential`.

## 5. Residual boundaries

- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).
- Only deterministic combinational loop conditions are modeled.

---

*φ² + φ⁻² = 3 | TRINITY*
