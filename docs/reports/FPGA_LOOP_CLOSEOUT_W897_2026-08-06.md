# FPGA / Wave Loop Close-out — W897

**Date:** 2026-08-06  
**Issue:** #1857 — feat(igla): Wave Loop 897 — module-scope `[613][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes  
**Branch:** `wave-loop-897`  
**PR:** #1858

---

## Summary

Wave Loop 897 extends the mechanical packed-vector array-of-struct ladder with a module-scope `[613][2]^6 Pt` witness. The resulting packed vector is **1,255,424 bits (~1.198 MiBit)** from 39,232 elements (613 outer × 64 inner). The witness is initialized from a pure `make_grid(0)` function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block. No compiler changes were required.

---

## What changed

- Added generator `scripts/gen_w897.py` (copied from `gen_w896.py`, copy-hazard checklist cleared).
- Added spec `specs/scratch/w897_bench_module_613x2p6_aos_var_call_write.t27` (~2.5 MB / 116,531 lines).
- Added seal `.trinity/seals/scratch_w897_bench_module_613x2p6_aos_var_call_write.json`.
- Added integration test `accepts_w897_bench_module_613x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
- Zero changes to `bootstrap/src/compiler.rs` or `bootstrap/stage0/FROZEN_HASH`.

---

## Validation matrix

| Gate | Command | Result |
|---|---|---|
| Parse | `t27c parse specs/scratch/w897_bench_module_613x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved |
| Seal verify | `t27c seal --verify ...` | `MATCH` |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w897_bench_module_613x2p6_aos_var_call_write` | PASS |
| Full suite | `cargo test --release --test icarus_lowerable` | 356 passed / 1 pre-existing failure |

The single failure is the pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27`; it is not introduced by W897.

---

## Learnings

- The 1.20-MiBit neighborhood remains a soft boundary for t27c and Icarus at 1.198 MiBit.
- The mechanical checklist still prevents copy hazards; no compiler changes needed.
- Full `icarus_lowerable` suite runtime is ~48 seconds; still acceptable for CI.

---

## Next-wave cooperation variants

Prepared in `.claude/plans/wave-loop-898.md`.

### Variant A (recommended) — continue the ladder
- Module-scope `[615][2]^6 Pt` packed AoS variable from call with indexed signed writes.
- Outer dimension 615 → 39,360 elements → ~1.202 MiBit.
- Smallest, reviewable diff; keeps mechanical ladder moving.

### Variant B — increase inner struct width
- Keep outer dimension 613 but expand the inner struct (e.g., `[2]^8 Pt` or `[4]^6 Pt`).
- Tests whether the ceiling is element count or total packed-vector width.
- Larger spec and longer CI; defer until ladder hits a hard boundary.

### Variant C — stress negative/variable indexing
- Add non-constant signed index expressions inside the `bench` block.
- Probes Icarus index-normalization and cocotb reference-model agreement.
- Risk of compiler/reference-model changes; keep as a side experiment.

---

## Closure

- Issue #1857 closed by PR #1858.
- Next wave: #1859 — Wave Loop 898 (`[615][2]^6 Pt`).

phi^2 + 1/phi^2 = 3 | TRINITY
