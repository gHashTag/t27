# Wave Loop 884 Close-Out Report

**Date:** 2026-08-06
**Issue:** #1828
**Branch:** `wave-loop-884`
**Parent branch:** `wave-loop-883` HEAD
**PR:** #1829 (to `master`)
**Cooperation variant:** A (recommended)

## What was done

Wave Loop 884 continued the mechanical packed-vector array-of-struct ladder past the
1-MiBit line with the recommended Variant A:

- Module-scope `[587][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable
  initialized from a function call, with indexed signed field writes and `assert_eq`
  read-back in a `bench` block.
- Generator `scripts/gen_w884.py` copied from `scripts/gen_w883.py` and updated:
  `OUTER = 587`, `MID_IDX = 293`, destination path, module header, and `MID_IDX` comment.
- Produced `specs/scratch/w884_bench_module_587x2p6_aos_var_call_write.t27`
  (37,568 elements, 1,202,176-bit packed vector, ~1.147 MiBit).
- Added integration test `accepts_w884_bench_module_587x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed witness: `.trinity/seals/scratch_w884_bench_module_587x2p6_aos_var_call_write.json`
  (`sha256:ed5a44bf...`).

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | PASS |
| `t27c icarus-lowerable` | lowerable |
| `t27c icarus-simulate` | PASSED (17 cycles) |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| Targeted cargo test `accepts_w884...` | PASS |
| Full `cargo test --release --test icarus_lowerable` | 343 passed; 1 pre-existing failure¹ |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

¹ The pre-existing `corpus_classifier_matches_lean_completeness` mismatch for
`specs/cloud/railway_deploy.t27` (Rust lowerable `false`, Lean theorem `true`) is not
introduced by W884 and is tracked separately.

## Process notes

- PR #1829 was opened with auto-merge enabled. GitHub required status checks are currently
  `expected`, so merge is blocked pending Actions runner availability.
- The wave branch was created from `wave-loop-883` HEAD because earlier wave PRs remain open.
- The generator copy-hazard checklist was cleared before the first run; no stale references
  required a second pass.
- The branch was rebuilt from `master` to resolve merge conflicts with the GF-T stack that
  landed concurrently, preserving only the wave implementation commits.

## Next wave

- Wave Loop 885: module-scope `[589][2]^6 Pt` (Variant A).
- Plan: `.claude/plans/wave-loop-885.md`.

phi^2 + 1/phi^2 = 3 | TRINITY
