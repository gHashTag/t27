# NOW — Wave Loop 506 in progress (2026-07-07)

**Last updated:** 2026-07-07

---

## Wave Loop 506 — Model `switch` statements for enum / trit dispatch (in progress)

- Branch: `wave-loop-506` (created)
- Issue: #1475 (placeholder — GH_TOKEN unavailable)
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W506_2026-07-07.md`

### Goal

Execute Variant B from the W506 cooperation plan: extend the Icarus-lowerable
operational semantics, shallow Verilog model, emitter, predicate, and generic
equivalence theorem to support `switch` statements for enum and trit dispatch.

---

## Wave Loop 505 — Harden Icarus sequential equivalence boundary (closed)

- Branch: `wave-loop-505`
- Issue: #1474
- Plan: `.claude/plans/wave-loop-505.md`
- Report: `docs/reports/WAVE_LOOP_505_CLOSEOUT.md`
- Cooperation W506: `docs/reports/FPGA_LOOP_COOPERATION_W506_2026-07-07.md`

### Deliverables

- Added five adversarial scratch witnesses:
  - `specs/scratch/w505_nested_if.t27` — nested `ifThenElse` with four return arms.
  - `specs/scratch/w505_if_in_for.t27` — conditional update inside a bounded `forLoop`.
  - `specs/scratch/w505_for_var_range.t27` — bounded `for` whose range is a parameter.
  - `specs/scratch/w505_for_return.t27` — return value computed by a bounded `for`.
  - `specs/scratch/w505_for_local_var_init.t27` — local variable declared inside the loop body.
- Added W505 witness environments/modules in `Lemmas.lean` and
  lowerability / sequentiality / value-preservation theorems in `Soundness.lean`.
- Each value-preservation theorem applies the generic sequential theorem
  `module_value_equiv_proved_sequential`.

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 253 lowerable specs, 0
  disagreements.
- `./scripts/tri test`:
  - 711 / 711 non-smoke PASS.
  - 191 / 191 yosys smoke PASS, 0 baseline failures.
  - 191 / 191 Icarus smoke PASS, 0 documented baseline failures.
  - 711 / 711 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### Residual boundaries

- `while` and `switch` remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

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

*φ² + φ⁻² = 3 | TRINITY*
