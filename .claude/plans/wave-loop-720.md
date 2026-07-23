# Wave Loop 720 Plan — `[259][2]^6 Pt` module-scope packed AoS var

**Issue #1691** (expected) | **Branch:** `wave-loop-720` | **Previous:** #1690 `wave-loop-719`

## Objective

Validate that the t27 compiler and reference model correctly handle a module-scope
packed array-of-structs with a non-power-of-two outer dimension of **259**,
initialized from a function call and mutated via indexed signed field writes.
Total width: **552,064 bits** (16,624 elements, ≈0.525 MiBit), still well below
the ~4-MiBit Icarus/Yosys comfort threshold.

## Decomposition

| # | Task | Owner | Acceptance criteria |
|---|------|-------|---------------------|
| 1 | Update `.trinity/current-issue.md` for #1691 | Queen (plan) | Issue #1691, variant A chosen, risks + next variants recorded. |
| 2 | Create generator `scripts/gen_w720.py` | Creator | Copy from `gen_w719.py`, set `OUTER = 259`, fix `MID_IDX` comment to `# 129`. |
| 3 | Generate witness `specs/scratch/w720_bench_module_259x2p6_aos_var_call_write.t27` | Creator | Script runs; multi-line W584 brace style; literal contains 16,624 `Pt{}` leaves. |
| 4 | Add integration test `accepts_w720_...` to `bootstrap/tests/icarus_lowerable.rs` | Creator | Test asserts the witness is icarus-lowerable. |
| 5 | Build and run direct gates | Verifier | `cargo build --release -p t27c` green; `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save` all PASS. |
| 6 | Run test suites | Verifier | `cargo test -p t27c --bin t27c` 1494/0/2; `cargo test -p tri` 78/0; `cargo test -p t27c --test icarus_lowerable` 180/0. |
| 7 | Create Icarus baseline `.trinity/icarus-baselines/specs/scratch/w720_...json` | Verifier | Empty array baseline committed. |
| 8 | Write closeout report | Queen | `docs/reports/FPGA_LOOP_CLOSEOUT_W720_2026-07-07.md` with W721 variants. |
| 9 | Update `.trinity/experience.md` and persistent memory | Learner | W720 learnings + memory index entry. |
| 10 | Branch `wave-loop-721` and commit with `Closes #1691` | Queen | Clean commit stack, session log + commit count checkpointed. |

## Risk register

| Risk | Mitigation | Owner |
|------|------------|-------|
| `MID_IDX` comment drift when copying generator | Manual fix to `# 129` after `sed` | Creator |
| Witness too large for interactive simulation | Stay at 0.525 MiBit; skip full-sim if >4 MiBit | Verifier |
| `assert_ne` not emitted by Icarus path | Use `assert_eq` on changed elements, not whole-array inequality | Creator |
| Modulo-wrap regression signal lost | Explicit `make_grid(32768)` assertions in test | Creator |
| Non-p2 outer dimension 259 striding bug | Mid-row indexed check `[129][1][0][0][0][0][0]` catches offset error | Verifier |

## Next Wave Loop 721 cooperation variants

1. **Variant A (recommended): `[261][2]^6 Pt` module-scope var from call with indexed signed writes.**
   558,336-bit packed vector, 16,736 elements, non-power-of-two outer dimension 261.
   Continues the odd outer-dimension ladder. **Recommended.**

2. **Variant B: `[259][2]^6 Pt` bench-local (function-local) packed array var from call with indexed signed writes.**
   Tests the same non-p2 outer dimension at function/bench scope.

3. **Variant C: `[259][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**
   Stays at 0.525 MiBit; tests control-flow guarded writes on packed reg.

## Exit criteria

- [x] All tasks above completed.
- [x] All conformance gates PASS.
- [x] `FROZEN_HASH` unchanged (no spec hashes changed).
- [x] Closeout report and memory saved.
- [x] Branch `wave-loop-721` exists and `wave-loop-720` is ready for PR.
