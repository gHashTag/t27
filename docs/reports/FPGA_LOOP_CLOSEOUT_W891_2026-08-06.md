# FPGA / Wave Loop Close-out — W891

**Date:** 2026-08-06  
**Issue:** #1843 — feat(igla): Wave Loop 891 — module-scope `[601][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes  
**Branch:** `wave-loop-891`  
**PR:** #1844

---

## Summary

Wave Loop 891 extends the mechanical packed-vector array-of-struct ladder with a module-scope `[601][2]^6 Pt` witness. The resulting packed vector is **1,230,848 bits (~1.174 MiBit)** from 38,464 elements (601 outer × 64 inner). The witness is initialized from a pure `make_grid(0)` function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block. No compiler changes were required.

---

## What changed

- Added generator `scripts/gen_w891.py` (copied from `gen_w890.py`, copy-hazard checklist cleared).
- Added spec `specs/scratch/w891_bench_module_601x2p6_aos_var_call_write.t27`.
- Added seal `.trinity/seals/scratch_w891_bench_module_601x2p6_aos_var_call_write.json`.
- Added integration test `accepts_w891_bench_module_601x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
- Zero changes to `bootstrap/src/compiler.rs` or `bootstrap/stage0/FROZEN_HASH`.

---

## Validation matrix

| Gate | Command | Result |
|---|---|---|
| Parse | `t27c parse specs/scratch/w891_bench_module_601x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved |
| Seal verify | `t27c seal --verify ...` | `MATCH` |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w891_bench_module_601x2p6_aos_var_call_write` | PASS |
| Full suite | `cargo test --release --test icarus_lowerable` | 350 passed / 1 pre-existing failure |

The single failure is the pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27`; it is not introduced by W891.

---

## Learnings

- The 1.18-MiBit neighborhood remains a soft boundary for t27c and Icarus at 1.174 MiBit.
- The mechanical checklist still prevents copy hazards; no compiler changes needed.
- Full `icarus_lowerable` suite runtime is ~25–46 seconds depending on caching.

---

## Next-wave cooperation variants

Prepared in `.claude/plans/wave-loop-892.md`.

### Variant A (recommended) — continue the ladder
- Module-scope `[603][2]^6 Pt` packed AoS variable from call with indexed signed writes.
- Outer dimension 603 → 38,592 elements → ~1.178 MiBit.
- Smallest, reviewable diff; keeps mechanical ladder moving.

### Variant B — increase inner struct width
- Keep outer dimension 601 but expand the inner struct (e.g., `[2]^8 Pt` or `[4]^6 Pt`).
- Tests whether the ceiling is element count or total packed-vector width.
- Larger spec and longer CI; defer until ladder hits a hard boundary.

### Variant C — stress negative/variable indexing
- Add non-constant signed index expressions inside the `bench` block.
- Probes Icarus index-normalization and cocotb reference-model agreement.
- Risk of compiler/reference-model changes; keep as a side experiment.

---

## Closure

- Issue #1843 closed by PR #1844.
- Next wave: #1845 — Wave Loop 892 (`[603][2]^6 Pt`).

phi^2 + 1/phi^2 = 3 | TRINITY
