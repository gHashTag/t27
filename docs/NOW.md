# NOW — Wave Loop 499 in progress / W500 next (2026-07-13)

**Last updated:** 2026-07-13

---

## Wave Loop 499 — Make `module_value_equiv` unconditional for all lowerable modules (Variant A in progress)

- Branch: `wave-loop-499`
- Issue: #1469
- Plan: `.claude/plans/wave-loop-499.md` (to create)
- Cooperation W500: `docs/reports/FPGA_LOOP_COOPERATION_W500_2026-07-13.md` (to create)

### In progress

- Change `emitModuleFuel` to emit every function/test/bench into
  `VModule.functions`.
- Remove `Module.callsResolved` and `Module.callsReachable` assumptions from
  `module_value_equiv_statement`.
- Add adversarial witness with unreachable functions that contain calls.
- Run verification gates:
  - `lake build Trinity.IcarusLowerable.Soundness`
  - `./scripts/tri test --fast`
  - `cargo test -p t27c --bin t27c`

---

## Wave Loop 498 — Complete the generic structural equivalence theorem (closed)

- Branch: `wave-loop-498`
- Issue: #1468
- Plan: `.claude/plans/wave-loop-498.md`
- Report: `docs/reports/WAVE_LOOP_498_CLOSEOUT.md`
- Cooperation W499: `docs/reports/FPGA_LOOP_COOPERATION_W499_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
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

- Fuel-based total evaluator in `SemanticsTotal.lean`.
- Total emitter model in `Emitter.lean` with fuel-threaded `widthOfType` and
  emission functions.
- Forward-simulation invariant in `Equivalence.lean` covering all combinational
  expression and statement forms.
- `module_value_equiv_statement` proved via `module_value_equiv_proved` in
  `Soundness.lean`.
- `native_decide` bridge lemmas connecting total and partial evaluators on the
  W495 witness set.

### Residual boundaries

- The theorem still assumes `Module.hasUniqueFunctionNames`.
- Conditionals and loops remain outside the modeled operational semantics.
- `Expr.typeOf` remains a heuristic helper.
- The local AOS element boundary remains the single documented Icarus baseline.

---

## Wave Loop 497 — Totalize the Icarus-lowerable combinational evaluator and scaffold the generic structural equivalence theorem (Variant A)

- Branch: `wave-loop-497`
- Issue: #1467
- Plan: `.claude/plans/wave-loop-497.md`
- Report: `docs/reports/WAVE_LOOP_497_CLOSEOUT.md`
- Cooperation W498: `docs/reports/FPGA_LOOP_COOPERATION_W498_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green.
  - One remaining `sorry` in `module_value_equiv_statement` in
    `Trinity/IcarusLowerable/Soundness.lean` closed by W498.
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure.
  - 697 / 697 seal matches.
  - 0 Icarus lowerability disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- NMSE seal: unchanged.

---

*φ² + φ⁻² = 3 | TRINITY*
