# Wave Loop 716 Plan — `[251][2]^6 Pt` module-scope packed AoS var

**Issue #1687** | **Branch:** `wave-loop-716` | **Previous:** #1686 `wave-loop-715`

## Objective

Validate that the t27 compiler and reference model correctly handle a module-scope
packed array-of-structs with a non-power-of-two outer dimension of **251**,
initialized from a function call and mutated via indexed signed field writes.
Total width: **526,976 bits** (16,176 elements, ≈0.502 MiBit), still well below
the ~4-MiBit Icarus/Yosys comfort threshold.

## Decomposition

| # | Task | Owner | Acceptance criteria |
|---|------|-------|---------------------|
| 1 | Update `.trinity/current-issue.md` for #1687 | Queen (plan) | Issue #1687, variant A chosen, risks + next variants recorded. |
| 2 | Create generator `scripts/gen_w716.py` | Creator | Copy from `gen_w715.py`, set `OUTER = 251`, fix `MID_IDX` comment to `# 125`. |
| 3 | Generate witness `specs/scratch/w716_bench_module_251x2p6_aos_var_call_write.t27` | Creator | Script runs; multi-line W584 brace style; literal contains 16,176 `Pt{}` leaves. |
| 4 | Add integration test `accepts_w716_...` to `bootstrap/tests/icarus_lowerable.rs` | Creator | Test asserts the witness is icarus-lowerable. |
| 5 | Build and run direct gates | Verifier | `cargo build --release -p t27c` green; `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save` all PASS. |
| 6 | Run test suites | Verifier | `cargo test -p t27c --bin t27c` 1494/0/2; `cargo test -p tri` 78/0; `cargo test -p t27c --test icarus_lowerable` 176/0. |
| 7 | Create Icarus baseline `.trinity/icarus-baselines/specs/scratch/w716_...json` | Verifier | Empty array baseline committed. |
| 8 | Write closeout report | Queen | `docs/reports/FPGA_LOOP_CLOSEOUT_W716_2026-07-07.md` with W717 variants. |
| 9 | Update `.trinity/experience.md` and persistent memory | Learner | W716 learnings + memory index entry. |
| 10 | Branch `wave-loop-717` and commit with `Closes #1687` | Queen | Clean commit stack, session log + commit count checkpointed. |

## Risk register

| Risk | Mitigation | Owner |
|------|------------|-------|
| `MID_IDX` comment drift when copying generator | Manual fix to `# 125` after `sed` | Creator |
| Witness too large for interactive simulation | Stay at 0.502 MiBit; skip full-sim if >4 MiBit | Verifier |
| `assert_ne` not emitted by Icarus path | Use `assert_eq` on changed elements, not whole-array inequality | Creator |
| Modulo-wrap regression signal lost | Explicit `make_grid(32768)` assertions in test | Creator |
| Non-p2 outer dimension 251 striding bug | Mid-row indexed check `[125][1][0][0][0][0][0]` catches offset error | Verifier |

## Next Wave Loop 717 cooperation variants

1. **Variant A (recommended): `[253][2]^6 Pt` module-scope var from call with indexed signed writes.**
   533,248-bit packed vector, 16,288 elements, non-power-of-two outer dimension 253.
   Continues the odd outer-dimension ladder. **Recommended.**

2. **Variant B: `[251][2]^6 Pt` bench-local (function-local) packed array var from call with indexed signed writes.**
   Tests the same non-p2 outer dimension at function/bench scope.

3. **Variant C: `[251][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**
   Stays at 0.502 MiBit; tests control-flow guarded writes on packed reg.

## Exit criteria

- [x] All tasks above completed.
- [x] All conformance gates PASS.
- [x] `FROZEN_HASH` unchanged (no spec hashes changed).
- [x] Closeout report and memory saved.
- [x] Branch `wave-loop-717` exists and `wave-loop-716` is ready for PR.
