# Current Issue — Wave Loop 817

| Field | Value |
|-------|-------|
| Wave | 817 |
| Issue | #1563 |
| Branch | `wave-loop-817` |
| Base | `wave-loop-816` (parent branch because earlier waves' PRs remain open) |
| Variant | `[453][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 29,056 elements × 32 bits = 929,792 bits (~0.886 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[453][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w817.py` with `OUTER = 453`, `MID_IDX = 226`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w817_bench_module_453x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w817_bench_module_453x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1563`, push branch, open PR to `master`.

## Cooperation variants for W818
- **A (recommended):** `[455][2]^6 Pt`, outer += 2, MID_IDX = 227.
- **B:** `[453][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[453][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
