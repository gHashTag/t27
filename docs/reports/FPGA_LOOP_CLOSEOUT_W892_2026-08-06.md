# FPGA / Wave Loop Close-out — W892

**Date:** 2026-08-06  
**Issue:** #1845 — feat(igla): Wave Loop 892 — module-scope `[603][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes  
**Branch:** `wave-loop-892`  
**PR:** #1847

---

## Summary

Wave Loop 892 extends the mechanical packed-vector array-of-struct ladder with a module-scope `[603][2]^6 Pt` witness. The resulting packed vector is **1,234,944 bits (~1.178 MiBit)** from 38,592 elements (603 outer × 64 inner). The witness is initialized from a pure `make_grid(0)` function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block. No compiler changes were required.

---

## What changed

- Added generator `scripts/gen_w892.py` (copied from `gen_w891.py`, copy-hazard checklist cleared).
- Added spec `specs/scratch/w892_bench_module_603x2p6_aos_var_call_write.t27`.
- Added seal `.trinity/seals/scratch_w892_bench_module_603x2p6_aos_var_call_write.json`.
- Added integration test `accepts_w892_bench_module_603x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
- Zero changes to `bootstrap/src/compiler.rs` or `bootstrap/stage0/FROZEN_HASH`.

---

## Validation matrix

| Gate | Command | Result |
|---|---|---|
| Parse | `t27c parse specs/scratch/w892_bench_module_603x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved |
| Seal verify | `t27c seal --verify ...` | `MATCH` |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w892_bench_module_603x2p6_aos_var_call_write` | PASS |
| Full suite | `cargo test --release --test icarus_lowerable` | 351 passed / 1 pre-existing failure |

The single failure is the pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27`; it is not introduced by W892.

---

## Learnings

- The 1.18-MiBit neighborhood remains a soft boundary for t27c and Icarus at 1.178 MiBit.
- The mechanical checklist still prevents copy hazards; no compiler changes needed.
- Full `icarus_lowerable` suite runtime is ~43 seconds; still acceptable for CI.

---

## Next-wave cooperation variants

Prepared in `.claude/plans/wave-loop-893.md`.

### Variant A (recommended) — continue the ladder
- Module-scope `[605][2]^6 Pt` packed AoS variable from call with indexed signed writes.
- Outer dimension 605 → 38,720 elements → ~1.182 MiBit.
- Smallest, reviewable diff; keeps mechanical ladder moving.

### Variant B — increase inner struct width
- Keep outer dimension 603 but expand the inner struct (e.g., `[2]^8 Pt` or `[4]^6 Pt`).
- Tests whether the ceiling is element count or total packed-vector width.
- Larger spec and longer CI; defer until ladder hits a hard boundary.

### Variant C — stress negative/variable indexing
- Add non-constant signed index expressions inside the `bench` block.
- Probes Icarus index-normalization and cocotb reference-model agreement.
- Risk of compiler/reference-model changes; keep as a side experiment.

---

## Closure

- Issue #1845 closed by PR #1847.
- Next wave: #1848 — Wave Loop 893 (`[605][2]^6 Pt`).

phi^2 + 1/phi^2 = 3 | TRINITY
