# NOW — Wave Loop 502 closed / W503 next (2026-07-13)

**Last updated:** 2026-07-13

---

## Wave Loop 502 — Harden Icarus lowerability gate with adversarial non-main witnesses (closed)

- Branch: `wave-loop-502`
- Issue: #1471
- Plan: `.claude/plans/wave-loop-502.md`
- Report: `docs/reports/WAVE_LOOP_502_CLOSEOUT.md`
- Cooperation W503: `docs/reports/FPGA_LOOP_COOPERATION_W503_2026-07-13.md`

### Deliverables

- Added four adversarial scratch witnesses that stress non-`main` entry points:
  `w502_non_main_called_from_emitted.t27`, `w502_non_main_chain_leaf.t27`,
  `w502_non_main_helper_struct_param.t27`, `w502_multiple_non_main_entries.t27`.
- Proved lowerability and value preservation for a non-`main` function in each
  witness, including a helper that takes a scalar struct parameter.
- Generalized `module_value_equiv_proved` / `module_value_equiv_statement` to
  accept an arbitrary `args : List Value`, extending W501's entry-point
  generalization to functions with parameters.
- Kept the Icarus smoke gate at 0 documented baseline failures.

---

## Wave Loop 501 — Generalize `module_value_equiv` beyond `main` (closed)

- Branch: `wave-loop-501`
- Issue: #1470
- Plan: `.claude/plans/wave-loop-501.md`
- Report: `docs/reports/WAVE_LOOP_501_CLOSEOUT.md`
- Cooperation W502: `docs/reports/FPGA_LOOP_COOPERATION_W502_2026-07-13.md`

### Deliverables

- Parameterized `module_value_equiv_proved` over any emitted function name
  `fnName` and function `fn`.
- Kept `module_value_equiv_main` as a convenience corollary.
- Added a non-main witness (`w501_non_main_entry_function.t27`) and a Lean
  `w501_non_main_entry_value_equiv` theorem that applies the generalized
  statement to `get_y`.
- Cleaned up the stale Icarus baseline file so the smoke gate reports 0
  documented baseline failures.

---

## Wave Loop 500 — Close the last documented Icarus baseline (closed)

- Branch: `wave-loop-500`
- Issue: #1458
- Plan: `.claude/plans/wave-loop-500.md`
- Report: `docs/reports/WAVE_LOOP_500_CLOSEOUT.md`
- Cooperation W501: `docs/reports/FPGA_LOOP_COOPERATION_W501_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs.
- `./scripts/tri test`:
  - 698 / 698 non-smoke PASS.
  - 178 / 178 yosys smoke PASS, 0 baseline failures.
  - 178 / 178 Icarus smoke PASS, 0 documented baseline failures.
  - 698 / 698 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### Deliverables

- `gen_verilog_pack_struct_array_element` supports local register-mode AOS
  element re-packing with sized zero fallback.
- Renamed witness `w493_local_aos_element_field_lowerable.t27`.
- Icarus smoke gate reached 178 / 178 PASS with zero baselines.

### Residual boundaries

- The theorem still assumed `main` is not host-only (closed by W501).
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
