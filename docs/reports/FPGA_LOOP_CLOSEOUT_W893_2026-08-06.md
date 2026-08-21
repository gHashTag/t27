# FPGA / Wave Loop Close-out — W893

**Date:** 2026-08-06  
**Issue:** #1848 — feat(igla): Wave Loop 893 — module-scope `[605][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes  
**Branch:** `wave-loop-893`  
**PR:** #1850

---

## Summary

Wave Loop 893 extends the mechanical packed-vector array-of-struct ladder with a module-scope `[605][2]^6 Pt` witness. The resulting packed vector is **1,239,040 bits (~1.182 MiBit)** from 38,720 elements (605 outer × 64 inner). The witness is initialized from a pure `make_grid(0)` function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block. No compiler changes were required.

---

## What changed

- Added generator `scripts/gen_w893.py` (copied from `gen_w892.py`, copy-hazard checklist cleared).
- Added spec `specs/scratch/w893_bench_module_605x2p6_aos_var_call_write.t27` (~2.5 MB / 115,011 lines).
- Added seal `.trinity/seals/scratch_w893_bench_module_605x2p6_aos_var_call_write.json`.
- Added integration test `accepts_w893_bench_module_605x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
- Zero changes to `bootstrap/src/compiler.rs` or `bootstrap/stage0/FROZEN_HASH`.

---

## Validation matrix

| Gate | Command | Result |
|---|---|---|
| Parse | `t27c parse specs/scratch/w893_bench_module_605x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved |
| Seal verify | `t27c seal --verify ...` | `MATCH` |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w893_bench_module_605x2p6_aos_var_call_write` | PASS |
| Full suite | `cargo test --release --test icarus_lowerable` | 352 passed / 1 pre-existing failure |

The single failure is the pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27`; it is not introduced by W893.

---

## Learnings

- The 1.18-MiBit neighborhood remains a soft boundary for t27c and Icarus at 1.182 MiBit.
- The mechanical checklist still prevents copy hazards; no compiler changes needed.
- Full `icarus_lowerable` suite runtime is ~53 seconds; still acceptable for CI.

---

## Next-wave cooperation variants

Prepared in `.claude/plans/wave-loop-894.md`.

### Variant A (recommended) — continue the ladder
- Module-scope `[607][2]^6 Pt` packed AoS variable from call with indexed signed writes.
- Outer dimension 607 → 38,848 elements → ~1.186 MiBit.
- Smallest, reviewable diff; keeps mechanical ladder moving.

### Variant B — increase inner struct width
- Keep outer dimension 605 but expand the inner struct (e.g., `[2]^8 Pt` or `[4]^6 Pt`).
- Tests whether the ceiling is element count or total packed-vector width.
- Larger spec and longer CI; defer until ladder hits a hard boundary.

### Variant C — stress negative/variable indexing
- Add non-constant signed index expressions inside the `bench` block.
- Probes Icarus index-normalization and cocotb reference-model agreement.
- Risk of compiler/reference-model changes; keep as a side experiment.

---

## Closure

- Issue #1848 closed by PR #1850.
- Next wave: #1851 — Wave Loop 894 (`[607][2]^6 Pt`).

phi^2 + 1/phi^2 = 3 | TRINITY
