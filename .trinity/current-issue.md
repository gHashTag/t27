# Current Issue — Wave Loop 811

| Field | Value |
|-------|-------|
| Wave | 811 |
| Issue | #1551 |
| Branch | `wave-loop-811` |
| Base | `wave-loop-810` @ `TBD` |
| Variant | `[441][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 28,224 elements × 32 bits = 903,168 bits (~0.861 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[441][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w811.py` with `OUTER = 441`, `MID_IDX = 220`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w811_bench_module_441x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w811_bench_module_441x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, and persistent memory updated.
- [ ] Commit with `Closes #1551`, push branch, open PR to `master`.

## Cooperation variants for W812
- **A (recommended):** `[443][2]^6 Pt`, outer += 2, MID_IDX = 221.
- **B:** `[441][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[441][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
