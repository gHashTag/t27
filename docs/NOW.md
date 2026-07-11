# NOW — Wave Loop 497 closed / Wave Loop 498 next (2026-07-13)

**Last updated:** 2026-07-13

---

## Wave Loop 498 — Next wave (Variant A recommended)

- Branch: `wave-loop-498` (to create)
- Issue: #1468 (to create)
- PR: (to open after close-out)
- Cooperation W498: `docs/reports/FPGA_LOOP_COOPERATION_W498_2026-07-13.md`

### Not started

- Select one of the three W498 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W498_2026-07-13.md`.
- Recommended default: **Variant A (scoped)** — complete the forward-simulation
  proof of `module_value_equiv_statement` for the pure combinational subset,
  then relax the reachability/closure assumptions if time allows.  Control-flow
  semantics extension stays out of scope unless the proof uncovers an emitter gap.

---

## Wave Loop 497 — Totalize the Icarus-lowerable combinational evaluator and prove the generic structural equivalence theorem (Variant A)

- Branch: `wave-loop-497`
- Issue: #1467
- Plan: `.claude/plans/wave-loop-497.md`
- Report: `docs/reports/WAVE_LOOP_497_CLOSEOUT.md`
- Cooperation W498: `docs/reports/FPGA_LOOP_COOPERATION_W498_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green.
  - One remaining `sorry` in `module_value_equiv_statement` in
    `Trinity/IcarusLowerable/Soundness.lean`; all other IcarusLowerable modules
    are green.  The full `lake build` has unrelated failures in
    `Trinity.H4Lagrangian` and `Trinity.NeutrinoMasses` (not touched).
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
- Aligned default declaration semantics and tightened combinational subset
  predicate.
- Fuel-based, proof-transparent lowerability / combinationality / call-closure
  predicates in `Predicate.lean` (`Expr.isLowerableFuel`, `Stmt.isCombinationalFuel`,
  `Module.callsResolved`, `Module.callsReachable`, etc.).
- Generic `module_value_equiv_statement` stated and scaffolded for lowerable,
  combinational, call-closed modules; the forward-simulation proof is the only
  remaining `sorry` in IcarusLowerable.
- `native_decide` bridge lemmas connecting total and partial evaluators on the
  W495 witness set.

### Residual boundaries

- Generic theorem still assumes call closure and reachable `main`.
- Conditionals and loops remain outside the modeled operational semantics.
- `Expr.typeOf` remains a heuristic helper.
- The local AOS element boundary remains the single documented Icarus baseline.

---

## Wave Loop 496 — Generic structural equivalence theorem for the Icarus-lowerable scalar subset (Variant A)

- Branch: `wave-loop-496`
- Issue: #1466
- Plan: `.claude/plans/wave-loop-496.md`
- Report: `docs/reports/WAVE_LOOP_496_CLOSEOUT.md`
- Cooperation W497: `docs/reports/FPGA_LOOP_COOPERATION_W497_2026-07-13.md`

### Verification

- `lake build` IcarusLowerable modules green with one `sorry` in
  `Soundness.lean`.
- `./scripts/tri test --fast`: 697/697 non-smoke PASS, 177/177 yosys smoke PASS,
  176/177 Icarus smoke PASS (1 documented baseline), 697/697 seal matches,
  0 Icarus disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- NMSE seal: unchanged.

---

*φ² + φ⁻² = 3 | TRINITY*
