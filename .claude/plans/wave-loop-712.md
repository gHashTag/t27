# Wave Loop 712 Plan — `[243][2]^6 Pt` module-scope packed AoS var

**Issue #1683** | **Branch:** `wave-loop-712` | **Previous:** #1682 `wave-loop-711`

## Objective

Validate that the t27 compiler and reference model correctly handle a module-scope
packed array-of-structs with a non-power-of-two outer dimension of **243**,
initialized from a function call and mutated via indexed signed field writes.
Total width: **502,016 bits** (15,552 elements, ≈0.478 MiBit), still well below
the ~4-MiBit Icarus/Yosys comfort threshold.

## Decomposition

| # | Task | Owner | Acceptance criteria |
|---|------|-------|---------------------|
| 1 | Update `.trinity/current-issue.md` | Queen (plan) | Issue #1683, variant A chosen, risks + next variants recorded. |
| 2 | Create generator `scripts/gen_w712.py` | Creator | Copy from `gen_w711.py`, set `OUTER = 243`, fix `MID_IDX` comment to `# 121`. |
| 3 | Generate witness `specs/scratch/w712_bench_module_243x2p6_aos_var_call_write.t27` | Creator | Script runs; multi-line W584 brace style; literal contains 15,552 `Pt{}` leaves. |
| 4 | Add integration test `accepts_w712_...` to `bootstrap/tests/icarus_lowerable.rs` | Creator | Test asserts the witness is icarus-lowerable. |
| 5 | Build and run direct gates | Verifier | `cargo build --release -p t27c` green; `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save` all PASS. |
| 6 | Run test suites | Verifier | `cargo test -p t27c --bin t27c` 1495/0/2; `cargo test -p tri` 78/0; `cargo test -p t27c --test icarus_lowerable` 172/0. |
| 7 | Create Icarus baseline `.trinity/icarus-baselines/specs/scratch/w712_...json` | Verifier | Empty array baseline committed. |
| 8 | Write closeout report | Queen | `docs/reports/FPGA_LOOP_CLOSEOUT_W712_2026-07-07.md` with W713 variants. |
| 9 | Update `.trinity/experience.md` and persistent memory | Learner | W712 learnings + memory index entry. |
| 10 | Branch `wave-loop-713` and commit with `Closes #1683` | Queen | Clean commit stack, session log + commit count checkpointed. |

## Risk register

| Risk | Mitigation | Owner |
|------|------------|-------|
| `MID_IDX` comment drift when copying generator | Manual fix to `# 121` after `sed` | Creator |
| Witness too large for interactive simulation | Stay at 0.478 MiBit; skip full-sim if >4 MiBit | Verifier |
| `assert_ne` not emitted by Icarus path | Use `assert_eq` on changed elements, not whole-array inequality | Creator |
| Modulo-wrap regression signal lost | Explicit `make_grid(32768)` assertions in test | Creator |
| Non-p2 outer dimension 243 striding bug | Mid-row indexed check `[121][1][0][0][0][0][0]` catches offset error | Verifier |

## Next Wave Loop 713 cooperation variants

1. **Variant A (recommended): `[245][2]^6 Pt` module-scope var from call with indexed signed writes.**
   508,288-bit packed vector, 15,680 elements, non-power-of-two outer dimension 245.
   Continues the odd outer-dimension ladder. **Recommended.**

2. **Variant B: `[243][2]^6 Pt` bench-local (function-local) packed array var from call with indexed signed writes.**
   Tests the same non-p2 outer dimension at function/bench scope.

3. **Variant C: `[243][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**
   Stays at 0.478 MiBit; tests control-flow guarded writes on packed reg.

## Exit criteria

- [ ] All tasks above completed.
- [ ] All conformance gates PASS.
- [ ] `FROZEN_HASH` unchanged (no spec hashes changed).
- [ ] Closeout report and memory saved.
- [ ] Branch `wave-loop-713` exists and `wave-loop-712` is ready for PR.
