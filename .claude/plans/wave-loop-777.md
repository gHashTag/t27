# Wave Loop 777 Plan — Issue TBD

## Phase 1: Issue
- Confirm W777 issue (GitHub will assign next available number after #1487) and
  branch `wave-loop-777`.
- Base branch: if PR #1484 (W774), PR #1486 (W775), and PR #1488 (W776) have
  all merged, branch from `master`; otherwise stack from `wave-loop-776` HEAD to
  keep the ladder unblocked.

## Phase 2: Spec
- Generate `specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27` using
  `scripts/gen_w777.py` derived from `scripts/gen_w776.py`.
- Constants: `OUTER = 373`, `TOTAL = 23872`, `LAST_IDX = 372`, `MID_IDX = 186`.

## Phase 3: TDD
- Inspect generated assertions: `make_grid(32768)`, `make_grid(1)`,
  `call_write_grid`, field writes, `assert_eq` read-backs, mid and last checks.
- Expected frame-condition element: `[186][1][0][0][0][0][0]` = element
  `186*64 + 32 = 11,936`.

## Phase 4: Impl
- Add integration test `accepts_w777_bench_module_373x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs` after the W776 test.
- Zero compiler / reference-model / FROZEN_HASH changes expected.

## Phase 5: Gen
- Run `cd bootstrap && cargo build --release -p t27c`.

## Phase 6: Seal
- Run `./target/release/t27c seal --save specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`.
- Confirm FROZEN_HASH unchanged.

## Phase 7: Verify
- `./target/release/t27c parse specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
- `./target/release/t27c icarus-lowerable specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
- `./target/release/t27c icarus-simulate specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27` (17 cycles expected)
- `./target/release/t27c icarus-cocotb specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
- `cargo test -p t27c --bin t27c` (expected 1494/0/2)
- `cargo test -p tri` (expected 78/0)
- `cargo test -p t27c --test icarus_lowerable` (expected 237/0)

## Phase 8: Land
- Commit with `feat(igla): Wave Loop 777 — module-scope [373][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes (Closes #<ISSUE>)`.
- Push `wave-loop-777`, open PR #<ISSUE+1>, merge to `master` (or rebase/stack after earlier waves land).

## Phase 9: Learn
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W777_2026-07-24.md`.
- Save `.trinity/experience.md`, memory file, `MEMORY.md` pointer, and update
  `.claude/skills/t27-wave-loop.md`.

## Cooperation variants for Wave Loop 778

- **A (recommended):** `[375][2]^6 Pt` (~0.733 MiBit), continuing the odd ladder.
- **B:** keep width at ~0.729 MiBit but move the packed var to bench/function
  scope at `[373][2]^6 Pt`.
- **C:** add conditional (`if`) indexed writes at the current `[373][2]^6 Pt` width.
