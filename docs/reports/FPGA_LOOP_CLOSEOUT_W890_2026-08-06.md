# FPGA / Wave Loop Close-out — W890

**Date:** 2026-08-06  
**Issue:** #1841 — feat(igla): Wave Loop 890 — module-scope `[599][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes  
**Branch:** `wave-loop-890`  
**PR:** #1842

---

## Summary

Wave Loop 890 extends the mechanical packed-vector array-of-struct ladder with a module-scope `[599][2]^6 Pt` witness. The resulting packed vector is **1,226,752 bits (~1.170 MiBit)** from 38,336 elements (599 outer × 64 inner). The witness is initialized from a pure `make_grid(0)` function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block. No compiler changes were required.

---

## What changed

- Added generator `scripts/gen_w890.py` (copied from `gen_w889.py`, copy-hazard checklist cleared).
- Added spec `specs/scratch/w890_bench_module_599x2p6_aos_var_call_write.t27`.
- Added seal `.trinity/seals/scratch_w890_bench_module_599x2p6_aos_var_call_write.json`.
- Added integration test `accepts_w890_bench_module_599x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
- Zero changes to `bootstrap/src/compiler.rs` or `bootstrap/stage0/FROZEN_HASH`.

---

## Validation matrix

| Gate | Command | Result |
|---|---|---|
| Parse | `t27c parse specs/scratch/w890_bench_module_599x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved |
| Seal verify | `t27c seal --verify ...` | `MATCH` |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w890_bench_module_599x2p6_aos_var_call_write` | PASS |
| Full suite | `cargo test --release --test icarus_lowerable` | 349 passed / 1 pre-existing failure |

The single failure is the pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27`; it is not introduced by W890.

---

## Learnings

- The 1.17-MiBit neighborhood remains a soft boundary for t27c and Icarus at 1.170 MiBit.
- The mechanical ladder continues to require only the three-location generator copy-hazard fix (destination path, module header, `MID_IDX` comment).
- Full `icarus_lowerable` suite runtime is now ~46 seconds; still acceptable for CI.

---

## Next-wave cooperation variants

Prepared in `.claude/plans/wave-loop-891.md`.

### Variant A (recommended) — continue the ladder
- Module-scope `[601][2]^6 Pt` packed AoS variable from call with indexed signed writes.
- Outer dimension 601 → 38,464 elements → ~1.174 MiBit.
- Smallest, reviewable diff; keeps mechanical ladder moving.

### Variant B — increase inner struct width
- Keep outer dimension 599 but increase the inner struct (e.g., `[2]^8 Pt` or `[4]^6 Pt`).
- Tests whether the ceiling is element count or total packed-vector width.
- Larger spec and longer CI time; defer until ladder hits a hard boundary.

### Variant C — stress negative/variable indexing
- Add non-constant signed index expressions (e.g., loop variable + offset) inside the `bench` block.
- Probes Icarus index-normalization and cocotb reference-model agreement.
- Risk of requiring compiler or reference-model changes; keep as a side experiment.

---

## Closure

- Issue #1841 closed by PR #1842.
- Next wave: #1843 — Wave Loop 891 (`[601][2]^6 Pt`).

phi^2 + 1/phi^2 = 3 | TRINITY
