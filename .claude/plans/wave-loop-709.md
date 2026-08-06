# Wave Loop 709 Plan — `[237][2]^6 Pt` module-scope packed AoS var

**Issue #1680** | **Branch:** `wave-loop-709` | **Previous:** #1679 `wave-loop-708`

## Objective

Validate that the t27 compiler and reference model correctly handle a module-scope
packed array-of-structs with a non-power-of-two outer dimension of **237**,
initialized from a function call and mutated via indexed signed field writes.
Total width: **483,200 bits** (15,168 elements, ≈0.461 MiBit), still well below
the ~4-MiBit Icarus/Yosys comfort threshold.

## Decomposition

| # | Task | Owner | Acceptance criteria |
|---|------|-------|---------------------|
| 1 | Update `.trinity/current-issue.md` | Queen (plan) | Issue #1680, variant A chosen, risks + next variants recorded. |
| 2 | Create generator `scripts/gen_w709.py` | Creator | Copy from `gen_w708.py`, set `OUTER = 237`, fix `MID_IDX` comment to `# 118`. |
| 3 | Generate witness `specs/scratch/w709_bench_module_237x2p6_aos_var_call_write.t27` | Creator | Script runs; multi-line W584 brace style; literal contains 15,168 `Pt{}` leaves. |
| 4 | Add integration test `accepts_w709_...` to `bootstrap/tests/icarus_lowerable.rs` | Creator | Test asserts the witness is icarus-lowerable. |
| 5 | Build and run direct gates | Verifier | `cargo build --release -p t27c` green; `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save` all PASS. |
| 6 | Run test suites | Verifier | `cargo test -p t27c --bin t27c` 1495/0/2; `cargo test -p tri` 78/0; `cargo test -p t27c --test icarus_lowerable` 169/0. |
| 7 | Create Icarus baseline `.trinity/icarus-baselines/specs/scratch/w709_...json` | Verifier | Empty array baseline committed. |
| 8 | Write closeout report | Queen | `docs/reports/FPGA_LOOP_CLOSEOUT_W709_2026-07-07.md` with W710 variants. |
| 9 | Update `.trinity/experience.md` and persistent memory | Learner | W709 learnings + memory index entry. |
| 10 | Branch `wave-loop-710` and commit with `Closes #1680` | Queen | Clean commit stack, session log + commit count checkpointed. |

## Risk register

| Risk | Mitigation | Owner |
|------|------------|-------|
| `MID_IDX` comment drift when copying generator | Manual fix to `# 118` after `sed` | Creator |
| Witness too large for interactive simulation | Stay at 0.461 MiBit; skip full-sim if >4 MiBit | Verifier |
| `assert_ne` not emitted by Icarus path | Use `assert_eq` on changed elements, not whole-array inequality | Creator |
| Modulo-wrap regression signal lost | Explicit `make_grid(32768)` assertions in test | Creator |
| Non-p2 outer dimension 237 striding bug | Mid-row indexed check `[118][1][0][0][0][0][0]` catches offset error | Verifier |

## Next Wave Loop 710 cooperation variants

1. **Variant A (recommended): `[239][2]^6 Pt` module-scope var from call with indexed signed writes.**
   489,472-bit packed vector, 15,296 elements, non-power-of-two outer dimension 239.
   Continues the odd outer-dimension ladder. **Recommended.**

2. **Variant B: `[237][2]^6 Pt` bench-local (function-local) packed array var from call with indexed signed writes.**
   Tests the same non-p2 outer dimension at function/bench scope.

3. **Variant C: `[237][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**
   Stays at 0.461 MiBit; tests control-flow guarded writes on packed reg.

## Exit criteria

- [x] All tasks above completed.
- [x] All conformance gates PASS.
- [x] `FROZEN_HASH` unchanged (no spec hashes changed).
- [x] Closeout report and memory saved.
- [x] Branch `wave-loop-710` exists and `wave-loop-709` is ready for PR.
