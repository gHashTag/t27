# Current Issue — Wave Loop 803

| Field | Value |
|-------|-------|
| Wave | 803 |
| Issue | #1535 |
| Branch | `wave-loop-803` |
| Base | `wave-loop-802` @ `7058c0652ce1fc886bcf3d9235749de8d0a6ba70` |
| Variant | `[425][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 27,200 elements × 32 bits = 870,400 bits (~0.830 MiBit) |
| Status | in progress |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[425][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w803.py` with `OUTER = 425`, `MID_IDX = 212`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w803_bench_module_425x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w803_bench_module_425x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, and persistent memory updated.
- [ ] Commit with `Closes #1535`, push branch, open PR to `master`.

## Cooperation variants for W804
- **A (recommended):** `[427][2]^6 Pt`, outer += 2, MID_IDX = 213.
- **B:** `[425][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[425][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
