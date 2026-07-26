# Current Issue — Wave Loop 805

| Field | Value |
|-------|-------|
| Wave | 805 |
| Issue | #1539 |
| Branch | `wave-loop-805` |
| Base | `wave-loop-804` @ `ab3782736bc2b270b4c3bb45cb78958a222e9a4e` |
| Variant | `[429][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 27,456 elements × 32 bits = 878,592 bits (~0.838 MiBit) |
| Status | in progress |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[429][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w805.py` with `OUTER = 429`, `MID_IDX = 214`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w805_bench_module_429x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w805_bench_module_429x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, and persistent memory updated.
- [ ] Commit with `Closes #1539`, push branch, open PR to `master`.

## Cooperation variants for W806
- **A (recommended):** `[431][2]^6 Pt`, outer += 2, MID_IDX = 215.
- **B:** `[425][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[425][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
