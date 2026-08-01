# FPGA Wave Loop Closeout — W827

**Date:** 2026-08-01  
**Wave:** 827  
**Issue:** [#1595](https://github.com/gHashTag/t27/issues/1595)  
**Branch:** `wave-loop-827` (from `wave-loop-826` HEAD `7645f1d`)  
**PR:** #1596  
**Variant:** A — `[473][2]^6 Pt` module-scope non-power-of-two outer-dimension packed array-of-struct variable from call with indexed signed writes.

## Summary

Extended the module-scope packed array-of-struct ladder from `[471][2]^6 Pt`
(W826) to `[473][2]^6 Pt` (W827). The witness contains 30,272 32-bit `Pt`
elements, producing a single 968,704-bit packed vector (~0.923 MiBit). No changes
to the compiler, reference model, or `FROZEN_HASH` were required.

The recurring generator copy hazard was handled by the established mechanical
checklist (destination path + module header f-string + `MID_IDX` comment updated
to `236`).

## Work performed

1. Created issue #1595 and branch `wave-loop-827` from `wave-loop-826` HEAD.
2. Copied `scripts/gen_w826.py` → `scripts/gen_w827.py`.
3. Fixed generator copy hazard:
   - `DST` → `specs/scratch/w827_bench_module_473x2p6_aos_var_call_write.t27`
   - module header → `w827_bench_module_473x2p6_aos_var_call_write`
   - `MID_IDX` comment → `236`
4. Generated witness.
5. Ran direct validation gates:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — `lowerable`
   - `t27c icarus-simulate` — 17 cycles, PASSED
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved
6. Added integration test `accepts_w827_bench_module_473x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
7. Verified `bootstrap/stage0/FROZEN_HASH` unchanged.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green (626 warnings, 0 errors) |
| `cargo test --release --test icarus_lowerable accepts_w827...` | 1/0 |
| `cargo test --release --test icarus_lowerable` (full suite) | 287/0 |
| Direct `t27c parse` W827 | PASS |
| Direct `t27c icarus-lowerable` W827 | lowerable |
| Direct `t27c icarus-simulate` W827 | 17 cycles, PASSED |
| Direct `t27c icarus-cocotb` W827 | reference-model OK |
| Direct `t27c seal --save` W827 | PASS |
| `FROZEN_HASH` | unchanged |

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 473 |
| Inner dimensions | `[2]^6` |
| Elements | 30,272 |
| Struct fields | `Pt { x : i16, y : i16 }` |
| Packed vector width | 968,704 bits |
| MiBit | ~0.923 |
| MID_IDX | 236 |
| MID_E (frame-condition element) | `236 * 64 + 32 = 15,136` |
| Simulation cycles | 17 |
| Compiler changes | 0 |
| Reference model changes | 0 |
| `FROZEN_HASH` changes | 0 |

## Weak points / risks

- **Generator copy hazard** remains the only manual failure mode in the otherwise
  mechanical ladder. Both the destination path and the module header f-string must
  be grepped for stale wave numbers after each copy. Parameterizing the wave
  prefix in the generator template would remove this.
- **30-day subject-line traceability** still below target; continue putting
  `Closes #N` in commit subjects.
- Pre-existing `verilog_array_literal_expr` regression and FPGA E2E CI red remain
  out of scope for this witness ladder.

## Cooperation variants for W828

| Variant | Description | Outer | MID_IDX | Elements | Bits | MiBit |
|---------|-------------|-------|---------|----------|------|-------|
| A (recommended) | Continue ladder by +2 | `[475][2]^6 Pt` | 237 | 30,400 | 972,800 | ~0.927 |
| B | Grow second inner dimension to stress stride scaling | `[473][3]^6 Pt` | 236 | 45,408 | 1,453,056 | ~1.385 |
| C | Add negative-index writes to exercise wrap-around | `[473][2]^6 Pt` | 236 | 30,272 | 968,704 | ~0.923 |

## Artifacts touched

- Added:
  - `scripts/gen_w827.py`
  - `specs/scratch/w827_bench_module_473x2p6_aos_var_call_write.t27`
  - `.trinity/seals/scratch_w827_bench_module_473x2p6_aos_var_call_write.json`
  - `docs/reports/FPGA_LOOP_CLOSEOUT_W827_2026-08-01.md`
  - `.claude/plans/wave-loop-828.md`
- Modified:
  - `bootstrap/tests/icarus_lowerable.rs`
  - `docs/NOW.md`
  - `.trinity/current-issue.md`
  - `.trinity/experience.md`
  - `.claude/skills/t27-wave-loop.md`
  - `.claude/skills/wave-loop-autopilot.md`

## Next steps

1. Stage W827 artifacts.
2. Commit with `feat(igla): Wave Loop 827 — module-scope [473][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes`.
3. Body: `Closes #1595`.
4. Push `wave-loop-827`.
5. Open PR #1596 to `master`.
6. Update persistent memory with W827 closeout.

*φ² + φ⁻² = 3 | TRINITY*
