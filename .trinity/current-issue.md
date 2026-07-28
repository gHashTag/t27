# Current Issue — Wave Loop 813

| Field | Value |
|-------|-------|
| Wave | 813 |
| Issue | #1555 |
| Branch | `wave-loop-813` |
| Base | `wave-loop-812` @ `TBD` |
| Variant | `[445][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 28,480 elements × 32 bits = 911,360 bits (~0.869 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[445][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w813.py` with `OUTER = 445`, `MID_IDX = 222`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w813_bench_module_445x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w813_bench_module_445x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1555`, push branch, open PR to `master`.

## Cooperation variants for W814
- **A (recommended):** `[447][2]^6 Pt`, outer += 2, MID_IDX = 223.
- **B:** `[445][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[445][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
