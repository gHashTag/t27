# Wave Loop 773 Plan — Issue #1481

## Phase 1: Issue
- Confirm W773 issue #1481 and branch `wave-loop-773`.

## Phase 2: Spec
- Generate `specs/scratch/w773_bench_module_365x2p6_aos_var_call_write.t27` using
  `scripts/gen_w773.py` derived from `scripts/gen_w772.py`.
- Constants: `OUTER = 365`, `TOTAL = 23360`, `LAST_IDX = 364`, `MID_IDX = 182`.

## Phase 3: TDD
- Inspect generated assertions: `make_grid(32768)`, `make_grid(1)`,
  `call_write_grid`, field writes, `assert_eq` read-backs, mid and last checks.

## Phase 4: Impl
- Add integration test `accepts_w773_bench_module_365x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.

## Phase 5: Gen
- Run `cd bootstrap && cargo build --release -p t27c`.

## Phase 6: Seal
- Run `./bootstrap/target/release/t27c seal --save specs/scratch/w773_bench_module_365x2p6_aos_var_call_write.t27`.
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
- Commit with `feat(igla): Wave Loop 773 — module-scope [365][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes (Closes #1481)`.
- Push `wave-loop-773`, open PR, merge to `master`, create `wave-loop-774`.

## Phase 9: Learn
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W773_2026-07-24.md`.
- Save `.trinity/experience.md`, memory file, `MEMORY.md` pointer, and update
  `skills-wave-loop-recipe.md`.

## Weak-point audit carried out this wave
- L1 TRACEABILITY: 51 of 61 30-day subject lines carry `Closes #N`/`Fixes #N` (≈84%).
- L4 TESTABILITY: 57 of 879 non-worktree `.t27` specs lack `test`/`invariant`/`bench` (≈6.5%).
- L7 UNITY: 19 `scripts/*.sh` remain under `scripts/`.
- Pre-existing FPGA failures tracked as weak point #1245.

## Scientific / engineering background consulted
- IEEE 1800-2017 packed arrays/structs (§7.4.1/§7.4.3).
- AMD UG900 2026.1 / AR 51836 Vivado packed-aggregate support.
- 2025-2026 ternary/MVL literature: KULeuven ternary-lut-dse (ISPASS 2026),
  TeLLMe v2, TerEffic, TernaryCore, SONIC (ISMVL 2026), TVHDL (ISMVL 2026).

## Cooperation variants for next Wave Loop

- **A (recommended):** `[367][2]^6 Pt` (~0.717 MiBit), continuing the odd ladder.
- **B:** move the packed variable to bench/function scope at the same width.
- **C:** add conditional (`if`) indexed writes at the current width.
