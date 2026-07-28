# Current Issue — Wave Loop 812

| Field | Value |
|-------|-------|
| Wave | 812 |
| Issue | #1553 |
| Branch | `wave-loop-812` |
| Base | `wave-loop-811` @ `TBD` |
| Variant | `[443][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 28,352 elements × 32 bits = 907,264 bits (~0.865 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[443][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w812.py` with `OUTER = 443`, `MID_IDX = 221`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w812_bench_module_443x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w812_bench_module_443x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, and persistent memory updated.
- [ ] Commit with `Closes #1553`, push branch, open PR to `master`.

## Cooperation variants for W813
- **A (recommended):** `[445][2]^6 Pt`, outer += 2, MID_IDX = 222.
- **B:** `[443][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[443][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
