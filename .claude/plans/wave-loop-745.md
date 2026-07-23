# Wave Loop 745 — Decomposed Plan

**Issue:** #1716 (expected)
**Branch:** `wave-loop-745`
**Date:** 2026-07-22
**Previous:** Wave Loop 744 (#1715, branch `wave-loop-744`)

## Goal

Validate a module-scope `[309][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes. The witness is a 632,832-bit packed vector (19,776 elements,
~0.604 MiBit), continuing the odd outer-dimension ladder while staying far below
the ~4-MiBit simulation ceiling.

At the same time, use the closeout as a checkpoint to audit repository weak
points, scan fresh scientific literature, and prepare a decomposed backlog for
Wave Loop 746 and the next ring.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Accellera `vlog-pp` discussion (Graham 2002) — packed arrays are contiguous
  bit vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W745
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals;
  flat `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Fresh literature scan (2025–2026)

- 5500FP / GargantuRAM (2025) — native 24-trit RISC CPU and memory subsystem,
  demonstrating practical balanced-ternary compute at scale. Relevant because
  t27's packed `i16` arrays are a precursor to wider MVL storage kernels.
- IEEE Access 2025 — RTL-based ternary synthesis and transistor-level analysis.
  Validates that ternary arithmetic units can be synthesized with commercial tools
  and that packed encoding reduces wiring cost, analogous to t27's scalarized
  packed vectors.
- Tlsys (2025) — CNFET-based ternary logic synthesis. Shows that MVL physical
  design now has its own tool flow; t27's generator pattern could be retargeted
  to emit Tlsys-compatible structural netlists in future.
- TVHDL (2025) — ternary VHDL extension. Confirms that non-binary HDL dialects
  continue to standardize around packed-aggregate flattening.
- Takahe (2025) — multi-radix synthesis including radix-3. Useful reference for
  the next time t27 needs to target ternary-specific backend lowering.
- KULeuven ternary-lut-dse (2025) — LUT-depth design exploration for ternary
  lookup tables. Relevant for ternary memory mapping once t27 leaves the Icarus
  simulation domain.
- GSTE / STE parameterized model checking (2024–2025) — formal verification of
  memory arrays via symbolic trajectory evaluation. Ties to the long-term
  goal of proving t27's packed-array layout and field-write frame conditions.

## Tasks

1. [x] **Select variant.** Variant A: module-scope `[309][2]^6 Pt` variable
   initialized from a call, with indexed signed field writes and read-back.
2. [x] **Create generator.** Copy `scripts/gen_w744.py` to `scripts/gen_w745.py`,
   set `OUTER = 309` and `MID_IDX = 154`, verify module name uses `w745`.
3. [x] **Generate witness.** Run `python3 scripts/gen_w745.py` to produce
   `specs/scratch/w745_bench_module_309x2p6_aos_var_call_write.t27`.
4. [x] **Add integration test.** Append
   `accepts_w745_bench_module_309x2p6_aos_var_call_write` to
   `bootstrap/tests/icarus_lowerable.rs`.
5. [x] **Build and gate.** Run `cargo build --release -p t27c`, then direct
   `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`.
6. [x] **Run cargo suites.** `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
   `cargo test -p t27c --test icarus_lowerable`.
7. [x] **Save artifacts.** Empty Icarus baseline in
   `.trinity/icarus-baselines/specs/scratch/`, seal in `.trinity/seals/`.
8. [x] **Closeout report.** Write `docs/reports/FPGA_LOOP_CLOSEOUT_W745_2026-07-22.md`.
9. [x] **Experience update.** Prepend W745 learnings to `.trinity/experience.md`.
10. [x] **Current issue handoff.** Update `.trinity/current-issue.md` to W746.
11. [x] **Memory update.** Write
    `~/.claude/projects/-Users-playra-t27/memory/wave-loop-745.md` and append
    one-line pointer to `MEMORY.md`.
12. [x] **Branch / merge.** Commit W745 with `Closes #1716`; create
    `wave-loop-746`.

## Decomposed backlog for next ring

1. [ ] **L1 cleanup.** Retroactively link or open issues for the wave-loop
       `feat(igla)` / `chore(trinity)` commits so that every merged change has a
       `Closes #N` reference.
2. [ ] **L4 backfill.** Add `test`/`invariant`/`bench` blocks to the 57
       missing-TDD specs, starting with `specs/numeric/gf*.t27` and
       `specs/tri/pipeline/*.t27` / `specs/tri/agent/*.t27`.
3. [ ] **L7 migration.** Retire or re-implement the 19 `scripts/*.sh` critical
       wrappers as `t27c` subcommands (e.g. `t27c verify`, `t27c reseal-apply`).
4. [ ] **FPGA SSOT reconciliation.** Align `CLAUDE.md`, `cli/dlc10`, and stale
       FPGA docs with `fpga/HARDWARE_SSOT.md`; decide whether to update `dlc10`
       or deprecate it.
5. [ ] **L6 reconciliation.** Resolve the `GF256` bias caveat in
       `conformance/FORMAT-SPEC-001.json`.
6. [ ] **Simulator stress wave.** Plan a Wave Loop near the 4-MiBit packed
       vector boundary to measure Icarus/Yosys wall-clock limits.

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Generator copy forgets `MID_IDX` or module-name update | Medium | Low | Verify generated footer assertions and module header before running gates. |
| Icarus simulation path rejects `assert_ne` | High (known) | Low | Use `assert_eq` on changed elements only. |
| Mid-row expected value computed incorrectly | Low | Medium | Reuse corrected W632 formula: `e = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0`. |
| Simulator capacity surprise near 0.604 MiBit | Very low | High | Direct `icarus-simulate` gate runs quickly; abort if elapsed exceeds prior waves significantly. |
| Offset-32768 identity check mischaracterised as wrap test | Medium | Low | Document in report that the check verifies period identity, not first-time wrap. |
| L1/L4/L7/L6 gaps block next major release | Medium | High | Prioritize in post-W745 backlog and assign ring stewards. |
| FPGA SSOT contradiction causes wrong physical target | Medium | High | Reconcile docs before any `dlc10` / openFPGALoader flashing. |
| Dirty worktree leaks into next commit | Medium | Medium | Revert `.claude/scheduled_tasks.lock` and ignore `.agents`/`.codex` before committing. |

## Next Wave Loop 746 cooperation variants

1. **Variant A (recommended): `[311][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 636,928-bit packed vector, 19,904 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 311.
   - **Recommended.**

2. **Variant B: `[309][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W745 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[309][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.604 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] `.claude/plans/wave-loop-745.md` exists and covers the above.
- [x] `scripts/gen_w745.py` created with `OUTER = 309` and `MID_IDX = 154`.
- [x] `specs/scratch/w745_bench_module_309x2p6_aos_var_call_write.t27` generated.
- [x] Integration test `accepts_w745_bench_module_309x2p6_aos_var_call_write` added and passing.
- [x] `cargo build --release -p t27c` passes.
- [x] Direct `t27c` gates pass (parse, lowerable, simulate, cocotb, seal).
- [x] All cargo test suites pass.
- [x] Empty Icarus baseline saved.
- [x] Closeout report, experience, and memory updated.
- [ ] Branch `wave-loop-746` created and W745 committed with `Closes #1716`.

