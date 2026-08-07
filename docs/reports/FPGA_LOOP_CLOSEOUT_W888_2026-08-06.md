# FPGA / Icarus Wave Loop Close-out — W888

**Date:** 2026-08-06  
**Issue:** #1836  
**Branch:** `wave-loop-888`  
**PR:** #1837

## What was delivered

Wave Loop 888 is the next mechanical rung in the packed-vector array-of-struct ladder:

- `specs/scratch/w888_bench_module_595x2p6_aos_var_call_write.t27`
  - Module-scope variable of shape `[595][2]^6 Pt`.
  - Outer dimension `595` is non-power-of-two.
  - Initialized from a function call (`make_grid(0)`).
  - Mutated via indexed signed field writes and read back with `assert_eq` inside a `bench` block.
  - **Elements:** 595 × 64 = 38,080 structs.
  - **Packed vector width:** 38,080 × 32 = 1,218,560 bits (~1.162 MiBit).

- `scripts/gen_w888.py` — copied from `scripts/gen_w887.py` and verified with the copy-hazard checklist (destination path, module header, `MID_IDX` comment).
- Integration test `accepts_w888_bench_module_595x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- Fresh seal `.trinity/seals/scratch_w888_bench_module_595x2p6_aos_var_call_write.json` verified with `seal --verify`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | PASS |
| `t27c icarus-lowerable` | lowerable |
| `t27c icarus-simulate` | PASSED (17 cycles) |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `t27c seal --verify` | all MATCH |
| Targeted `cargo test --release --test icarus_lowerable accepts_w888_bench_module_595x2p6_aos_var_call_write` | PASS |
| Full suite `cargo test --release --test icarus_lowerable` | 347 passed; 1 pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27` |

## Invariants

- No changes to `bootstrap/src/compiler.rs` or reference model.
- `bootstrap/stage0/FROZEN_HASH` unchanged.
- All generated files ASCII-only, sealed, and lowerable.

## Research background (unchanged)

The same weak-point context from prior waves applies:

- Icarus Verilog has no 1-MiBit hard cap (LRM minimum 65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed bound-normalization; Icarus V13.0 2026-03-02 improves packed/unpacked array handling).
- Vitis HLS UG1399 `compact=bit` is the commercial analog for packing structs into wide vectors.
- Vericert v2.0.0 (2026-01-29), 2024 PLDI verified hyperblock scheduling (DOI 10.1145/3656455), Graphiti (ASPLOS 2026), and Let It Flow (PLDI 2026) frame the verified-HLS context.
- FPGA Roofline (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.

## Next wave cooperation variants (W889)

See `.claude/plans/wave-loop-889.md`.

- **Variant A (recommended):** `[597][2]^6 Pt` — continue the outer-dimension ladder to ~1.166 MiBit.
- **Variant B:** `[595][3]^6 Pt` — keep outer dimension near W888 but scale field count to 3.
- **Variant C:** `[595][2]^6 Pt` with explicit negative-index wrap-around writes — exercise signed-index bound normalization.

## Action log

1. Created issue #1836 for W888.
2. Branched `wave-loop-888` from `wave-loop-887` HEAD (earlier wave PRs remain open).
3. Generated `scripts/gen_w888.py` and produced the spec.
4. Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save`, `seal --verify`.
5. Added targeted integration test; targeted test passes.
6. Committed and pushed with `Closes #1836`.
7. Opened PR #1837 and enabled auto-merge.
8. Rebased `wave-loop-888` onto latest `master` after W886/W887 landed, then force-pushed, because the branch protection rule requires the head to be up-to-date.
9. Created next-wave issue #1838 and plan `.claude/plans/wave-loop-889.md`.
10. Updated `.trinity/current-issue.md`, skill tracker, experience, and persistent memory.

---

φ² + 1/φ² = 3 | TRINITY
