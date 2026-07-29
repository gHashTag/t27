# Current Issue — Wave Loop 816

| Field | Value |
|-------|-------|
| Wave | 816 |
| Issue | #1561 |
| Branch | `wave-loop-816` |
| Base | `wave-loop-815` (parent branch because earlier waves' PRs remain open) |
| Variant | `[451][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 28,864 elements × 32 bits = 923,648 bits (~0.881 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[451][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w816.py` with `OUTER = 451`, `MID_IDX = 225`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w816_bench_module_451x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w816_bench_module_451x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1561`, push branch, open PR to `master`.

## Cooperation variants for W817
- **A (recommended):** `[453][2]^6 Pt`, outer += 2, MID_IDX = 226.
- **B:** `[451][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[451][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
