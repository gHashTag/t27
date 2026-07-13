# Wave Loop 508 — Model `break` / `continue` in bounded loops

**Issue:** #1477 (placeholder — GitHub token still failing)  
**Branch:** `wave-loop-508`  
**Variant:** A — `break` / `continue` in bounded loops (default from `docs/reports/FPGA_LOOP_COOPERATION_W508_2026-07-07.md`)  
**Status:** setup  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Execute **Variant A** from the W508 cooperation plan: add `break` and `continue` control flow to the Icarus-lowerable operational semantics, the shallow Verilog model, the emitter, the lowerability/sequential predicate, and the generic equivalence theorem. Target at least one scratch witness that passes both the classifier and Icarus smoke and has a value-preservation theorem proved via `module_value_equiv_proved_sequential`.

---

## Scope

1. Review `docs/reports/WAVE_LOOP_507_CLOSEOUT.md` and `docs/reports/FPGA_LOOP_COOPERATION_W508_2026-07-07.md`.
2. Extend `proofs/lean4/Trinity/IcarusLowerable/Ast.lean` with `Stmt.break` and `Stmt.continue`.
3. Extend `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean` with `VStmt.break` and `VStmt.continue` (or equivalent guard encoding).
4. Extend `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean` with total evaluators that thread an early-exit flag through statement-list evaluation.
5. Extend `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` so `break`/`continue` are lowerable only inside a loop body.
6. Extend `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean` to emit early-exit guards or procedural `break`/`continue`.
7. Extend `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean` with a flag-threaded forward-simulation invariant and prove the `break`/`continue` cases in `all_equiv`.
8. Add scratch witnesses:
   - `w508_break_search.t27` — `while` loop that exits early on a match,
   - `w508_continue_sum.t27` — `for` loop that skips odd indices with `continue`,
   - `w508_break_nested.t27` — `break` inside a nested `while` inside a `for`.
9. Add environments/modules in `Lemmas.lean` and lowerability/sequentiality/value-preservation theorems in `Soundness.lean`.
10. Run `./scripts/tri test`, `./scripts/tri verify --lean-lowerable`, and `lake build Trinity.IcarusLowerable.Soundness`.

---

## Residual boundaries from W507

- `break` and `continue` inside loops remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).
- Only deterministic combinational conditions are modeled in loops.

---

*φ² + φ⁻² = 3 | TRINITY*
