# Current Issue — Wave Loop 828

| Field | Value |
|-------|-------|
| Wave | 828 |
| Issue | #1597 (expected) |
| Branch | `wave-loop-828` |
| Base | `wave-loop-827` (parent branch because earlier waves' PRs remain open) |
| Variant | `[475][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 30,400 elements × 32 bits = 972,800 bits (~0.927 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[475][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w828.py` with `OUTER = 475`, `MID_IDX = 237`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w828_bench_module_475x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w828_bench_module_475x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1597`, push branch, open PR to `master`.

## Cooperation variants for W829

- **A (recommended):** `[477][2]^6 Pt`, outer += 2, `MID_IDX = 238`.
- **B:** `[475][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[475][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
