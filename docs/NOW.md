# NOW — Wave Loop 495 closed / Wave Loop 496 next (2026-07-13)

**Last updated:** 2026-07-13

---

## Wave Loop 496 — Next wave (Variant A recommended)

- Branch: `wave-loop-496` (to create)
- Issue: #1466 (to create)
- PR: (to open after close-out)
- Cooperation W496: `docs/reports/FPGA_LOOP_COOPERATION_W496_2026-07-13.md`

### Not started

- Select one of the three W496 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W496_2026-07-13.md`.
- Recommended default: **Variant A** — prove the generic structural
  equivalence theorem for the Icarus-lowerable scalar subset, removing the
  remaining `sorry` in `module_value_equiv_statement`.

---

## Wave Loop 495 — Semantic equivalence for function calls and W493 witnesses (Variant A)

- Branch: `wave-loop-495`
- Issue: #1465
- Plan: `.claude/plans/wave-loop-495.md`
- Report: `docs/reports/WAVE_LOOP_495_CLOSEOUT.md`
- Cooperation W496: `docs/reports/FPGA_LOOP_COOPERATION_W496_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.Ast Trinity.IcarusLowerable.Predicate
  Trinity.IcarusLowerable.Verilog Trinity.IcarusLowerable.Emitter
  Trinity.IcarusLowerable.Lemmas Trinity.IcarusLowerable.Semantics
  Trinity.IcarusLowerable.Soundness Trinity.IcarusLowerable.Completeness`: green.
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- NMSE seal: unchanged (no `bootstrap/src/compiler.rs` change).

### Deliverables

- Shallow Verilog AST extended with `VFunction` definitions.
- Emitter model emits t27 functions as Verilog functions and derives index
  element widths from the base type.
- t27 and Verilog evaluators inline function bodies; `evalVModule` runs a
  named function after module-level items.
- Four W493 positive witnesses modeled in Lean and proved with `native_decide`:
  - `w493_nested_struct_field_from_identifier_lowerable`
  - `w493_local_scalar_struct_field_lowerable`
  - `w493_module_scalar_struct_field_lowerable`
  - `w493_module_aos_element_field_lowerable`
- Generic `module_value_equiv_statement` stated (full structural proof left as
  future work for W496).

---

## Wave Loop 494 — Semantic equivalence for the Icarus-lowerable scalar subset (Variant A)

- Branch: `wave-loop-494`
- Issue: #1464 (closed)
- Plan: `.claude/plans/wave-loop-494.md`
- Report: `docs/reports/WAVE_LOOP_494_CLOSEOUT.md`
- Cooperation W495: `docs/reports/FPGA_LOOP_COOPERATION_W495_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.*`: green.
- `./scripts/tri test --fast --icarus-lowerable`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- NMSE seal: unchanged (no `bootstrap/src/compiler.rs` change).

---

*φ² + φ⁻² = 3 | TRINITY*
