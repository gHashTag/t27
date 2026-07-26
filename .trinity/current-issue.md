# Current Issue — Wave Loop 804

| Field | Value |
|-------|-------|
| Wave | 804 |
| Issue | #1537 |
| Branch | `wave-loop-804` |
| Base | `wave-loop-803` @ `d1ee170f48666f720bff35c85fd6134bdf9e32c1` |
| Variant | `[427][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 27,328 elements × 32 bits = 875,008 bits (~0.834 MiBit) |
| Status | in progress |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[427][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w804.py` with `OUTER = 427`, `MID_IDX = 213`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w804_bench_module_427x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w804_bench_module_427x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, and persistent memory updated.
- [ ] Commit with `Closes #1537`, push branch, open PR to `master`.

## Cooperation variants for W805
- **A (recommended):** `[429][2]^6 Pt`, outer += 2, MID_IDX = 214.
- **B:** `[425][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[425][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
