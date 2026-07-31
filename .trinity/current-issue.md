# Current Issue — Wave Loop 823

| Field | Value |
|-------|-------|
| Wave | 823 |
| Issue | #1585 |
| Branch | `wave-loop-823` |
| Base | `wave-loop-822` (parent branch because earlier waves' PRs remain open) |
| Variant | `[465][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 29,760 elements × 32 bits = 952,320 bits (~0.908 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[465][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [x] Generator `scripts/gen_w823.py` with `OUTER = 465`, `MID_IDX = 232`; copy hazard fixed before first run.
- [x] Witness `specs/scratch/w823_bench_module_465x2p6_aos_var_call_write.t27` generated and parsed.
- [x] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [x] Integration test `accepts_w823_bench_module_465x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [x] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [x] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [x] Commit with `Closes #1585`, push branch, open PR to `master`.

## Cooperation variants for W824
- **A (recommended):** `[467][2]^6 Pt`, outer += 2, MID_IDX = 233.
- **B:** `[465][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[465][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
