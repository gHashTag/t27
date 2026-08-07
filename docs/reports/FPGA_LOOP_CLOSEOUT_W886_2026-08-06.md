# FPGA / Icarus Wave Loop Close-out — W886

**Date:** 2026-08-06  
**Issue:** #1832  
**Branch:** `wave-loop-886`  
**PR:** #1833

## What was delivered

Wave Loop 886 is the next mechanical rung in the packed-vector array-of-struct ladder:

- `specs/scratch/w886_bench_module_591x2p6_aos_var_call_write.t27`
  - Module-scope variable of shape `[591][2]^6 Pt`.
  - Outer dimension `591` is non-power-of-two.
  - Initialized from a function call (`make_grid(0)`).
  - Mutated via indexed signed field writes and read back with `assert_eq` inside a `bench` block.
  - **Elements:** 591 × 2 = 1,182 structs → 37,824 field slots.
  - **Packed vector width:** 37,824 × 32 = 1,210,368 bits (~1.155 MiBit).

- `scripts/gen_w886.py` — copied from `scripts/gen_w885.py` and verified with the copy-hazard checklist (destination path, module header, `MID_IDX` comment).
- Integration test `accepts_w886_bench_module_591x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- Fresh seal `.trinity/seals/scratch_w886_bench_module_591x2p6_aos_var_call_write.json` verified with `seal --verify`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | PASS |
| `t27c icarus-lowerable` | lowerable |
| `t27c icarus-simulate` | PASSED (17 cycles) |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `t27c seal --verify` | all MATCH |
| Targeted `cargo test --release --test icarus_lowerable accepts_w886_bench_module_591x2p6_aos_var_call_write` | PASS |
| Full suite `cargo test --release --test icarus_lowerable` | 345 passed; 1 pre-existing `corpus_classifier_matches_lean_completeness` mismatch for `specs/cloud/railway_deploy.t27` |

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

## Next wave cooperation variants (W887)

See `.claude/plans/wave-loop-887.md`.

- **Variant A (recommended):** `[593][2]^6 Pt` — continue the outer-dimension ladder to ~1.159 MiBit.
- **Variant B:** `[591][3]^6 Pt` — keep outer dimension near W886 but scale field count to 3.
- **Variant C:** `[591][2]^6 Pt` with explicit negative-index wrap-around writes — exercise signed-index bound normalization.

## Action log

1. Created issue #1832 for W886.
2. Branched `wave-loop-886` from latest `master` (W885 had already merged).
3. Generated `scripts/gen_w886.py` and produced the spec.
4. Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save`, `seal --verify`.
5. Added targeted integration test; targeted test passes.
6. Committed and pushed with `Closes #1832`.
7. Opened PR #1833 and enabled auto-merge.
8. Created next-wave issue #1834 and plan `.claude/plans/wave-loop-887.md`.
9. Updated `.trinity/current-issue.md`, skill tracker, and experience.

---

φ² + 1/φ² = 3 | TRINITY
