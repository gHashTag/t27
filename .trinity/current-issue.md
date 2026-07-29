# Current Issue — Wave Loop 818

| Field | Value |
|-------|-------|
| Wave | 818 |
| Issue | #1564 |
| Branch | `wave-loop-818` |
| Base | `wave-loop-817` (parent branch because earlier waves' PRs remain open) |
| Variant | `[455][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 29,120 elements × 32 bits = 931,840 bits (~0.889 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[455][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w818.py` with `OUTER = 455`, `MID_IDX = 227`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w818_bench_module_455x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w818_bench_module_455x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1564`, push branch, open PR to `master`.

## Cooperation variants for W819
- **A (recommended):** `[457][2]^6 Pt`, outer += 2, MID_IDX = 228.
- **B:** `[455][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[455][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
