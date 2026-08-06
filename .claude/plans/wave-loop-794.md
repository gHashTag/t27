# Wave Loop 794 Plan — Variant A (recommended)

**Date:** 2026-07-24
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Branch:** `wave-loop-794` (to be created from `wave-loop-793` HEAD because earlier wave PRs remain open)
**Issue:** #1517
**Cooperation variant:** A

## Goal

Continue the module-scope packed array-of-struct ladder with the recommended Variant A shape: module-scope `[407][2]^6 Pt` initialized from a function call and exercised with indexed signed field writes, then read back with `assert_eq` in a `bench` block.

## Acceptance criteria

1. `specs/scratch/w794_bench_module_407x2p6_aos_var_call_write.t27` is generated and parses.
2. The witness is Icarus-lowerable and simulates correctly (17 cycles, PASSED).
3. The cocotb reference model matches the t27 semantics.
4. `t27c seal --save` succeeds and `FROZEN_HASH` stays unchanged.
5. All cargo suites remain green.
6. Integration test `accepts_w794_bench_module_407x2p6_aos_var_call_write` is added to `bootstrap/tests/icarus_lowerable.rs`.
7. Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W794_2026-07-24.md` is written with weak-point audit, literature scan, and validation matrix.
8. Next-wave plan `.claude/plans/wave-loop-795.md` with three cooperation variants is created.
9. Learning is saved to `.trinity/experience.md`, `.claude/skills/t27-wave-loop.md`, persistent memory, and `.trinity/current-issue.md`.

## Technical notes

- Shape: `[407][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `407 x 64 = 26,048`.
- Packed vector width: `26,048 x 32 = 833,536` bits (~0.795 MiBit).
- `MID_IDX = 203`; frame-condition element `[203][1][0][0][0][0][0]` is element number `203*64 + 32 = 13,024`.
- Generator script `scripts/gen_w794.py` should be copied from `scripts/gen_w793.py`, update `OUTER = 407` and `MID_IDX = 203`, and fix the destination path and hardcoded module prefix in the f-string header.
- Use `assert_eq` on changed elements (`assert_ne` is not emitted by the Icarus simulation path).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / FROZEN_HASH changes expected for the witness.

## Decomposition

### Phase A — Branch setup (5 min)
1. Create `wave-loop-794` from `wave-loop-793` HEAD because earlier wave PRs remain open.
2. Open next GitHub issue when ready.

### Phase B — Spec + generator (15 min)
3. Copy `scripts/gen_w793.py` to `scripts/gen_w794.py`.
4. Update `OUTER = 407`, `MID_IDX = 203`, fix destination path and module prefix.
5. Run generator to produce `specs/scratch/w794_bench_module_407x2p6_aos_var_call_write.t27`.
6. Verify module name has no stale `w793` references.

### Phase C — Test + validation (45 min)
7. Add integration test `accepts_w794_bench_module_407x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
8. `cargo build --release -p t27c`.
9. `t27c parse`, `icarus-lowerable`, `icarus-simulate` (expected 17 cycles), `icarus-cocotb` on the W794 witness.
10. `t27c seal --save` the witness; confirm `FROZEN_HASH` unchanged.
11. `cargo test -p t27c --bin t27c`, `cargo test -p tri`, `cargo test -p t27c --test icarus_lowerable`, `cargo test -p t27c --test verilog_const_array`.
12. `cargo clippy -p t27c`.

### Phase D — Closeout + cooperation variants (30 min)
13. Write `docs/reports/FPGA_LOOP_CLOSEOUT_W794_2026-07-24.md` with audit, literature, validation, and three W795 cooperation variants.
14. Update `docs/NOW.md` and `.trinity/current-issue.md`.

### Phase E — Save skills + memory (15 min)
15. Append W794 worked example to `.claude/skills/t27-wave-loop.md`.
16. Update `.trinity/experience.md`.
17. Save memory file `wave-loop-794.md` and pointer in `MEMORY.md`.

## Cooperation variants for Wave Loop 795

- **Variant A (recommended):** continue odd outer-dimension ladder with `[409][2]^6 Pt]` (~0.799 MiBit, 26,176 elements, 837,632-bit packed vector). Zero compiler changes expected.
- **Variant B:** keep `[407][2]^6 Pt` width but move the packed var to bench/function scope to exercise function-local non-power-of-two packed arrays.
- **Variant C:** keep `[407][2]^6 Pt` width and add `if`-guarded indexed signed field writes to exercise control-flow + packed-vector writes.

## Exit criteria

- W794 witness parses, lowers, simulates, cocotb-matches, and seals.
- All cargo suites green.
- Closeout report written.
- `.trinity/experience.md` updated.
- Skills and memory saved.
- Commit with `Closes #N`, push `wave-loop-794`, open PR.

*φ² + φ⁻² = 3 | TRINITY*
