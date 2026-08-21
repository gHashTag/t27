# Wave Loop 883 Close-Out Report

**Date:** 2026-08-06
**Issue:** #1814
**Branch:** `wave-loop-883`
**Parent branch:** `wave-loop-882` HEAD (`85e2db7c`)
**PR:** #1815 (to `master`)
**Cooperation variant:** A (recommended)

## What was done

Wave Loop 883 continued the mechanical packed-vector array-of-struct ladder past the
1-MiBit line with the recommended Variant A:

- Module-scope `[585][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable
  initialized from a function call, with indexed signed field writes and `assert_eq`
  read-back in a `bench` block.
- Generator `scripts/gen_w883.py` copied from `scripts/gen_w882.py` and updated:
  `OUTER = 585`, `MID_IDX = 292`, destination path, module header, and `MID_IDX` comment.
- Produced `specs/scratch/w883_bench_module_585x2p6_aos_var_call_write.t27`
  (37,440 elements, 1,198,080-bit packed vector, ~1.143 MiBit).
- Added integration test `accepts_w883_bench_module_585x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed witness: `.trinity/seals/scratch_w883_bench_module_585x2p6_aos_var_call_write.json`
  (`sha256:80d27981...`).

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | PASS |
| `t27c icarus-lowerable` | lowerable |
| `t27c icarus-simulate` | PASSED (17 cycles) |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| Targeted cargo test `accepts_w883...` | PASS |
| Full `cargo test --release --test icarus_lowerable` | 342 passed; 1 pre-existing failure¹ |
| `bootstrap/stage0/FROZEN_HASH` | unchanged |

¹ The pre-existing `corpus_classifier_matches_lean_completeness` mismatch for
`specs/cloud/railway_deploy.t27` (Rust lowerable `false`, Lean theorem `true`) reproduces on
clean `wave-loop-882` and is not introduced by W883. It is being tracked separately.

## Process notes

- PR #1815 was opened with auto-merge enabled. GitHub required status checks are currently
  `expected`, so merge is blocked pending Actions runner availability — the same pattern as
  W881 (#1810) and W882 (#1813).
- The wave branch was created from `wave-loop-882` HEAD because earlier wave PRs remain open.
- The generator copy-hazard checklist was cleared before the first run; no stale references
  required a second pass.

## Next wave

- Wave Loop 884: module-scope `[587][2]^6 Pt` (Variant A).
- Plan: `.claude/plans/wave-loop-884.md`.
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W884_2026-08-06.md`.

phi^2 + 1/phi^2 = 3 | TRINITY
