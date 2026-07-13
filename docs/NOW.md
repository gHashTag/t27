# NOW — Wave Loop 516 planned (2026-07-07)

**Last updated:** 2026-07-07

---

## Wave Loop 516 — Cooperation variants (planned)

- Branch: `wave-loop-516` (to create from `wave-loop-515`)
- Issue: #1485 (placeholder — GH_TOKEN unavailable)
- Cooperation W516: `docs/reports/FPGA_LOOP_COOPERATION_W516_2026-07-07.md`
- Plan: select Variant A, B, or C and implement in the next loop.

### Goal

Pick one of the three W516 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W516_2026-07-07.md`:

- **Variant A (recommended):** enable whole-array-field reads from packed scalar
  structs and packed arrays-of-structs.
- **Variant B:** clear the remaining W508 `break`/`continue` smoke baselines.
- **Variant C:** add packed scalar struct equality / inequality operators in the
  Icarus-lowerable subset.

### Residual boundaries from W515

- Whole-array-field reads from packed structs / AOS are not yet lowered.
- The W508 early-exit yosys/Icarus baselines remain.
- Packed scalar struct equality operators are not yet lowerable.

---

## Wave Loop 515 — Function-local packed scalar struct copy initializers (closed)

- Branch: `wave-loop-515`
- Issue: #1484 (placeholder — GH_TOKEN unavailable)
- Plan: `.claude/plans/wave-loop-515.md`
- Report: `docs/reports/WAVE_LOOP_515_CLOSEOUT.md`
- Cooperation W516: `docs/reports/FPGA_LOOP_COOPERATION_W516_2026-07-07.md`

### Goal

Execute a revised **Variant C** from the W515 cooperation plan: remove the
unlowered boundary that function-local packed scalar struct variables cannot be
initialized by copying another packed struct value.

### Deliverables

- Refined `copy_propagate` in `bootstrap/src/compiler.rs` to preserve `var`
  declarations of struct-like type, fixing unresolved field access when a
  copied packed struct local is later mutated.
- Added scratch witnesses:
  - `specs/scratch/w515_local_packed_struct_copy.t27`
  - `specs/scratch/w515_module_to_local_packed_struct_copy.t27`
  - `specs/scratch/w515_local_packed_struct_return_copy.t27`
- Added Lean environments and value-preservation theorems in
  `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` and `Soundness.lean`.
- Resealed 31 existing specs affected by the optimizer refinement and saved
  seals for the three new W515 scratch specs.

### Verification

- `cargo test -p t27c --bin t27c`: **1525 / 0 / 2**.
- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed, 252 lowerable specs, 0
  disagreements.
- `./scripts/tri test --icarus-lowerable --fast`: acceptable — 739/739
  parse+typecheck+gen PASS, 0 seal mismatches, Icarus lowerability 0
  disagreements. The 5 smoke failures match the updated baseline (2 yosys W508
  break baselines + 3 Icarus W508/function-local pragma baselines).

### Residual boundaries

- Whole-array-field reads from packed structs / AOS are not yet lowered.
- The W508 early-exit yosys/Icarus baselines remain.
- Scalar and array `var` copy propagation still aliases the source for
  non-struct types (documented semantic quirk).

---

*φ² + φ⁻² = 3 | TRINITY*
