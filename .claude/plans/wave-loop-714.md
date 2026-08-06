# Wave Loop 714 Plan — `[247][2]^6 Pt` module-scope packed AoS var

**Issue #1685** | **Branch:** `wave-loop-714` | **Previous:** #1684 `wave-loop-713`

## Objective

Validate that the t27 compiler and reference model correctly handle a module-scope
packed array-of-structs with a non-power-of-two outer dimension of **247**,
initialized from a function call and mutated via indexed signed field writes.
Total width: **514,432 bits** (15,952 elements, ≈0.490 MiBit), still well below
the ~4-MiBit Icarus/Yosys comfort threshold.

## Decomposition

| # | Task | Owner | Acceptance criteria |
|---|------|-------|---------------------|
| 1 | Update `.trinity/current-issue.md` for #1685 | Queen (plan) | Issue #1685, variant A chosen, risks + next variants recorded. |
| 2 | Create generator `scripts/gen_w714.py` | Creator | Copy from `gen_w713.py`, set `OUTER = 247`, fix `MID_IDX` comment to `# 123`. |
| 3 | Generate witness `specs/scratch/w714_bench_module_247x2p6_aos_var_call_write.t27` | Creator | Script runs; multi-line W584 brace style; literal contains 15,952 `Pt{}` leaves. |
| 4 | Add integration test `accepts_w714_...` to `bootstrap/tests/icarus_lowerable.rs` | Creator | Test asserts the witness is icarus-lowerable. |
| 5 | Build and run direct gates | Verifier | `cargo build --release -p t27c` green; `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save` all PASS. |
| 6 | Run test suites | Verifier | `cargo test -p t27c --bin t27c` 1494/0/2; `cargo test -p tri` 78/0; `cargo test -p t27c --test icarus_lowerable` 174/0. |
| 7 | Create Icarus baseline `.trinity/icarus-baselines/specs/scratch/w714_...json` | Verifier | Empty array baseline committed. |
| 8 | Write closeout report | Queen | `docs/reports/FPGA_LOOP_CLOSEOUT_W714_2026-07-07.md` with W715 variants. |
| 9 | Update `.trinity/experience.md` and persistent memory | Learner | W714 learnings + memory index entry. |
| 10 | Branch `wave-loop-715` and commit with `Closes #1685` | Queen | Clean commit stack, session log + commit count checkpointed. |

## Risk register

| Risk | Mitigation | Owner |
|------|------------|-------|
| `MID_IDX` comment drift when copying generator | Manual fix to `# 123` after `sed` | Creator |
| Witness too large for interactive simulation | Stay at 0.490 MiBit; skip full-sim if >4 MiBit | Verifier |
| `assert_ne` not emitted by Icarus path | Use `assert_eq` on changed elements, not whole-array inequality | Creator |
| Modulo-wrap regression signal lost | Explicit `make_grid(32768)` assertions in test | Creator |
| Non-p2 outer dimension 247 striding bug | Mid-row indexed check `[123][1][0][0][0][0][0]` catches offset error | Verifier |

## Next Wave Loop 715 cooperation variants

1. **Variant A (recommended): `[249][2]^6 Pt` module-scope var from call with indexed signed writes.**
   520,704-bit packed vector, 16,064 elements, non-power-of-two outer dimension 249.
   Continues the odd outer-dimension ladder. **Recommended.**

2. **Variant B: `[247][2]^6 Pt` bench-local (function-local) packed array var from call with indexed signed writes.**
   Tests the same non-p2 outer dimension at function/bench scope.

3. **Variant C: `[247][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**
   Stays at 0.490 MiBit; tests control-flow guarded writes on packed reg.

## Exit criteria

- [x] All tasks above completed.
- [x] All conformance gates PASS.
- [x] `FROZEN_HASH` unchanged (no spec hashes changed).
- [x] Closeout report and memory saved.
- [x] Branch `wave-loop-715` exists and `wave-loop-714` is ready for PR.
