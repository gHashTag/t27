# NOW — Wave Loop 500 in progress / W501 next (2026-07-13)

**Last updated:** 2026-07-13

---

## Wave Loop 500 — Close the last documented Icarus baseline (Variant A in progress)

- Branch: `wave-loop-500`
- Issue: #1458
- Plan: `.claude/plans/wave-loop-500.md`
- Cooperation W501: `docs/reports/FPGA_LOOP_COOPERATION_W501_2026-07-13.md` (to create)

### In progress

- Detect register-mode local arrays of structs in
  `gen_verilog_pack_struct_array_element`.
- Re-pack indexed local register-mode AOS elements as packed vectors using
  per-element per-field registers (`base_idx_flatfield`).
- Emit a sized zero fallback (`{N{1'b0}}`) in the variable-index priority mux.
- Rename the adversarial witness to `w493_local_aos_element_field_lowerable.t27`
  and reseal.
- Run verification gates:
  - `lake build Trinity.IcarusLowerable.Soundness`
  - `./scripts/tri verify --lean-lowerable`
  - `./scripts/tri test`
  - `cargo test -p t27c --bin t27c`

---

## Wave Loop 499 — Make `module_value_equiv` unconditional for all lowerable modules (closed)

- Branch: `wave-loop-499`
- Issue: #1459
- Plan: `.claude/plans/wave-loop-499.md`
- Report: `docs/reports/WAVE_LOOP_499_CLOSEOUT.md`
- Cooperation W500: `docs/reports/FPGA_LOOP_COOPERATION_W500_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed.
- `./scripts/tri test`:
  - 698 / 698 non-smoke PASS.
  - 178 / 178 yosys smoke PASS, 0 baseline failures.
  - 177 / 178 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 698 / 698 seal matches.
  - FPGA board-less smoke gate / replay: OK.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### Deliverables

- `emitModuleFuel` emits every non-host-only function as a `VFunction`.
- `Module.callsResolved` / `Module.callsReachable` removed from the generic
  theorem assumptions.
- `Module.hasUniqueFunctionNames` and `Module.callContext` added as
  well-formedness invariants.
- New adversarial witness `w499_unconditional_function_emission.t27`.

### Residual boundaries

- The theorem still assumes `main` is not host-only.
- The local AOS element boundary was the single documented Icarus baseline
  (closed by W500).
- Conditionals and loops remain outside the modeled operational semantics.

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
