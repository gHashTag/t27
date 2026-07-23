# Wave Loop 770 Plan — Issue #1741

## Phase 1: Issue
- Confirm W770 issue #1741 and branch `wave-loop-770`.

## Phase 2: Spec
- Generate `specs/scratch/w770_bench_module_359x2p6_aos_var_call_write.t27` using
  `scripts/gen_w770.py` derived from `scripts/gen_w769.py`.
- Constants: `OUTER = 359`, `TOTAL = 22976`, `LAST_IDX = 358`, `MID_IDX = 179`.

## Phase 3: TDD
- Inspect generated assertions: `make_grid(32768)`, `make_grid(1)`,
  `call_write_grid`, field writes, `assert_eq` read-backs, mid and last checks.

## Phase 4: Impl
- Add integration test `accepts_w770_bench_module_359x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.

## Phase 5: Gen
- Run `cd bootstrap && cargo build --release -p t27c`.

## Phase 6: Seal
- Run `./bootstrap/target/release/t27c seal --save specs/scratch/w770_bench_module_359x2p6_aos_var_call_write.t27`.
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
- Commit with `feat(igla): Wave Loop 770 — module-scope [359][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes (Closes #1741)`.
- Push `wave-loop-770`, open PR, merge to `master`, create `wave-loop-771`.

## Phase 9: Learn
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W770_2026-07-24.md`.
- Save `.trinity/experience.md`, memory file, `MEMORY.md` pointer, and update
  `skills-wave-loop-recipe.md`.

## Cooperation variants for next Wave Loop

- **A (recommended):** `[361][2]^6 Pt` (~0.705 MiBit), continuing the odd ladder.
- **B:** move the packed variable to bench/function scope at the same width.
- **C:** add conditional (`if`) indexed writes at the current width.
