# Wave Loop 776 Plan — Issue TBD

## Phase 1: Issue
- Confirm W776 issue (GitHub will assign next available number after #1485) and branch `wave-loop-776`.
- Base branch: if PR #1484 (W774) and PR #1486 (W775) have both merged, branch from `master`; otherwise stack from `wave-loop-775` HEAD to keep the ladder unblocked.

## Phase 2: Spec
- Generate `specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27` using
  `scripts/gen_w776.py` derived from `scripts/gen_w775.py`.
- Constants: `OUTER = 371`, `TOTAL = 23744`, `LAST_IDX = 370`, `MID_IDX = 185`.

## Phase 3: TDD
- Inspect generated assertions: `make_grid(32768)`, `make_grid(1)`,
  `call_write_grid`, field writes, `assert_eq` read-backs, mid and last checks.
- Expected frame-condition element: `[185][1][0][0][0][0][0]` = element `185*64 + 32 = 11,872`.

## Phase 4: Impl
- Add integration test `accepts_w776_bench_module_371x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs` after the W775 test.
- Zero compiler / reference-model / FROZEN_HASH changes expected.

## Phase 5: Gen
- Run `cd bootstrap && cargo build --release -p t27c`.

## Phase 6: Seal
- Run `./target/release/t27c seal --save specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`.
- Confirm FROZEN_HASH unchanged.

## Phase 7: Verify
- `./target/release/t27c parse specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`
- `./target/release/t27c icarus-lowerable specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`
- `./target/release/t27c icarus-simulate specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27` (17 cycles expected)
- `./target/release/t27c icarus-cocotb specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`
- `cargo test -p t27c --bin t27c` (expected 1494/0/2)
- `cargo test -p tri` (expected 78/0)
- `cargo test -p t27c --test icarus_lowerable` (expected 236/0)

## Phase 8: Land
- Commit with `feat(igla): Wave Loop 776 — module-scope [371][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes (Closes #<ISSUE>)`.
- Push `wave-loop-776`, open PR #<ISSUE+1>, merge to `master` (or rebase/stack after W774/W775 land).

## Phase 9: Learn
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W776_2026-07-24.md`.
- Save `.trinity/experience.md`, memory file, `MEMORY.md` pointer, and update
  `.claude/skills/t27-wave-loop.md`.

## Cooperation variants for next Wave Loop

- **A (recommended):** `[373][2]^6 Pt` (~0.725 MiBit), continuing the odd ladder.
- **B:** move the packed variable to bench/function scope at the current `[371][2]^6 Pt` width.
- **C:** add conditional (`if`) indexed writes at the current `[371][2]^6 Pt` width.
