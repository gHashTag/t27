# FPGA / Wave Loop Close-out — W889

**Date:** 2026-08-06  
**Issue:** #1838 — feat(igla): Wave Loop 889 — module-scope `[597][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes  
**Branch:** `wave-loop-889`  
**PR:** #1840

---

## Summary

Wave Loop 889 extends the mechanical packed-vector array-of-struct ladder with a module-scope `[597][2]^6 Pt` witness. The resulting packed vector is **1,222,656 bits (~1.166 MiBit)** from 38,208 elements (597 outer × 64 inner). The witness is initialized from a pure `make_grid(0)` function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block. No compiler changes were required.

---

## What changed

- Added generator `scripts/gen_w889.py` (copied from `gen_w888.py`, copy-hazard checklist cleared).
- Added spec `specs/scratch/w889_bench_module_597x2p6_aos_var_call_write.t27`.
- Added seal `.trinity/seals/scratch_w889_bench_module_597x2p6_aos_var_call_write.json`.
- Added integration test `accepts_w889_bench_module_597x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
- Zero changes to `bootstrap/src/compiler.rs` or `bootstrap/stage0/FROZEN_HASH`.

---

## Validation matrix

| Gate | Command | Result |
|---|---|---|
| Parse | `t27c parse specs/scratch/w889_bench_module_597x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved |
| Seal verify | `t27c seal --verify ...` | `MATCH` |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w889_bench_module_597x2p6_aos_var_call_write` | PASS |
| Full suite | `cargo test --release --test icarus_lowerable` | 348 passed / 1 pre-existing failure |

The single failure is the pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27`; it is not introduced by W889.

---

## Learnings

- The 1.17-MiBit neighborhood remains a soft boundary for t27c and Icarus at 1.166 MiBit.
- When rebasing a branch onto `origin/master` after the previous wave's close-out content has already landed via squash merge, the redundant close-out commit may conflict. Using `git rebase --skip` drops it cleanly and lets the implementation apply without loss.
- The standard copy-hazard checklist (destination path, module header, `MID_IDX` comment) continues to prevent stale-reference bugs when copying generator scripts.

---

## Next-wave cooperation variants

Prepared in `.claude/plans/wave-loop-890.md`.

### Variant A (recommended) — continue the ladder
- Module-scope `[599][2]^6 Pt` packed AoS variable from call with indexed signed writes.
- Outer dimension 599 → 38,336 elements → ~1.170 MiBit.
- Smallest, reviewable diff; keeps mechanical ladder moving.

### Variant B — increase inner struct width
- Keep outer dimension 597 but increase the inner struct (e.g., `[2]^8 Pt` or `[4]^6 Pt`).
- Tests whether the ceiling is element count or total packed-vector width.
- Larger spec and longer CI time; less predictable blast radius.

### Variant C — stress negative/variable indexing
- Add non-constant signed index expressions (e.g., loop variable + offset) inside the `bench` block.
- Probes Icarus index-normalization and cocotb reference-model agreement.
- Risk of requiring compiler or reference-model changes; keep as a side experiment.

---

## Closure

- Issue #1838 closed by PR #1840.
- Next wave: #1841 — Wave Loop 890 (`[599][2]^6 Pt`).

phi^2 + 1/phi^2 = 3 | TRINITY
