# NOW — Wave Loop 505 in progress (2026-07-07)

**Last updated:** 2026-07-07

---

## Wave Loop 505 — Harden Icarus sequential equivalence boundary (in progress)

- Branch: `wave-loop-505` (to create)
- Issue: #1474 (placeholder — GH_TOKEN unavailable)
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W505_2026-07-07.md`

### Goal

Choose among the three W505 cooperation variants and continue the Icarus
structural-equivalence line after W504.  Likely priorities:

1. Stress the new sequential `if` / `for` boundary with adversarial witnesses
   (Variant A).
2. Model `switch` statements for enum / trit dispatch (Variant B).
3. Model bounded `while` loops and unify the loop invariant (Variant C).

---

## Wave Loop 504 — Extend generic Icarus equivalence theorem to bounded `forLoop` (closed)

- Branch: `wave-loop-504`
- Issue: #1473
- Plan: `.claude/plans/wave-loop-504.md`
- Report: `docs/reports/WAVE_LOOP_504_CLOSEOUT.md`
- Cooperation W505: `docs/reports/FPGA_LOOP_COOPERATION_W505_2026-07-07.md`

### Deliverables

- Defined the **sequential** subset: combinational statements plus bounded
  `forLoop` whose range and body are sequential.
- Proved combinationality implies sequentiality for statements, functions, and
  modules in `Predicate.lean`.
- Aligned loop fuel consumption so each iteration evaluates the body and
  recurses at the smaller fuel in `SemanticsTotal.lean`.
- Generalized `all_equiv` in `Equivalence.lean` to sequential modules and added a
  dedicated `P_forLoop` predicate; proved the `Stmt.forLoop` case.
- Added sequential main theorems `module_value_equiv_proved_sequential` and
  `module_value_equiv_main_sequential`; kept combinational corollaries as
  wrappers.
- Added scratch witness:
  - `specs/scratch/w504_for_sum.t27` — bounded `for` with a parameter `n`.
- Added W504 witness environments/modules in `Lemmas.lean` and lowerability /
  sequential / value-preservation theorems in `Soundness.lean`.

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 259 lowerable specs, 0
  disagreements.
- `./scripts/tri test`:
  - 706 / 706 non-smoke PASS.
  - 186 / 186 yosys smoke PASS, 0 baseline failures.
  - 186 / 186 Icarus smoke PASS, 0 documented baseline failures.
  - 706 / 706 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### Residual boundaries

- `while` and `switch` remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

## Wave Loop 503 — Extend Icarus equivalence proof to sequential constructs (closed)

- Branch: `wave-loop-503`
- Issue: #1472
- Plan: `.claude/plans/wave-loop-503.md`
- Report: `docs/reports/WAVE_LOOP_503_CLOSEOUT.md`
- Cooperation W504: `docs/reports/FPGA_LOOP_COOPERATION_W504_2026-07-12.md`

### Deliverables

- Added `ifThenElse` and bounded `forLoop` to the t27 and shallow-Verilog
  operational semantics.
- Added `ifThenElse` / `forLoop` constructors to `Verilog.lean`.
- Updated `Emitter.lean` to emit real sequential constructs.
- Broadened `Predicate.lean` so `ifThenElse` is combinational when its parts
  are; `forLoop` is lowerable but remains non-combinational.
- Extended `all_equiv` in `Equivalence.lean` with the `ifThenElse` case.
- Added scratch witnesses:
  - `w503_if_return.t27` — conditional return of a numeric literal,
  - `w503_for_accumulator.t27` — bounded `for` summing into a local variable.
- Added W503 witness environments/modules in `Lemmas.lean` and
  lowerability/value-preservation theorems in `Soundness.lean`.

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0
  disagreements.
- `./scripts/tri test`:
  - 705 / 705 non-smoke PASS.
  - 185 / 185 yosys smoke PASS, 0 baseline failures.
  - 185 / 185 Icarus smoke PASS, 0 documented baseline failures.
  - 705 / 705 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### Residual boundaries

- Bounded `forLoop` is modeled and lowerable, but not yet covered by the generic
  `module_value_equiv_statement` theorem (closed by W504).
- `while` and `switch` remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

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
- Conditionals and loops remain outside the modeled operational semantics
  (targeted by W503).

---

*φ² + φ⁻² = 3 | TRINITY*
