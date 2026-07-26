# Current Issue — Wave Loop 809

| Field | Value |
|-------|-------|
| Wave | 809 |
| Issue | #1547 |
| Branch | `wave-loop-809` |
| Base | `wave-loop-808` @ `TBD` |
| Variant | `[437][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 27,968 elements × 32 bits = 894,976 bits (~0.853 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[437][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w809.py` with `OUTER = 437`, `MID_IDX = 218`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w809_bench_module_437x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w809_bench_module_437x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, and persistent memory updated.
- [ ] Commit with `Closes #1547`, push branch, open PR to `master`.

## Cooperation variants for W810
- **A (recommended):** `[439][2]^6 Pt`, outer += 2, MID_IDX = 219.
- **B:** `[437][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[437][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
