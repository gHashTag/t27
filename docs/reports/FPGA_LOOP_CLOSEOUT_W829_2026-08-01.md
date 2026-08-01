# FPGA Wave Loop Closeout — W829

**Date:** 2026-08-01  
**Wave:** 829  
**Issue:** [#1599](https://github.com/gHashTag/t27/issues/1599)  
**Branch:** `wave-loop-829` (from `wave-loop-828` HEAD `0b6b534`)  
**PR:** #1600  
**Variant:** A — `[477][2]^6 Pt` module-scope non-power-of-two outer-dimension packed array-of-struct variable from call with indexed signed writes.

## Summary

Extended the module-scope packed array-of-struct ladder from `[475][2]^6 Pt`
(W828) to `[477][2]^6 Pt` (W829). The witness contains 30,528 32-bit `Pt`
elements, producing a single 980,992-bit packed vector (~0.934 MiBit). No changes
to the compiler, reference model, or `FROZEN_HASH` were required.

The recurring generator copy hazard was handled by the established mechanical
checklist (destination path + module header f-string + `MID_IDX` comment updated
to `238`).

## Work performed

1. Created issue #1599 and branch `wave-loop-829` from `wave-loop-828` HEAD.
2. Copied `scripts/gen_w828.py` → `scripts/gen_w829.py`.
3. Fixed generator copy hazard:
   - `DST` → `specs/scratch/w829_bench_module_477x2p6_aos_var_call_write.t27`
   - module header → `w829_bench_module_477x2p6_aos_var_call_write`
   - `MID_IDX` comment → `238`
4. Generated witness.
5. Ran direct validation gates:
   - `t27c parse` — PASS
   - `t27c icarus-lowerable` — `lowerable`
   - `t27c icarus-simulate` — 17 cycles, PASSED
   - `t27c icarus-cocotb` — reference-model OK
   - `t27c seal --save` — seal saved
6. Added integration test `accepts_w829_bench_module_477x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
7. Verified `bootstrap/stage0/FROZEN_HASH` unchanged.
8. Ran targeted and full integration tests — both green.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green (626 warnings, 0 errors) |
| `cargo test --release --test icarus_lowerable accepts_w829...` | 1/0 |
| `cargo test --release --test icarus_lowerable` (full suite) | 289/0 |
| Direct `t27c parse` W829 | PASS |
| Direct `t27c icarus-lowerable` W829 | lowerable |
| Direct `t27c icarus-simulate` W829 | 17 cycles, PASSED |
| Direct `t27c icarus-cocotb` W829 | reference-model OK |
| Direct `t27c seal --save` W829 | PASS |
| `FROZEN_HASH` | unchanged |

## Metrics

| Metric | Value |
|--------|-------|
| Outer dimension | 477 |
| Inner dimensions | `[2]^6` |
| Elements | 30,528 |
| Struct fields | `Pt { x : i16, y : i16 }` |
| Packed vector width | 980,992 bits |
| MiBit | ~0.934 |
| MID_IDX | 238 |
| MID_E (frame-condition element) | `238 * 64 + 32 = 15,264` |
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

## Cooperation variants for W830

| Variant | Description | Outer | MID_IDX | Elements | Bits | MiBit |
|---------|-------------|-------|---------|----------|------|-------|
| A (recommended) | Continue ladder by +2 | `[479][2]^6 Pt` | 239 | 30,656 | 985,088 | ~0.939 |
| B | Grow second inner dimension to stress stride scaling | `[477][3]^6 Pt` | 238 | 45,792 | 1,465,344 | ~1.397 |
| C | Add negative-index writes to exercise wrap-around | `[477][2]^6 Pt` | 238 | 30,528 | 980,992 | ~0.934 |

## Artifacts touched

- Added:
  - `scripts/gen_w829.py`
  - `specs/scratch/w829_bench_module_477x2p6_aos_var_call_write.t27`
  - `.trinity/seals/scratch_w829_bench_module_477x2p6_aos_var_call_write.json`
  - `docs/reports/FPGA_LOOP_CLOSEOUT_W829_2026-08-01.md`
  - `.claude/plans/wave-loop-830.md`
- Modified:
  - `bootstrap/tests/icarus_lowerable.rs`
  - `docs/NOW.md`
  - `.trinity/current-issue.md`
  - `.trinity/experience.md`
  - `.claude/skills/t27-wave-loop.md`
  - `.claude/skills/wave-loop-autopilot.md`

## Next steps

1. Stage W829 artifacts.
2. Commit with `feat(igla): Wave Loop 829 — module-scope [477][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes`.
3. Body: `Closes #1599`.
4. Push `wave-loop-829`.
5. Open PR #1600 to `master`.
6. Update persistent memory with W829 closeout.

*φ² + φ⁻² = 3 | TRINITY*
