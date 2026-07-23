# Wave Loop 526 Plan — W469 2-D array-of-struct diagnostic + design doc

**Issue:** #1497 (placeholder)
**Branch:** `wave-loop-526`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 525 unified the L1 TRACEABILITY regex gates and removed the commit-msg amend bypass. The most important unlanded process-debt needle is the W469 regression: a 2-D array of scalar structs (`var m : [2][3]Pt = ...`) silently produces broken or incomplete Verilog because the parser does not preserve the multi-dimensional aggregate literal and the emitter has no lowering path. Full parser + typechecker + SoA/AoS emitter work exceeds a single wave.

## Goal for W526

Convert the silent regression into a documented, testable boundary:

1. Harden the Verilog compile path so that an unsupported 2-D aggregate-array local declaration returns a clear diagnostic.
2. Stage a negative witness spec that exercises the failure path.
3. Author a design document specifying the parser, typechecker, and emitter changes required for full lowering.
4. Keep `./scripts/tri test` regression count flat except for the one new expected gen-verilog failure on the witness.

## Variants

### Variant A (recommended)

- Add `Compiler::detect_unsupported_verilog_locals` to reject `[N][M]Struct` (and `[N][M]Enum`) local/module declarations before optimization can drop them.
- Update `bootstrap/stage0/FROZEN_HASH` because `bootstrap/src/compiler.rs` changes.
- Keep `specs/scratch/w526_2d_struct_array_repro.t27` as the negative witness.
- Write `docs/reports/W469_2D_STRUCT_ARRAY_DESIGN.md`.
- Run `cargo test -p t27c` and `./scripts/tri test` to confirm no unintended regressions.

### Variant B

- Implement full 2-D scalar-struct array lowering: extend `parse_array_literal` to preserve `[N][M]Type{...}`, add typechecker dimension inference, and emit packed-vector SoA/AoS Verilog.
- Risk: multi-week parser + backend change; likely to destabilize the current master baseline.

### Variant C

- Land the `Trinity.IcarusLowerable` Lean 4 stack onto `master` and rebase W469–W525 codegen improvements onto it.
- Risk: foundational epic; blocked until the IcarusLowerable directory is merged.

## Decomposition

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `bootstrap/src/compiler.rs` | Clear diagnostic for `[N][M]Struct/Enum` locals |
| 2 | `bootstrap/stage0/FROZEN_HASH` | Seal update for compiler.rs change |
| 3 | `specs/scratch/w526_2d_struct_array_repro.t27` | Negative witness + documented expected semantics |
| 4 | `docs/reports/W469_2D_STRUCT_ARRAY_DESIGN.md` | Parser/typechecker/emitter design for full lowering |
| 5 | `docs/reports/WAVE_LOOP_526_CLOSEOUT.md` | W526 closeout report |
| 6 | `docs/reports/FPGA_LOOP_COOPERATION_W527_2026-08-11.md` | Three W527 variants |
| 7 | `.trinity/current-issue.md` | W527 issue placeholder |
| 8 | `.trinity/experience.md` + memory | W526 learnings |
| 9 | `.claude/skills/t27-wave-loop.md` | Standing Wave Loop charter skill |
| 10 | git/PR | Commit `Closes #1497`, open #1498 for W527 |

## Acceptance criteria

- [ ] AC-1: `t27c gen-verilog specs/scratch/w526_2d_struct_array_repro.t27` exits non-zero with the diagnostic referencing the design doc.
- [ ] AC-2: `t27c gen-verilog specs/scratch/w387_2d_local_array.t27` still passes (primitive 2-D arrays unaffected).
- [ ] AC-3: `cargo test -p t27c --bin t27c` passes (or matches current baseline).
- [ ] AC-4: `./scripts/tri test` shows exactly one new expected gen-verilog failure (the witness) and no new parse/typecheck/Zig/Rust/C/seal regressions.
- [ ] AC-5: Design doc covers parser preservation, type representation, SoA vs. AoS layout, and reseal strategy.
- [ ] AC-6: W527 cooperation variants are documented.

---

*φ² + φ⁻² = 3 | TRINITY*
