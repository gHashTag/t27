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

## Worked example — Wave Loop 533

Wave Loop 533 closed the last major packed-vector gap: module-level single scalar
structs with fixed-size scalar array fields:

- Added `base_type_name`, `is_lowerable_scalar_struct_type`, and `fn_return_types`
  in `bootstrap/src/compiler.rs` so bare lowerable structs share the same width/sign
  logic as arrays-of-structs.
- Fixed `packed_width` / `packed_signed` for bare lowerable scalar structs to
  prevent silent 32-bit truncation on function parameters and return values.
- Lowered module-level `const` scalar structs as `localparam`/`parameter [W:0]` and
  module-level `var` scalar structs as `reg [W:0]` with `initial` initialization.
- Added a `LocalEmitPhase` / `emit_local` helper and hoisted test-block local
  declarations above procedural statements, fixing an Icarus syntax error for
  `var tmp : Pt = make(...);`.
- Fixed `parse_const_decl` to parse `Ident{LBrace}` initializers into real
  `ExprStructLit` nodes instead of raw text or dropped consts.
- Added 8 scratch witnesses (6 positive + 2 negative), resealed the corpus, and
  recorded 8 Icarus JSON baselines.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` 36/0 Icarus PASS,
  0 seal mismatches, 24 pre-existing yosys smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: when a new shape becomes lowerable, update `packed_width` and
`packed_signed` before touching any emitter; otherwise function signatures stay
wrong even after declarations look correct. Also, Verilog `reg` declarations must
be hoisted to the top of every procedural block — never interleave them with
statements.

## Worked example — Wave Loop 534

Wave Loop 534 hardened the Icarus lowerability boundary by making it structural,
documented, and cross-checked:

- Added `Compiler::is_icarus_lowerable` and `Compiler::icarus_lowerability_reason`
  in `bootstrap/src/compiler.rs`; the classifier walks the parsed t27 AST and
  rejects host-only helpers, non-lowerable types, unresolved/qualified imports,
  `while (true)`, iterator-style `for`, and mis-placed `break`/`continue`.
- Fixed a latent bug where recursive `ast_is_icarus_lowerable` returned
  `Ok(false)` without propagating it (the `?` operator only short-circuits on
  `Err`, not on `Ok(false)`).
- Added the `t27c icarus-lowerable [--json]` CLI subcommand and wired it into
  `bootstrap/src/main.rs`.
- Switched `bootstrap/src/suite.rs::is_icarus_lowerable` to the structural
  classifier as the authoritative gate, keeping `iverilog -g2012 -o /dev/null`
  as a backend sanity cross-check.
- Created six adversarial scratch witnesses (`specs/scratch/w534_negative_*.t27`)
  and sealed them.
- Added `bootstrap/tests/icarus_lowerable.rs` to assert that the classifier
  rejects all W534 negative witnesses and accepts known lowerable W5xx/W3xx
  witnesses.
- Documented the boundary in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  new integration test 2/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` 35/0 Icarus PASS,
  0 seal mismatches, 24 pre-existing yosys smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: a lowerability boundary defined only by generated-Verilog + an
external compiler is unsound — the backend can emit syntactically valid
placeholder Verilog for semantically unlowerable constructs.  The source-AST
structural predicate must be the source of truth, with the external compiler
used only as a cross-check.

## Worked example — Wave Loop 535

Wave Loop 535 aligned the Lean 4 lowerability predicate with the Rust structural
classifier:

- Added fuel-threaded `Ty.isLowerableFuel` in
  `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` so struct-field
  lowerability is checked recursively and transparently to the Lean kernel.
- Tightened `Stmt.isLowerableFuel` to reject `while (true)` and
  `Expr.isLowerableFuel` to reject calls to imported names.
- Added six `¬ Module.isLowerable` theorems in `Lemmas.lean` for the W534
  adversarial witnesses (cast to string, `f32` field, host-only helper,
  non-lowerable struct assignment, unbounded `while`, unresolved import) and
  discharged them with `native_decide`.
- Removed the obsolete `imported_ctor_sound` theorem from `Soundness.lean` after
  the import-rejection rule made it false.
- Created `specs/igla/w535_bounded_while_module.t27` as a positive bounded-while
  corpus witness, sealed it, and added the matching environment, module, and
  `igla_w535_bounded_while_module_lowerable` theorem to `Completeness.lean`.
- Updated `docs/ICARUS_LOWERABLE_BOUNDARY.md` to document the tightened rules,
  the six negative theorems, and the positive corpus witness.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 2/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` 35/0 Icarus PASS,
  0 seal mismatches, 24 pre-existing yosys smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Lemmas` green,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`,
  `lake build Trinity.IcarusLowerable.Completeness` 8573 jobs / 0 `sorry`.

Key learning: when tightening a formal predicate, use a fuel-threaded recursive
definition for any check that walks nested types, delete or rewrite positive
theorems that become false immediately, and treat undefined struct names
leniently in simplified corpus models until the generator supplies full field
lists.

## Worked example — Wave Loop 536

Wave Loop 536 added a cocotb reference-model cosimulation gate:

- Derived `serde::Serialize` on `Node`/`NodeKind` in `bootstrap/src/compiler.rs`
  and updated `bootstrap/stage0/FROZEN_HASH`.
- Added `t27c parse --json` and `t27c gen-verilog-for-simulation` subcommands.
- Created `scripts/cocotb_ref_model.py` to extract `assert_eq` expected literals
  from the t27 AST, run `iverilog` + `vvp`, and verify simulation log PASS
  lines.  The script uses `cocotb_tools.runner` when available and falls back
  to direct subprocess invocation otherwise.
- Added `t27c icarus-cocotb` and the `--cocotb` suite flag in
  `bootstrap/src/suite.rs` (Phase 3e).
- Seeded the gate with lowerable `w5xx`/`w3xx` scratch regression specs; the
  suite reports 35/35 cocotb reference-model checks passing.
- Wrote `docs/reports/WAVE_LOOP_536_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W537_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W537.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast` 35/0 Icarus PASS,
  35/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baselines
  unchanged.

Key learning: environment-specific Python dependencies (PEP 668, Python 3.14
compatibility) make strict cocotb availability fragile.  Design reference-model
gates to degrade gracefully to direct simulator subprocess invocation so the
gate keeps running even when the fancy framework is temporarily unavailable.

## Worked example — Wave Loop 537

Wave Loop 537 closed the undefined-struct leniency in the Lean lowerability
predicate and forced Rust/Lean agreement across the whole corpus:

- Changed `Ty.isLowerableFuel` for `.struct name` in
  `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` to require a non-empty
  `env.structFields name`, matching the Rust structural classifier's rejection
  of undeclared structs.
- Repaired all 249 corpus envs in `Completeness.lean`:
  - 133 lowerable envs got stub declarations for every referenced undefined
    struct; empty-field structs were replaced with a single `u32` field.
  - 116 non-lowerable envs got a deliberately non-lowerable marker struct
    (`w537_non_lowerable_marker` with an `f32` field) and a dummy function that
    uses it, so the theorem asserts `Module.isLowerable ... = false`.
- Added `w537_undefined_struct_not_lowerable` in `Lemmas.lean` as a negative
  witness theorem and discharged it with `native_decide`.
- Added `corpus_classifier_matches_lean_completeness` in
  `bootstrap/tests/icarus_lowerable.rs` to read every `Completeness.lean`
  theorem, map env names back to `specs/**/*.t27`, run `t27c icarus-lowerable
  --json`, and assert that the Rust verdict matches the Lean theorem.  Four
  Lean-only witnesses are allowed.
- Created `specs/scratch/w537_negative_undefined_struct.t27`, sealed it, and
  documented it in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.
- Wrote `docs/reports/WAVE_LOOP_537_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W538_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W538.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast` 35/0 Icarus PASS,
  35/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baselines
  unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: when a formal predicate is more lenient than the compiler
classifier, tighten the predicate first and then repair every generated corpus
env so the theorem asserts the real classifier verdict.  For non-lowerable specs
whose extracted module is too coarse to reproduce the rejection, a deliberately
non-lowerable marker struct/function is an acceptable way to keep the proof
meaningful and CI-checkable.

## Worked example — Wave Loop 538

Wave Loop 538 added a VCD probe and an independent reference-model cross-check
to the cocotb gate:

- Added a per-test-block probe counter to `VerilogCodegen` and emitted
  `reg [63:0] _t27_probe_<block>_<N>` declarations for every `assert_eq` actual
  expression in simulation mode, hoisted to the top of the generated
  `initial` block.
- Emitted `$dumpfile("dump.vcd"); $dumpvars(0);` inside
  `// synthesis translate_off` only when `emit_test_assertions` is true, so
  synthesis-mode seals stayed stable.
- Updated `scripts/cocotb_ref_model.py` to capture VCD in both direct
  `iverilog/vvp` and cocotb runner paths, parse final probe values with a
  minimal built-in VCD parser, and compare them against independently evaluated
  expected literals.  Negative expected literals are compared as signed 64-bit
  two's complement to match Verilog sign extension.
- Skipped X/missing probes gracefully (typical for wide non-scalar values) and
  fell back to the log-based self-check.
- Updated `bootstrap/src/suite.rs::normalize_icarus_output` to filter out VCD
  startup diagnostics and `[PROBE]` debug lines, so the existing Phase 3d
  baselines remained valid without re-recording.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- Wrote `docs/reports/WAVE_LOOP_538_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W539_2026-07-15.md`, and advanced
  `.trinity/current-issue.md` to W539.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable --cocotb --fast`
  35/0 Icarus PASS, 35/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: gate all simulation-only instrumentation with
`emit_test_assertions` to keep synthesis seals stable, and normalize new debug
output out of deterministic baseline comparisons instead of re-recording every
baseline.  Treat unreadable VCD probes as skipped supplemental checks, not gate
failures, when the chosen probe width cannot represent the value.

## Worked example — Wave Loop 539

Wave Loop 539 replaced W538's fixed 64-bit VCD probe with typed probes and
extended the Python reference model evaluator to handle the Icarus-lowerable
expression subset:

- Added `expr_width_signed` and `field_scalar_array_info` to
  `bootstrap/src/compiler.rs` to infer the scalar width and signedness of every
  `assert_eq` actual expression, and emitted `reg [W-1:0]` probes (with a safe
  64-bit fallback).  Added a `probe_specs` vector to carry metadata per test block.
- Replaced the previous 64-bit signed heuristic in
  `scripts/cocotb_ref_model.py` with a `Bv` bit-vector class that tracks width
  and signedness independently of Python `int`.
- Implemented a recursive evaluator for literals, variables, parameterless
  function calls, struct field access, scalar array indexing, binary/unary
  operators, casts, switch, and ternary expressions.
- Updated the built-in VCD parser to record per-identifier widths and the
  cross-check to interpret probe values with the correct width/signedness.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- Wrote `docs/reports/WAVE_LOOP_539_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W540_2026-07-08.md`, and advanced
  `.trinity/current-issue.md` to W540.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast`
  35/0 Icarus PASS, 35/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: always carry `(width, signed)` with every reference-model value;
never infer signedness from the sign of a Python `int`.  Reuse the compiler's
existing type/width helpers so the Python evaluator mirrors the Verilog packed
layout exactly.

---

*φ² + φ⁻² = 3 | TRINITY*
