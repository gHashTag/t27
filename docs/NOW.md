# NOW — Wave Loop 513 planned (2026-07-07)

**Last updated:** 2026-07-07

---

## Wave Loop 513 — Function-local packed arrays-of-structs (planned)

- Branch: `wave-loop-513` (to create from `wave-loop-512`)
- Issue: #1482 (placeholder — GH_TOKEN unavailable)
- Plan: `.claude/plans/wave-loop-513.md` (to create)
- Report: `docs/reports/WAVE_LOOP_513_CLOSEOUT.md` (to create)
- Cooperation W514: `docs/reports/FPGA_LOOP_COOPERATION_W514_2026-07-07.md` (to create)

### Goal

Execute **Variant A** from the W513 cooperation plan: extend the W512
packed-vector lowering for arrays of scalar structs with fixed-size scalar array
fields from bench-local and module-level storage into **function-local
declarations**.

### Deliverables (planned)

- Extend `gen_verilog_local_decl_hoisted` / `gen_verilog_local_assign` in
  `bootstrap/src/compiler.rs` to emit packed-vector memories for function-local
  arrays whose element type is a lowerable scalar struct.
- Wire packed-AOS read/write/argument/return paths to function-local names,
  including any function-local prefix.
- Add scratch witnesses:
  - `w513_local_aos_read.t27`
  - `w513_local_aos_write.t27`
  - `w513_local_aos_return.t27`
- Add W513 environments/modules in
  `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` and value-preservation
  theorems in `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`.
- Reseal affected specs after the Verilog layout change.

---

## Wave Loop 512 — Arrays of structs with array-typed element fields (closed)

- Branch: `wave-loop-512`
- Issue: #1481 (placeholder — GH_TOKEN unavailable)
- Plan: `.claude/plans/wave-loop-512.md`
- Report: `docs/reports/WAVE_LOOP_512_CLOSEOUT.md`
- Cooperation W513: `docs/reports/FPGA_LOOP_COOPERATION_W513_2026-07-07.md`

### Goal

Execute **Variant A** from the W512 cooperation plan: extend the W509–W511
packed-vector lowering for scalar structs with fixed-size scalar array fields
from single instances (local/param/return/module) out to arrays of such structs.

### Deliverables

- Added `local_packed_struct_array_*` and `module_packed_struct_array_*` tracking
  maps plus packed-AOS helpers in `bootstrap/src/compiler.rs`.
- Emitted bench-local and module-level arrays of lowerable scalar structs as
  unpacked memories of packed vectors, reusing the MSB-first field layout.
- Extended packed read/write paths to resolve `aos[i].field[j]` through the
  outer memory address and inner packed-vector slice.
- Added call-site argument packing so bench-local / module packed AOS can be
  passed into functions.
- Added scratch witnesses:
  - `specs/scratch/w512_aos_array_field_read.t27`
  - `specs/scratch/w512_aos_array_field_write.t27`
  - `specs/scratch/w512_aos_array_field_return.t27`
- Added W512 environments/modules in
  `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` and value-preservation
  theorems in `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`.
- Resealed affected specs after the Verilog layout change.

### Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 252 lowerable specs, 0
  disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- `./scripts/tri test --icarus-lowerable`: acceptable — 730/730
  parse+typecheck+gen PASS, 208/210 yosys smoke PASS, 209/210 Icarus smoke PASS,
  730/730 seal matches, Icarus lowerability 0 disagreements. The 3 smoke failures
  are documented W508 early-exit baselines.

### Residual boundaries

- Function-local packed AOS declarations are not yet lowered.
- ram_style / ROM-style pragmas are not yet applied to packed structs / AOS.
- The generic sequential theorem still accepts only identifier LHS assignments
  and initialized module-level declarations.
- The W508 break/continue/return early-exit interaction remains a documented
  baseline on this branch.

---

*φ² + φ⁻² = 3 | TRINITY*
