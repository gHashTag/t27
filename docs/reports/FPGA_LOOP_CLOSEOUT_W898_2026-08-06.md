# FPGA / Wave Loop Close-out — W898

**Date:** 2026-08-06  
**Issue:** #1859 — feat(igla): Wave Loop 898 — module-scope `[615][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes  
**Branch:** `wave-loop-898`  
**PR:** #1900

---

## Summary

Wave Loop 898 extends the mechanical packed-vector array-of-struct ladder with a module-scope `[615][2]^6 Pt` witness. The resulting packed vector is **1,259,520 bits (~1.202 MiBit)** from 39,360 elements (615 outer × 64 inner). The witness is initialized from a pure `make_grid(0)` function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block. No compiler changes were required.

---

## What changed

- Added generator `scripts/gen_w898.py` (copied from `gen_w897.py`, copy-hazard checklist cleared).
- Added spec `specs/scratch/w898_bench_module_615x2p6_aos_var_call_write.t27` (~2.6 MB / 116,911 lines).
- Added seal `.trinity/seals/scratch_w898_bench_module_615x2p6_aos_var_call_write.json`.
- Added integration test `accepts_w898_bench_module_615x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
- Zero changes to `bootstrap/src/compiler.rs` or `bootstrap/stage0/FROZEN_HASH`.

---

## Validation matrix

| Gate | Command | Result |
|---|---|---|
| Parse | `t27c parse specs/scratch/w898_bench_module_615x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved |
| Seal verify | `t27c seal --verify ...` | `MATCH` |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w898_bench_module_615x2p6_aos_var_call_write` | PASS |
| Full suite | `cargo test --release --test icarus_lowerable` | 357 passed / 1 pre-existing failure |

The single failure is the pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27`; it is not introduced by W898.

---

## Learnings

- The 1.20-MiBit neighborhood remains a soft boundary for t27c and Icarus at 1.202 MiBit.
- The mechanical checklist still prevents copy hazards; no compiler changes needed.
- Full `icarus_lowerable` suite runtime is still acceptable for CI.

---

## Next-wave cooperation variants

Prepared in `.claude/plans/wave-loop-899.md`.

### Variant A (recommended) — continue the ladder
- Module-scope `[617][2]^6 Pt` packed AoS variable from call with indexed signed writes.
- Outer dimension 617 → 39,488 elements → ~1.206 MiBit.
- Smallest, reviewable diff; keeps mechanical ladder moving.

### Variant B — increase inner struct width
- Keep outer dimension 615 but expand the inner struct (e.g., `[2]^8 Pt` or `[4]^6 Pt`).
- Tests whether the ceiling is element count or total packed-vector width.
- Larger spec and longer CI; defer until ladder hits a hard boundary.

### Variant C — stress negative/variable indexing
- Add non-constant signed index expressions inside the `bench` block.
- Probes Icarus index-normalization and cocotb reference-model agreement.
- Risk of compiler/reference-model changes; keep as a side experiment.

---

## Closure

- Issue #1859 closed by PR #1900.
- Next wave: #1901 — Wave Loop 899 (`[617][2]^6 Pt`).

phi^2 + 1/phi^2 = 3 | TRINITY
