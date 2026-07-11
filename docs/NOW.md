# NOW — Wave Loop 496 closed / Wave Loop 497 next (2026-07-13)

**Last updated:** 2026-07-13

---

## Wave Loop 497 — Next wave (Variant A recommended)

- Branch: `wave-loop-497` (to create)
- Issue: #1467 (to create)
- PR: (to open after close-out)
- Cooperation W497: `docs/reports/FPGA_LOOP_COOPERATION_W497_2026-07-13.md`

### Not started

- Select one of the three W497 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W497_2026-07-13.md`.
- Recommended default: **Variant A** — totalize the Icarus-lowerable
  combinational evaluator with explicit fuel, prove the generic structural
  equivalence theorem, and bridge the old partial evaluator to the new total
  evaluator on the W495 witness set with `native_decide`.

---

## Wave Loop 496 — Generic structural equivalence theorem for the Icarus-lowerable scalar subset (Variant A)

- Branch: `wave-loop-496`
- Issue: #1466
- Plan: `.claude/plans/wave-loop-496.md`
- Report: `docs/reports/WAVE_LOOP_496_CLOSEOUT.md`
- Cooperation W497: `docs/reports/FPGA_LOOP_COOPERATION_W497_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.Ast Trinity.IcarusLowerable.AstInduction
  Trinity.IcarusLowerable.Predicate Trinity.IcarusLowerable.Verilog
  Trinity.IcarusLowerable.Emitter Trinity.IcarusLowerable.Lemmas
  Trinity.IcarusLowerable.Semantics Trinity.IcarusLowerable.Soundness
  Trinity.IcarusLowerable.Completeness`: green.
  - One warning: `Trinity/IcarusLowerable/Soundness.lean:120:8: declaration
    uses 'sorry'` (`module_value_equiv_statement`).
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus lowerability disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- NMSE seal: unchanged (no `bootstrap/src/compiler.rs` change).

### Deliverables

- Pure-combinational subset predicates in `Predicate.lean`:
  `Expr.isCombinational`, `Stmt.isCombinational`, `Function.isCombinational`,
  `Module.isCombinational`.
- Custom nested induction principle for `Expr` in `AstInduction.lean`.
- Valuation equivalence invariant `Valuation.equiv` in `Semantics.lean`.
- `module_value_equiv_statement` retained as a stated goal with an honest
  residual `sorry`; the blocker is documented as the `partial` evaluator
  opacity, and W497 will totalize the semantics to close it.

### Residual boundaries

- `module_value_equiv_statement` not yet proved; requires totalization of the
  combinational evaluator.
- Conditionals and loops remain outside the modeled operational semantics.
- `Expr.typeOf` remains a heuristic helper.
- The local AOS element boundary remains the single documented Icarus baseline.

---

## Wave Loop 495 — Semantic equivalence for function calls and W493 witnesses (Variant A)

- Branch: `wave-loop-495`
- Issue: #1465
- Plan: `.claude/plans/wave-loop-495.md`
- Report: `docs/reports/WAVE_LOOP_495_CLOSEOUT.md`
- Cooperation W496: `docs/reports/FPGA_LOOP_COOPERATION_W496_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.*`: green.
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- NMSE seal: unchanged.

---

*φ² + φ⁻² = 3 | TRINITY*
