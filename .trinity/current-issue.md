# Current Issue — Wave Loop 808

| Field | Value |
|-------|-------|
| Wave | 808 |
| Issue | #1545 |
| Branch | `wave-loop-808` |
| Base | `wave-loop-807` @ `TBD` |
| Variant | `[435][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 27,840 elements × 32 bits = 890,880 bits (~0.849 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[435][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w808.py` with `OUTER = 435`, `MID_IDX = 217`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w808_bench_module_435x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w808_bench_module_435x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, and persistent memory updated.
- [ ] Commit with `Closes #1545`, push branch, open PR to `master`.

## Cooperation variants for W809
- **A (recommended):** `[437][2]^6 Pt`, outer += 2, MID_IDX = 218.
- **B:** `[435][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[435][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
