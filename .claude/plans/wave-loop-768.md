# Wave Loop 768 Plan — Issue #1739

## Phase 1: Issue
- Confirm W768 issue #1739 and branch `wave-loop-768`.

## Phase 2: Spec
- Generate `specs/scratch/w768_bench_module_355x2p6_aos_var_call_write.t27` using
  `scripts/gen_w768.py` derived from `scripts/gen_w767.py`.
- Constants: `OUTER = 355`, `TOTAL = 22720`, `LAST_IDX = 354`, `MID_IDX = 177`.

## Phase 3: TDD
- Inspect generated assertions: `make_grid(32768)`, `make_grid(1)`,
  `call_write_grid`, field writes, `assert_eq` read-backs, mid and last checks.

## Phase 4: Impl
- Add integration test `accepts_w768_bench_module_355x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.

## Phase 5: Gen
- Run `cd bootstrap && cargo build --release -p t27c`.

## Phase 6: Seal
- Run `./bootstrap/target/release/t27c seal --save specs/scratch/w768_bench_module_355x2p6_aos_var_call_write.t27`.
- Confirm FROZEN_HASH unchanged.

## Phase 7: Verify
- `./bootstrap/target/release/t27c parse ...`
- `./bootstrap/target/release/t27c icarus-lowerable ...`
- `./bootstrap/target/release/t27c icarus-simulate ...` (17 cycles)
- `./bootstrap/target/release/t27c icarus-cocotb ...`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`

## Phase 8: Land
- Commit with `feat(igla): Wave Loop 768 — module-scope [355][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes (Closes #1739)`.
- Push `wave-loop-768`, open PR, merge to `master`, create `wave-loop-769`.

## Phase 9: Learn
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W768_2026-07-23.md`.
- Save `.trinity/experience.md`, memory file, `MEMORY.md` pointer, and update
  `skills-wave-loop-recipe.md`.

## Cooperation variants for next Wave Loop

- **A (recommended):** `[357][2]^6 Pt` (~0.698 MiBit), continuing the odd ladder.
- **B:** move the packed variable to bench/function scope at the same width.
- **C:** add conditional (`if`) indexed writes at the current width.
