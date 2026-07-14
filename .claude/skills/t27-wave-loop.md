---
description: Standing Wave Loop charter for t27 — investigate weak points, research papers, plan, implement, report, and propose next-wave cooperation variants.
parameters:
  - name: wave
    type: string
    description: Wave number (e.g. "526")
  - name: issue
    type: string
    description: GitHub issue number for the wave
---

# t27 Wave Loop Skill

This skill encodes the standing Wave Loop charter repeated across t27 sessions:

> investigate weak points, research relevant scientific literature, create a
> decomposed plan, implement the recommended variant, write a closeout report,
> propose three cooperation variants for the next Wave Loop, and save skills
> and experience at the end.

Procedure:

1. **Investigate weak points** — audit the current branch, recent test
   baselines, and unlanded process-debt needles.
2. **Research scientific literature** — find 2–4 papers or canonical models
   relevant to the needle (e.g. Vericert, CompCert, Vitis HLS AoS/SoA rules,
   Roofline).
3. **Create a decomposed plan** — write `.claude/plans/wave-loop-{N}.md` with
   three variants (A recommended, B implementation-heavy, C process/tooling).
4. **Implement the recommended variant** — make the smallest reviewable diff that
   advances the needle, update `FROZEN_HASH` if `bootstrap/src/compiler.rs`
   changes, and run the relevant validation gates.
5. **Write the closeout report** — `docs/reports/WAVE_LOOP_{N}_CLOSEOUT.md`.
6. **Write cooperation variants** —
   `docs/reports/FPGA_LOOP_COOPERATION_W{N+1}_YYYY-MM-DD.md`.
7. **Update issue tracking** — `.trinity/current-issue.md` for the next wave.
8. **Save learnings** — append to `.trinity/experience.md` and persistent memory.
9. **Save/update this skill** — keep the charter encoded in
   `.claude/skills/t27-wave-loop.md`.

## Invariants

- Follow L1 TRACEABILITY: every commit must reference an issue with
  `Closes #N`, `Fixes #N`, `Refs #N`, etc.
- Never hand-edit files under `gen/`; change specs and regenerate.
- Update `bootstrap/stage0/FROZEN_HASH` whenever `bootstrap/src/compiler.rs`
  is modified.
- Prefer a clear diagnostic over silently passing smoke tests with broken
  generated code.

## Phase completion marker

When a PHI LOOP phase is complete, include:

```
Phase complete: [phase name]
→ Phase [next phase number]: [next phase name]
```

## Worked example — Wave Loop 530

Wave Loop 530 made the static Icarus-lowerability classifier executable:

- Fixed a latent 2-D packed-vector layout bug in `bootstrap/src/compiler.rs`
  (reverse Verilog concatenation parts so t27 index `[0]` maps to the LSB).
- Added `VerilogCodegen::emit_test_assertions` and
  `Compiler::compile_verilog_for_simulation`.
- Added `t27c icarus-simulate` and the `--icarus-simulate` / `--icarus-lowerable`
  flags to `t27c suite` (exposed via `./scripts/tri test`).
- Added Phase 3d in `bootstrap/src/suite.rs`: compile generated Verilog with
  `iverilog`, run with `vvp`, and compare `$display` output against JSON
  baselines under `.trinity/icarus-baselines/`.
- Scoped the first regression suite to W493–W529 lowerable scratch witnesses
  (`specs/scratch/w5*.t27`) and recorded 10 baselines.
- Resealed 125 specs whose `gen_hash_verilog` changed after the layout fix.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable` 10/10 Icarus PASS,
  0 seal mismatches, 16 pre-existing yosys smoke baselines.

Key learning: a simulation gate catches value-level regressions that static
syntax-only smoke gates miss; it also exposed that unrelated scratch specs must
be kept out of the regression suite by a deliberate whitelist.

## Worked example — Wave Loop 531

Wave Loop 531 extended the Icarus simulation regression suite to primitive arrays:

- Lowered function-local and module-level arrays of primitive scalars as
  unpacked Verilog arrays in `bootstrap/src/compiler.rs`, fixing signed widths
  and variable-index writes that the old packed scalar-reg fallback broke.
- Added W531 helpers for primitive-array detection, access, and initialization.
- Extended `icarus_regression_specs` in `bootstrap/src/suite.rs` to include
  lowerable `w3*` scratch specs alongside the existing `w5*` witnesses.
- Resealed 23 specs whose `gen_hash_verilog` changed after the lowering switch.
- Recorded new/updated Icarus JSON baselines under
  `.trinity/icarus-baselines/`.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable` 24/0 Icarus PASS,
  0 seal mismatches, 16 pre-existing yosys smoke baselines.

Key learning: the same broken array-lowering fallback existed in two places
(`StmtLocal` and `gen_verilog_var`); fixing only one left module-level RAM
witnesses broken. Unpacked arrays are the correct Verilog lowering for primitive
t27 arrays when signed widths or variable indices matter.

## Worked example — Wave Loop 532

Wave Loop 532 extended the packed-vector subset to signed scalar-array struct
fields:

- Added `scalar_field_width`, `scalar_field_is_signed`, `scalar_array_info`,
  `emit_packed_scalar_value`, `emit_packed_struct_field_value`, and
  `emit_packed_array_element_value` in `bootstrap/src/compiler.rs` so that
  scalar-struct fields of the form `[N]i8/i16/i32` are sized and signed correctly.
- Added `try_emit_struct_array_field_element_access` to lower `grid[i][j].data[k]`
  as a single dynamic part-select, scaling the inner index by the inner element
  width.
- Emitted signed negative literals as `-{w}'sd{abs}` inside packed concatenations
  to satisfy Icarus and keep each value at exactly the declared width.
- Allowed colon syntax in on-demand array-literal re-parsing so module-level
  `const` initializers lower correctly.
- Added `is_lowerable_scalar_struct` and `// UNSUPPORTED_ICARUS` markers to keep
  the classifier aligned with the backend for string/enum/float fields.
- Added 7 scratch witnesses (5 positive, 2 negative), resealed the corpus,
  and recorded 5 Icarus JSON baselines.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable` 28/0 Icarus PASS,
  0 seal mismatches, 23 pre-existing yosys smoke baselines unchanged.

Key learning: when adding a new access shape to an existing lowering path, add a
separate helper rather than modifying the old one; otherwise HIR parity and
existing 1-D flattening regress. Sized signed literals are also required inside
packed concatenations — `$signed(-value)` is ambiguous in width and breaks the
layout.

---

*φ² + φ⁻² = 3 | TRINITY*
